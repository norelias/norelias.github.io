//! Stockage **local** du profil : JSON versionné, écriture atomique, sauvegarde `.bak`,
//! récupération d'un fichier corrompu, migrations. Aucun compte, aucun réseau.

use crate::privacy::ConsentLedger;
use crate::profile::{Profile, SCHEMA_VERSION};
use serde_json::Value;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum StoreError {
    Io(std::io::Error),
    Json(serde_json::Error),
    /// Fichier écrit par une version plus récente : on refuse de l'écraser.
    NewerVersion(u32),
}
impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self { Self::Io(e) => write!(f, "erreur disque : {e}"), Self::Json(e) => write!(f, "profil illisible : {e}"),
            Self::NewerVersion(v) => write!(f, "profil créé par une version plus récente (schéma {v})") }
    }
}
impl std::error::Error for StoreError {}
impl From<std::io::Error> for StoreError { fn from(e: std::io::Error) -> Self { Self::Io(e) } }
impl From<serde_json::Error> for StoreError { fn from(e: serde_json::Error) -> Self { Self::Json(e) } }

#[derive(Debug, PartialEq, Eq)]
pub enum Recovery { None, FromBackup, ResetToDefaults }

pub struct Loaded { pub profile: Profile, pub recovery: Recovery, pub quarantined: Option<PathBuf> }

pub struct Store { dir: PathBuf }

impl Store {
    pub fn new(dir: impl Into<PathBuf>) -> Self { Self { dir: dir.into() } }
    fn path(&self, name: &str) -> PathBuf { self.dir.join(name) }

    /// Écriture atomique : fichier temporaire, fsync, copie `.bak` de l'ancienne version, renommage.
    fn write_atomic(&self, name: &str, bytes: &[u8]) -> Result<(), StoreError> {
        fs::create_dir_all(&self.dir)?;
        let (dst, tmp, bak) = (self.path(name), self.path(&format!("{name}.tmp")), self.path(&format!("{name}.bak")));
        { let mut f = File::create(&tmp)?; f.write_all(bytes)?; f.sync_all()?; }
        if dst.exists() { let _ = fs::copy(&dst, &bak); }
        fs::rename(&tmp, &dst)?;
        Ok(())
    }

    pub fn save(&self, profile: &Profile) -> Result<(), StoreError> {
        self.write_atomic("profile.json", serde_json::to_string_pretty(profile)?.as_bytes())
    }

    pub fn load(&self) -> Result<Loaded, StoreError> {
        let main = self.path("profile.json");
        if !main.exists() {
            return Ok(Loaded { profile: Profile::default(), recovery: Recovery::None, quarantined: None });
        }
        match Self::read_profile(&main) {
            Ok(p) => Ok(Loaded { profile: p, recovery: Recovery::None, quarantined: None }),
            Err(StoreError::NewerVersion(v)) => Err(StoreError::NewerVersion(v)),
            Err(_) => {
                // Fichier corrompu : on le met de côté (jamais supprimé), puis on tente la sauvegarde.
                let q = self.path(&format!("profile.json.corrupt-{}", now()));
                fs::rename(&main, &q)?;
                match Self::read_profile(&self.path("profile.json.bak")) {
                    Ok(p) => Ok(Loaded { profile: p, recovery: Recovery::FromBackup, quarantined: Some(q) }),
                    Err(_) => Ok(Loaded { profile: Profile::default(), recovery: Recovery::ResetToDefaults, quarantined: Some(q) }),
                }
            }
        }
    }

    fn read_profile(path: &Path) -> Result<Profile, StoreError> {
        let v: Value = serde_json::from_slice(&fs::read(path)?)?;
        Ok(serde_json::from_value(migrate(v)?)?)
    }

    pub fn load_ledger(&self) -> Result<ConsentLedger, StoreError> {
        let p = self.path("consent.json");
        if !p.exists() { return Ok(ConsentLedger::default()); }
        Ok(serde_json::from_slice(&fs::read(p)?)?)
    }
    pub fn save_ledger(&self, l: &ConsentLedger) -> Result<(), StoreError> {
        self.write_atomic("consent.json", serde_json::to_string_pretty(l)?.as_bytes())
    }

    /// Exporte le profil pour le transférer vers un autre poste.
    pub fn export_to(&self, dest: &Path) -> Result<(), StoreError> {
        fs::copy(self.path("profile.json"), dest)?;
        Ok(())
    }

    /// Supprime toutes les données d'intégration locales (profil, sauvegardes, consentements).
    /// Les fichiers sont d'abord écrasés par des zéros ; sur SSD/systèmes journalisés, cet
    /// écrasement n'est **pas** une garantie d'effacement physique (voir SECURITY.md).
    pub fn erase_all(&self) -> Result<usize, StoreError> {
        let mut n = 0;
        if !self.dir.exists() { return Ok(0); }
        for e in fs::read_dir(&self.dir)? {
            let p = e?.path();
            if p.is_file() {
                let len = fs::metadata(&p)?.len() as usize;
                { let mut f = File::create(&p)?; f.write_all(&vec![0u8; len])?; f.sync_all()?; }
                fs::remove_file(&p)?;
                n += 1;
            }
        }
        Ok(n)
    }
}

fn now() -> u64 { std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) }

/// Migrations successives jusqu'à `SCHEMA_VERSION`.
/// v0 (prototype) : un champ `level` unique et `lang` ; v1 : deux niveaux séparés, `language`.
fn migrate(mut v: Value) -> Result<Value, StoreError> {
    let ver = v.get("schema_version").and_then(Value::as_u64).unwrap_or(0) as u32;
    if ver > SCHEMA_VERSION { return Err(StoreError::NewerVersion(ver)); }
    let obj = v.as_object_mut().ok_or_else(|| serde_json::from_str::<Value>("}").unwrap_err())?;
    if ver < 1 {
        if let Some(level) = obj.remove("level") {
            obj.entry("photo_level").or_insert(level.clone());
            obj.entry("retouch_level").or_insert(level);
        }
        if let Some(lang) = obj.remove("lang") { obj.entry("language").or_insert(lang); }
        obj.insert("schema_version".into(), Value::from(1));
    }
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::{Coaching, Language, LegacySoftware, Skill, Specialty};

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("norelias-test-{name}-{}-{}", std::process::id(), now()));
        let _ = fs::remove_dir_all(&d);
        d
    }

    #[test]
    fn first_run_has_no_file_and_needs_onboarding() {
        let s = Store::new(tmp("first"));
        let l = s.load().unwrap();
        assert!(l.profile.needs_onboarding());
        assert_eq!(l.recovery, Recovery::None);
    }

    #[test]
    fn save_then_reload_roundtrips_every_answer() {
        let s = Store::new(tmp("roundtrip"));
        let mut p = Profile::default();
        p.photo_level = Skill::Professional;
        p.specialties = vec![Specialty::Wedding, Specialty::Portrait];
        p.previous_software = vec![LegacySoftware::Lightroom, LegacySoftware::Other("DxO".into())];
        p.set_camera("Nikon", "D5200").unwrap();
        p.coaching = Coaching::Off;
        p.language = Language::Fr;
        p.complete();
        s.save(&p).unwrap();
        assert_eq!(s.load().unwrap().profile, p);
    }

    #[test]
    fn answers_can_be_changed_later() {
        let s = Store::new(tmp("edit"));
        let mut p = Profile::default();
        p.complete();
        s.save(&p).unwrap();
        let mut p2 = s.load().unwrap().profile;
        p2.theme = crate::profile::Theme::Light;
        p2.specialties.push(Specialty::Landscape);
        s.save(&p2).unwrap();
        assert_eq!(s.load().unwrap().profile.specialties, vec![Specialty::Landscape]);
    }

    #[test]
    fn no_temporary_file_is_left_behind() {
        let dir = tmp("atomic");
        let s = Store::new(&dir);
        s.save(&Profile::default()).unwrap();
        s.save(&Profile::default()).unwrap();
        let names: Vec<String> = fs::read_dir(&dir).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().into()).collect();
        assert!(!names.iter().any(|n| n.ends_with(".tmp")), "{names:?}");
        assert!(names.contains(&"profile.json.bak".to_string()));
    }

    #[test]
    fn corrupted_file_is_quarantined_and_backup_restored() {
        let dir = tmp("corrupt");
        let s = Store::new(&dir);
        let mut p = Profile::default();
        p.set_camera("Canon", "R6").unwrap();
        s.save(&p).unwrap();
        s.save(&p).unwrap(); // crée le .bak
        fs::write(dir.join("profile.json"), b"{ coupure de courant").unwrap();
        let l = s.load().unwrap();
        assert_eq!(l.recovery, Recovery::FromBackup);
        assert_eq!(l.profile.camera, p.camera);
        assert!(l.quarantined.unwrap().exists(), "le fichier corrompu doit être conservé pour diagnostic");
    }

    #[test]
    fn corrupted_without_backup_resets_to_defaults() {
        let dir = tmp("corrupt2");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("profile.json"), b"nope").unwrap();
        let l = Store::new(&dir).load().unwrap();
        assert_eq!(l.recovery, Recovery::ResetToDefaults);
        assert!(l.profile.needs_onboarding());
    }

    #[test]
    fn v0_profile_is_migrated() {
        let dir = tmp("migrate");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("profile.json"), br#"{"level":"advanced","lang":"fr","status":"completed"}"#).unwrap();
        let p = Store::new(&dir).load().unwrap().profile;
        assert_eq!(p.photo_level, Skill::Advanced);
        assert_eq!(p.retouch_level, Skill::Advanced);
        assert_eq!(p.language, Language::Fr);
        assert_eq!(p.schema_version, 1);
    }

    #[test]
    fn newer_schema_is_refused_and_left_untouched() {
        let dir = tmp("newer");
        fs::create_dir_all(&dir).unwrap();
        let raw = br#"{"schema_version":99}"#;
        fs::write(dir.join("profile.json"), raw).unwrap();
        assert!(matches!(Store::new(&dir).load(), Err(StoreError::NewerVersion(99))));
        assert_eq!(fs::read(dir.join("profile.json")).unwrap(), raw);
    }

    #[test]
    fn unknown_and_missing_fields_are_tolerated() {
        let dir = tmp("lenient");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("profile.json"), br#"{"schema_version":1,"theme":"dark","future_field":42}"#).unwrap();
        let p = Store::new(&dir).load().unwrap().profile;
        assert_eq!(p.theme, crate::profile::Theme::Dark);
    }

    #[test]
    fn erase_all_removes_everything() {
        let dir = tmp("erase");
        let s = Store::new(&dir);
        s.save(&Profile::default()).unwrap();
        s.save(&Profile::default()).unwrap();
        s.save_ledger(&ConsentLedger::default()).unwrap();
        assert!(s.erase_all().unwrap() >= 3);
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 0);
    }
}
