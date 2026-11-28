use norelias_account::{address::Address, signup::SignupData};
use norelias_i18n::units::Units;
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::PathBuf};

/// Profil « Mon Studio » : source unique des informations professionnelles. Tout est facultatif.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct StudioProfile {
    pub photographer_name: String,
    /// Faux = photographe indépendant sans studio commercial.
    pub has_studio: bool,
    pub studio_name: String,
    pub tagline: String,
    pub logo_path: Option<String>,
    pub copyright_holder: String,
    pub address: Address,
    pub phone_e164: Option<String>,
    pub website: String,
    pub socials: Vec<String>,
    pub tax_ids: Vec<(String, String)>,
    pub language: String,
    pub timezone: String,
    pub currency: String,
    pub units: Option<Units>,
}

impl StudioProfile {
    pub fn from_signup(d: &SignupData) -> Self {
        Self { photographer_name: d.display_name.clone(), has_studio: d.has_studio, studio_name: d.studio_name.clone(), tagline: d.tagline.clone(), logo_path: d.logo_path.clone(),
            copyright_holder: if d.studio_name.is_empty() { d.display_name.clone() } else { d.studio_name.clone() }, address: d.address.clone(), phone_e164: d.phone_e164.clone(),
            website: d.website.clone(), socials: d.socials.clone(), tax_ids: d.tax_ids.clone(), language: d.language.clone(), timezone: d.timezone.clone(), currency: d.currency.clone(),
            units: Some(if d.imperial_units { Units::Imperial } else { Units::Metric }) }
    }
    /// Nom à afficher comme filigrane par défaut : studio, sinon photographe.
    pub fn display_brand(&self) -> &str { if !self.studio_name.trim().is_empty() { &self.studio_name } else { &self.photographer_name } }
    pub fn holder(&self) -> &str { if !self.copyright_holder.trim().is_empty() { &self.copyright_holder } else { self.display_brand() } }
}

pub struct ProfileStore { dir: PathBuf }
impl ProfileStore {
    pub fn new(dir: impl Into<PathBuf>) -> Self { Self { dir: dir.into() } }
    pub fn save(&self, p: &StudioProfile) -> std::io::Result<()> { atomic(&self.dir, "studio.json", serde_json::to_string_pretty(p)?.as_bytes()) }
    pub fn load(&self) -> std::io::Result<StudioProfile> {
        match fs::read(self.dir.join("studio.json")) { Ok(b) => Ok(serde_json::from_slice(&b)?), Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(StudioProfile::default()), Err(e) => Err(e) }
    }
}
pub(crate) fn atomic(dir: &std::path::Path, name: &str, bytes: &[u8]) -> std::io::Result<()> {
    fs::create_dir_all(dir)?;
    let tmp = dir.join(format!("{name}.tmp"));
    { let mut f = fs::File::create(&tmp)?; f.write_all(bytes)?; f.sync_all()?; }
    fs::rename(tmp, dir.join(name))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn brand_falls_back_to_photographer() {
        let mut p = StudioProfile { photographer_name: "Élodie Côté".into(), ..Default::default() };
        assert_eq!(p.display_brand(), "Élodie Côté");
        p.studio_name = "Norélias Photography".into();
        assert_eq!((p.display_brand(), p.holder()), ("Norélias Photography", "Norélias Photography"));
        p.copyright_holder = "Élodie Côté".into();
        assert_eq!(p.holder(), "Élodie Côté");
    }
    #[test]
    fn store_roundtrip_with_unicode_and_missing_file() {
        let d = std::env::temp_dir().join(format!("norelias-studio-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        let s = ProfileStore::new(&d);
        assert_eq!(s.load().unwrap(), StudioProfile::default());
        let p = StudioProfile { studio_name: "Studio 日本語 Ñandú".into(), tagline: "Lumière & émotion".into(), ..Default::default() };
        s.save(&p).unwrap();
        assert_eq!(s.load().unwrap(), p);
        let _ = fs::remove_dir_all(&d);
    }
    #[test]
    fn from_signup_maps_fields() {
        let d = SignupData { display_name: "Jean".into(), has_studio: false, studio_name: "Jean".into(), tagline: "".into(), logo_path: None, address: Address::default(), phone_e164: Some("+15143987000".into()),
            website: "https://x.ca".into(), socials: vec![], tax_ids: vec![], level: None, specialties: vec![], language: "fr".into(), timezone: "America/Toronto".into(), currency: "CAD".into(), imperial_units: false };
        let p = StudioProfile::from_signup(&d);
        assert_eq!((p.units, p.holder(), p.phone_e164.as_deref()), (Some(Units::Metric), "Jean", Some("+15143987000")));
    }
}
