//! Conditions d'utilisation et politique de confidentialité : acceptation **obligatoire** (version et date
//! enregistrées) ; consentement marketing **distinct, facultatif, décoché par défaut**, avec historique.
//! Les textes juridiques eux-mêmes ne sont pas rédigés ici : seuls leurs identifiants de version le sont.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LegalVersions { pub terms: String, pub privacy: String, /// Version du texte de la case marketing.
    pub marketing_text: String }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Acceptance { pub terms_version: String, pub privacy_version: String, pub accepted_at: u64 }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketingEvent { pub granted: bool, pub at: u64, pub text_version: String }

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Consents { pub acceptances: Vec<Acceptance>, pub marketing: Vec<MarketingEvent> }

#[derive(Debug, PartialEq, Eq)]
pub enum TermsError { TermsRequired, PrivacyRequired }

impl TermsError { pub fn key(&self) -> &'static str { match self { Self::TermsRequired => "terms.terms_required", Self::PrivacyRequired => "terms.privacy_required" } } }

impl Consents {
    /// Inscription : les deux acceptations sont requises ; le marketing est facultatif (faux si non fourni).
    pub fn at_signup(accept_terms: bool, accept_privacy: bool, marketing: bool, v: &LegalVersions, now: u64) -> Result<Self, TermsError> {
        if !accept_terms { return Err(TermsError::TermsRequired); }
        if !accept_privacy { return Err(TermsError::PrivacyRequired); }
        let mut c = Consents { acceptances: vec![Acceptance { terms_version: v.terms.clone(), privacy_version: v.privacy.clone(), accepted_at: now }], marketing: vec![] };
        c.set_marketing(marketing, v, now);
        Ok(c)
    }
    /// Enregistre chaque changement (jamais d'écrasement : l'historique sert de preuve).
    pub fn set_marketing(&mut self, granted: bool, v: &LegalVersions, now: u64) {
        if self.marketing_granted() != granted || self.marketing.is_empty() { self.marketing.push(MarketingEvent { granted, at: now, text_version: v.marketing_text.clone() }); }
    }
    pub fn marketing_granted(&self) -> bool { self.marketing.last().is_some_and(|e| e.granted) }
    pub fn needs_reacceptance(&self, v: &LegalVersions) -> bool { self.acceptances.last().is_none_or(|a| a.terms_version != v.terms || a.privacy_version != v.privacy) }
    pub fn accept_current(&mut self, v: &LegalVersions, now: u64) { self.acceptances.push(Acceptance { terms_version: v.terms.clone(), privacy_version: v.privacy.clone(), accepted_at: now }); }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn v(t: &str) -> LegalVersions { LegalVersions { terms: t.into(), privacy: "p1".into(), marketing_text: "m1".into() } }
    #[test]
    fn both_acceptances_are_mandatory() {
        assert_eq!(Consents::at_signup(false, true, false, &v("t1"), 0), Err(TermsError::TermsRequired));
        assert_eq!(Consents::at_signup(true, false, true, &v("t1"), 0), Err(TermsError::PrivacyRequired));
    }
    #[test]
    fn version_and_date_are_recorded_and_marketing_is_independent_and_off_by_default() {
        let c = Consents::at_signup(true, true, false, &v("2026-11"), 1234).unwrap();
        assert_eq!(c.acceptances[0], Acceptance { terms_version: "2026-11".into(), privacy_version: "p1".into(), accepted_at: 1234 });
        assert!(!c.marketing_granted(), "décoché par défaut");
        assert_eq!(c.marketing.len(), 1);
        let opt = Consents::at_signup(true, true, true, &v("t"), 5).unwrap();
        assert!(opt.marketing_granted() && opt.acceptances.len() == 1);
    }
    #[test]
    fn marketing_history_is_append_only() {
        let mut c = Consents::at_signup(true, true, true, &v("t"), 1).unwrap();
        c.set_marketing(true, &v("t"), 2);
        c.set_marketing(false, &v("t"), 3);
        c.set_marketing(true, &v("t"), 4);
        assert_eq!(c.marketing.iter().map(|e| (e.granted, e.at)).collect::<Vec<_>>(), vec![(true, 1), (false, 3), (true, 4)]);
    }
    #[test]
    fn new_version_requires_reacceptance() {
        let mut c = Consents::at_signup(true, true, false, &v("t1"), 1).unwrap();
        assert!(!c.needs_reacceptance(&v("t1")) && c.needs_reacceptance(&v("t2")));
        c.accept_current(&v("t2"), 9);
        assert!(!c.needs_reacceptance(&v("t2")) && c.acceptances.len() == 2, "l'ancienne acceptation est conservée");
    }
}
