use norelias_core::Lang;
use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Skill { Beginner, Intermediate, Advanced, Professional }

/// Mode d'espace de travail. Même moteur pour les trois.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode { Beginner, Intermediate, Expert }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Specialty {
    Portrait, Wedding, Family, Newborn, Landscape, Wildlife, Sports, Events,
    Product, RealEstate, Street, Studio, School, Other,
}
impl Specialty {
    pub const ALL: [Specialty; 14] = [
        Specialty::Portrait, Specialty::Wedding, Specialty::Family, Specialty::Newborn, Specialty::Landscape,
        Specialty::Wildlife, Specialty::Sports, Specialty::Events, Specialty::Product, Specialty::RealEstate,
        Specialty::Street, Specialty::Studio, Specialty::School, Specialty::Other,
    ];
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LegacySoftware {
    Photoshop, Lightroom, CaptureOne, Evoto, LuminarNeo, TopazPhotoAi, PhotoMechanic, None, Other(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Camera { pub make: String, pub model: String }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Goal { Hobby, Portfolio, PaidWork, Learn, SaveTime, Archive }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiNeed { Selection, Denoise, Upscale, ObjectRemoval, PortraitRetouch, NaturalLanguage, None }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrintNeed { Prints, Albums, ClientGalleries, WebExport, SocialMedia, Products }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Coaching { Guided, OnDemand, Off }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Language { System, Fr, En }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme { System, Dark, Light }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShortcutProfileId { Norelias, PhotoshopLike, LightroomLike, CaptureOneLike }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OnboardingStatus { Pending, Completed, Skipped }

/// Préférences de stockage. Les consentements (cloud, télémétrie) ne sont **pas** ici : voir `privacy`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct StoragePrefs {
    pub catalog_dir: Option<String>,
    pub backup_dir: Option<String>,
    pub auto_backup: bool,
    pub cache_limit_mb: u32,
}
impl Default for StoragePrefs {
    fn default() -> Self { Self { catalog_dir: None, backup_dir: None, auto_backup: true, cache_limit_mb: 4096 } }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Profile {
    pub schema_version: u32,
    pub status: OnboardingStatus,
    pub photo_level: Skill,
    pub retouch_level: Skill,
    pub specialties: Vec<Specialty>,
    pub previous_software: Vec<LegacySoftware>,
    pub camera: Option<Camera>,
    pub goals: Vec<Goal>,
    pub ai_needs: Vec<AiNeed>,
    pub print_needs: Vec<PrintNeed>,
    pub coaching: Coaching,
    pub tooltips: bool,
    /// Suivi de progression de l'Académie : désactivé par défaut.
    pub track_progress: bool,
    pub mode_override: Option<Mode>,
    pub shortcut_profile: ShortcutProfileId,
    pub language: Language,
    pub theme: Theme,
    pub storage: StoragePrefs,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            status: OnboardingStatus::Pending,
            photo_level: Skill::Intermediate,
            retouch_level: Skill::Beginner,
            specialties: vec![],
            previous_software: vec![],
            camera: None,
            goals: vec![],
            ai_needs: vec![],
            print_needs: vec![],
            coaching: Coaching::OnDemand,
            tooltips: true,
            track_progress: false,
            mode_override: None,
            shortcut_profile: ShortcutProfileId::Norelias,
            language: Language::System,
            theme: Theme::System,
            storage: StoragePrefs::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ProfileError { BadText(&'static str), TooLong(&'static str), BadNumber(&'static str) }
impl std::fmt::Display for ProfileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self { Self::BadText(w) => write!(f, "{w} : caractères invalides"), Self::TooLong(w) => write!(f, "{w} : trop long"),
            Self::BadNumber(w) => write!(f, "{w} : valeur hors limites") }
    }
}
impl std::error::Error for ProfileError {}

fn clean(s: &str, what: &'static str) -> Result<String, ProfileError> {
    let t = s.trim();
    if t.chars().any(|c| c.is_control()) { return Err(ProfileError::BadText(what)); }
    if t.chars().count() > 80 { return Err(ProfileError::TooLong(what)); }
    Ok(t.to_string())
}

impl Profile {
    pub fn needs_onboarding(&self) -> bool { self.status == OnboardingStatus::Pending }
    /// Passer le questionnaire : valeurs par défaut conservées, jamais redemandé automatiquement.
    pub fn skip(&mut self) { self.status = OnboardingStatus::Skipped; }
    pub fn complete(&mut self) { self.status = OnboardingStatus::Completed; }
    /// Recommencer le questionnaire (menu Aide) sans perdre les réponses existantes.
    pub fn reopen(&mut self) { self.status = OnboardingStatus::Pending; }

    pub fn set_camera(&mut self, make: &str, model: &str) -> Result<(), ProfileError> {
        let (make, model) = (clean(make, "marque")?, clean(model, "modèle")?);
        self.camera = if make.is_empty() && model.is_empty() { None } else { Some(Camera { make, model }) };
        Ok(())
    }
    pub fn set_cache_limit_mb(&mut self, mb: u32) -> Result<(), ProfileError> {
        if !(256..=1_048_576).contains(&mb) { return Err(ProfileError::BadNumber("cache")); }
        self.storage.cache_limit_mb = mb;
        Ok(())
    }

    /// Mode proposé : moyenne (arrondie vers le bas) des deux niveaux.
    pub fn suggested_mode(&self) -> Mode {
        match (self.photo_level as u8 + self.retouch_level as u8) / 2 { 0 | 1 => if self.retouch_level == Skill::Beginner || self.photo_level == Skill::Beginner { Mode::Beginner } else { Mode::Intermediate }, 2 => Mode::Intermediate, _ => Mode::Expert }
    }
    /// Mode effectif : le choix explicite de l'utilisateur l'emporte toujours.
    pub fn mode(&self) -> Mode { self.mode_override.unwrap_or_else(|| self.suggested_mode()) }

    pub fn suggested_shortcut_profile(&self) -> ShortcutProfileId {
        for s in &self.previous_software {
            match s {
                LegacySoftware::Lightroom | LegacySoftware::PhotoMechanic => return ShortcutProfileId::LightroomLike,
                LegacySoftware::Photoshop => return ShortcutProfileId::PhotoshopLike,
                LegacySoftware::CaptureOne => return ShortcutProfileId::CaptureOneLike,
                _ => {}
            }
        }
        ShortcutProfileId::Norelias
    }

    pub fn lang(&self) -> Lang {
        match self.language {
            Language::Fr => Lang::Fr,
            Language::En => Lang::En,
            Language::System => system_lang(),
        }
    }
    /// Une aide quelconque est-elle active ? Faux ⇒ aucune infobulle, aucun tutoriel proactif.
    pub fn aids_enabled(&self) -> bool { self.coaching != Coaching::Off || self.tooltips }
    pub fn disable_all_aids(&mut self) { self.coaching = Coaching::Off; self.tooltips = false; }
}

fn system_lang() -> Lang {
    let v = std::env::var("LC_ALL").or_else(|_| std::env::var("LANG")).unwrap_or_default().to_lowercase();
    if v.starts_with("fr") { Lang::Fr } else { Lang::En }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_are_private_and_unobtrusive() {
        let p = Profile::default();
        assert!(p.needs_onboarding());
        assert!(!p.track_progress, "le suivi doit être désactivé par défaut");
        assert!(p.storage.auto_backup);
    }
    #[test]
    fn skipping_never_asks_again_but_can_reopen() {
        let mut p = Profile::default();
        p.skip();
        assert!(!p.needs_onboarding());
        p.reopen();
        assert!(p.needs_onboarding());
    }
    #[test]
    fn mode_suggestions() {
        let mut p = Profile::default();
        p.photo_level = Skill::Beginner; p.retouch_level = Skill::Beginner;
        assert_eq!(p.suggested_mode(), Mode::Beginner);
        p.photo_level = Skill::Professional; p.retouch_level = Skill::Beginner;
        assert_eq!(p.suggested_mode(), Mode::Beginner, "pro de la prise de vue mais débutant en retouche");
        p.photo_level = Skill::Advanced; p.retouch_level = Skill::Intermediate;
        assert_eq!(p.suggested_mode(), Mode::Intermediate);
        p.photo_level = Skill::Professional; p.retouch_level = Skill::Professional;
        assert_eq!(p.suggested_mode(), Mode::Expert);
        p.mode_override = Some(Mode::Beginner);
        assert_eq!(p.mode(), Mode::Beginner, "le choix de l'utilisateur prime");
    }
    #[test]
    fn camera_validation() {
        let mut p = Profile::default();
        p.set_camera("  Nikon ", "D5200").unwrap();
        assert_eq!(p.camera, Some(Camera { make: "Nikon".into(), model: "D5200".into() }));
        assert_eq!(p.set_camera("Ni\u{0}kon", "x"), Err(ProfileError::BadText("marque")));
        assert_eq!(p.set_camera(&"a".repeat(81), "x"), Err(ProfileError::TooLong("marque")));
        p.set_camera("", " ").unwrap();
        assert_eq!(p.camera, None);
        assert!(p.set_cache_limit_mb(10).is_err());
    }
    #[test]
    fn shortcut_suggestion_from_previous_software() {
        let mut p = Profile::default();
        assert_eq!(p.suggested_shortcut_profile(), ShortcutProfileId::Norelias);
        p.previous_software = vec![LegacySoftware::Evoto, LegacySoftware::CaptureOne];
        assert_eq!(p.suggested_shortcut_profile(), ShortcutProfileId::CaptureOneLike);
    }
    #[test]
    fn disabling_all_aids() {
        let mut p = Profile::default();
        assert!(p.aids_enabled());
        p.disable_all_aids();
        assert!(!p.aids_enabled());
    }
}
