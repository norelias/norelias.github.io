//! Parcours d'inscription en 4 étapes (Compte · Studio · Expérience · Préférences).
//! Seuls courriel et mot de passe sont **obligatoires**, et seulement pour créer un compte ; tout le reste est facultatif.
//! « Utiliser sans compte » (mode local) ne demande rien.

use crate::address::{self, Address};
use crate::crypto::KdfCost;
use crate::record::{AccountError, AccountRecord};
use crate::{password, phone, validate, FieldError};
use norelias_onboarding::profile::{Mode, Specialty};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step { Account, Studio, Experience, Preferences }
impl Step {
    pub const ALL: [Step; 4] = [Step::Account, Step::Studio, Step::Experience, Step::Preferences];
    pub fn title_key(self) -> &'static str {
        match self { Step::Account => "signup.step.account", Step::Studio => "signup.step.studio", Step::Experience => "signup.step.experience", Step::Preferences => "signup.step.preferences" }
    }
}

#[derive(Clone, Debug, Default)]
pub struct SignupDraft {
    pub first_name: String, pub last_name: String, pub email: String, pub password: String, pub password_confirm: String,
    /// Faux = photographe indépendant sans studio commercial.
    pub has_studio: bool,
    pub studio_name: String, pub tagline: String, pub logo_path: Option<String>,
    pub address: Address, pub phone: String, pub website: String,
    pub socials: Vec<String>, pub tax_ids: Vec<(String, String)>,
    pub level: Option<Mode>, pub specialties: Vec<Specialty>,
    pub language: String, pub timezone: String, pub currency: String, pub imperial_units: bool,
}

/// Données validées et normalisées issues de l'inscription (consommées par le profil « Mon Studio »).
#[derive(Clone, Debug, PartialEq)]
pub struct SignupData {
    pub display_name: String, pub has_studio: bool, pub studio_name: String, pub tagline: String, pub logo_path: Option<String>,
    pub address: Address, pub phone_e164: Option<String>, pub website: String, pub socials: Vec<String>, pub tax_ids: Vec<(String, String)>,
    pub level: Option<Mode>, pub specialties: Vec<Specialty>,
    pub language: String, pub timezone: String, pub currency: String, pub imperial_units: bool,
}

fn e(field: &'static str, key: &'static str) -> FieldError { FieldError { field, key } }

impl SignupDraft {
    fn identity(&self) -> Vec<String> { vec![self.email.clone(), self.first_name.clone(), self.last_name.clone(), self.studio_name.clone()] }

    pub fn validate_step(&self, step: Step) -> Vec<FieldError> {
        let mut v = vec![];
        match step {
            Step::Account => {
                for (f, s) in [("first_name", &self.first_name), ("last_name", &self.last_name)] { if !validate::text(s, 80) { v.push(e(f, "error.too_long")); } }
                if self.email.trim().is_empty() { v.push(e("email", "error.required")); } else if !validate::email(&self.email) { v.push(e("email", "error.email.invalid")); }
                let id = self.identity();
                let idr: Vec<&str> = id.iter().map(String::as_str).collect();
                if self.password.is_empty() { v.push(e("password", "error.required")); } else if !password::strength(&self.password, &idr).acceptable { v.push(e("password", "error.password.weak")); }
                if self.password != self.password_confirm { v.push(e("password_confirm", "error.password.mismatch")); }
            }
            Step::Studio => {
                if !validate::text(&self.studio_name, 120) { v.push(e("studio_name", "error.too_long")); }
                if !validate::text(&self.tagline, 160) { v.push(e("tagline", "error.too_long")); }
                v.extend(address::validate(&self.address));
                if !self.phone.trim().is_empty() && phone::parse(&self.phone, Some(&self.address.country)).is_err() { v.push(e("phone", "error.phone.invalid")); }
                if !self.website.trim().is_empty() && !validate::web_url(&self.website) { v.push(e("website", "error.url.invalid")); }
                if self.socials.iter().any(|s| !s.trim().is_empty() && !validate::web_url(s)) { v.push(e("socials", "error.url.invalid")); }
                if self.tax_ids.iter().any(|(k, val)| !validate::text(k, 40) || !validate::text(val, 60)) { v.push(e("tax_ids", "error.too_long")); }
            }
            Step::Experience => {}
            Step::Preferences => {
                if !self.timezone.is_empty() && !norelias_i18n::format::timezone_well_formed(&self.timezone) { v.push(e("timezone", "error.required")); }
                if !self.currency.is_empty() && !(self.currency.len() == 3 && self.currency.chars().all(|c| c.is_ascii_uppercase())) { v.push(e("currency", "error.required")); }
                if !self.language.is_empty() && norelias_i18n::Locale::parse(&self.language).is_none() { v.push(e("language", "error.required")); }
            }
        }
        v
    }
    pub fn validate_all(&self) -> Vec<FieldError> { Step::ALL.iter().flat_map(|s| self.validate_step(*s)).collect() }

    /// Données normalisées, sans création de compte (mode local). Les champs vides restent vides.
    pub fn data(&self) -> Result<SignupData, Vec<FieldError>> {
        let mut errs = vec![];
        for s in [Step::Studio, Step::Experience, Step::Preferences] { errs.extend(self.validate_step(s)); }
        if !errs.is_empty() { return Err(errs); }
        let phone_e164 = if self.phone.trim().is_empty() { None } else { phone::parse(&self.phone, Some(&self.address.country)).ok().map(|p| p.e164) };
        let display = [self.first_name.trim(), self.last_name.trim()].iter().filter(|s| !s.is_empty()).cloned().collect::<Vec<_>>().join(" ");
        Ok(SignupData { display_name: display.clone(), has_studio: self.has_studio,
            // Photographe indépendant : le nom du photographe sert de nom commercial par défaut.
            studio_name: if self.studio_name.trim().is_empty() && !self.has_studio { display } else { self.studio_name.trim().into() },
            tagline: self.tagline.trim().into(), logo_path: self.logo_path.clone(),
            address: address::normalize(&self.address), phone_e164, website: self.website.trim().into(),
            socials: self.socials.iter().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(), tax_ids: self.tax_ids.clone(),
            level: self.level, specialties: self.specialties.clone(), language: self.language.clone(), timezone: self.timezone.clone(), currency: self.currency.clone(), imperial_units: self.imperial_units })
    }

    /// Création du compte local + données. Retourne les codes de récupération (affichés une fois).
    pub fn finish(&self, cost: KdfCost, now: u64) -> Result<(AccountRecord, Vec<String>, SignupData), Vec<FieldError>> {
        let errs = self.validate_all();
        if !errs.is_empty() { return Err(errs); }
        let data = self.data()?;
        let (rec, codes) = AccountRecord::create(&self.email, &self.password, &data.display_name, cost, now).map_err(|er| match er {
            AccountError::InvalidEmail => vec![e("email", "error.email.invalid")], _ => vec![e("password", "error.password.weak")] })?;
        Ok((rec, codes, data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn base() -> SignupDraft { SignupDraft { email: "jean@exemple.ca".into(), password: "Correct-Horse-Battery-42".into(), password_confirm: "Correct-Horse-Battery-42".into(), ..Default::default() } }
    fn keys(v: &[FieldError]) -> Vec<(&str, &str)> { v.iter().map(|x| (x.field, x.key)).collect() }

    #[test]
    fn only_email_and_password_are_required() {
        let d = base();
        assert!(d.validate_all().is_empty(), "aucune information personnelle superflue n'est exigée");
        assert!(d.finish(KdfCost::TESTING, 0).is_ok());
    }
    #[test]
    fn account_step_errors() {
        let mut d = SignupDraft::default();
        assert_eq!(keys(&d.validate_step(Step::Account)), vec![("email", "error.required"), ("password", "error.required")]);
        d.email = "x".into(); d.password = "password".into(); d.password_confirm = "autre".into();
        assert_eq!(keys(&d.validate_step(Step::Account)), vec![("email", "error.email.invalid"), ("password", "error.password.weak"), ("password_confirm", "error.password.mismatch")]);
    }
    #[test]
    fn password_may_not_contain_identity() {
        let mut d = base();
        d.first_name = "Jean".into(); d.last_name = "Tremblay".into();
        d.password = "JeanTremblay-2026".into(); d.password_confirm = d.password.clone();
        assert!(keys(&d.validate_step(Step::Account)).contains(&("password", "error.password.weak")));
    }
    #[test]
    fn studio_step_validates_only_what_is_filled() {
        let mut d = base();
        d.phone = "514 398 7000".into(); d.address = Address { country: "CA".into(), postal_code: "H2Y 1C6".into(), region: "QC".into(), ..Default::default() };
        d.website = "https://norelias.ca".into();
        assert!(d.validate_step(Step::Studio).is_empty());
        d.phone = "12".into(); d.website = "norelias".into(); d.address.postal_code = "99999".into(); d.socials = vec!["javascript:x".into()];
        let errs = d.validate_step(Step::Studio);
        let k = keys(&errs);
        for want in [("phone", "error.phone.invalid"), ("website", "error.url.invalid"), ("postal_code", "error.postal.invalid"), ("socials", "error.url.invalid")] { assert!(k.contains(&want), "{want:?} dans {k:?}"); }
    }
    #[test]
    fn independent_photographer_uses_own_name() {
        let mut d = base();
        d.first_name = "Élodie".into(); d.last_name = "Côté".into(); d.has_studio = false;
        let data = d.data().unwrap();
        assert_eq!((data.studio_name.as_str(), data.has_studio), ("Élodie Côté", false));
        d.has_studio = true; d.studio_name = "Norélias Photography".into();
        assert_eq!(d.data().unwrap().studio_name, "Norélias Photography");
    }
    #[test]
    fn phone_is_stored_as_e164_and_address_normalised() {
        let mut d = base();
        d.phone = "(514) 398-7000".into();
        d.address = Address { country: "ca".into(), postal_code: "h2y1c6".into(), region: "qc".into(), ..Default::default() };
        let data = d.data().unwrap();
        assert_eq!(data.phone_e164.as_deref(), Some("+15143987000"));
        assert_eq!(data.address.postal_code, "H2Y 1C6");
    }
    #[test]
    fn local_only_needs_no_account() {
        let d = SignupDraft::default(); // rien de rempli
        assert!(d.data().is_ok(), "le mode local ne demande aucune donnée");
    }
    #[test]
    fn preferences_validation() {
        let mut d = base();
        d.timezone = "../etc".into(); d.currency = "cad".into(); d.language = "1".into();
        assert_eq!(d.validate_step(Step::Preferences).len(), 3);
        d.timezone = "America/Toronto".into(); d.currency = "CAD".into(); d.language = "fr-CA".into();
        assert!(d.validate_step(Step::Preferences).is_empty());
    }
}
