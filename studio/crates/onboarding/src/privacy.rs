//! Consentements explicites. **Par défaut, rien n'est autorisé.**
//! Tout code qui enverrait une photographie ou des statistiques d'usage hors de l'ordinateur
//! doit d'abord appeler `ConsentLedger::require`.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Purpose {
    /// Envoi de photographies à un service d'IA distant.
    CloudAi,
    /// Téléchargement de modèles IA (aucune photo envoyée).
    ModelDownload,
    /// Hébergement de galeries clients.
    GalleryHosting,
    /// Statistiques d'usage anonymisées.
    Telemetry,
    /// Rapports de plantage.
    CrashReports,
}

impl Purpose {
    /// Ce but implique-t-il la sortie de photographies de l'ordinateur ?
    pub fn sends_photos(self) -> bool { matches!(self, Purpose::CloudAi | Purpose::GalleryHosting) }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Consent {
    pub purpose: Purpose,
    pub provider: String,
    pub paid: bool,
    pub cost_acknowledged: bool,
    pub granted_at: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ConsentError {
    NotGranted(Purpose),
    ProviderRequired,
    CostNotAcknowledged,
}
impl std::fmt::Display for ConsentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotGranted(p) => write!(f, "consentement non accordé pour {p:?}"),
            Self::ProviderRequired => write!(f, "le fournisseur doit être nommé"),
            Self::CostNotAcknowledged => write!(f, "ce fournisseur est payant : le coût doit être confirmé avant utilisation"),
        }
    }
}
impl std::error::Error for ConsentError {}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsentLedger { entries: Vec<Consent> }

impl ConsentLedger {
    pub fn is_granted(&self, p: Purpose) -> bool { self.entries.iter().any(|c| c.purpose == p) }

    /// Accorde un consentement (action explicite de l'utilisateur). Un service externe doit être nommé ;
    /// un service payant exige la confirmation préalable du coût.
    pub fn grant(&mut self, purpose: Purpose, provider: &str, paid: bool, cost_acknowledged: bool, now: u64) -> Result<(), ConsentError> {
        let provider = provider.trim();
        if provider.is_empty() { return Err(ConsentError::ProviderRequired); }
        if paid && !cost_acknowledged { return Err(ConsentError::CostNotAcknowledged); }
        self.revoke(purpose);
        self.entries.push(Consent { purpose, provider: provider.to_string(), paid, cost_acknowledged, granted_at: now });
        Ok(())
    }
    pub fn revoke(&mut self, purpose: Purpose) { self.entries.retain(|c| c.purpose != purpose); }
    pub fn revoke_all(&mut self) { self.entries.clear(); }

    /// Porte d'entrée obligatoire avant toute sortie de données.
    pub fn require(&self, purpose: Purpose) -> Result<&Consent, ConsentError> {
        self.entries.iter().find(|c| c.purpose == purpose).ok_or(ConsentError::NotGranted(purpose))
    }
    pub fn entries(&self) -> &[Consent] { &self.entries }
}

#[cfg(test)]
mod tests {
    use super::*;
    const ALL: [Purpose; 5] = [Purpose::CloudAi, Purpose::ModelDownload, Purpose::GalleryHosting, Purpose::Telemetry, Purpose::CrashReports];

    #[test]
    fn nothing_is_allowed_by_default() {
        let l = ConsentLedger::default();
        for p in ALL { assert_eq!(l.require(p), Err(ConsentError::NotGranted(p))); }
    }
    #[test]
    fn grant_is_scoped_to_one_purpose_and_revocable() {
        let mut l = ConsentLedger::default();
        l.grant(Purpose::ModelDownload, "modèles Norélias", false, false, 1).unwrap();
        assert!(l.require(Purpose::ModelDownload).is_ok());
        assert!(l.require(Purpose::CloudAi).is_err(), "un consentement ne s'étend pas aux autres usages");
        l.revoke(Purpose::ModelDownload);
        assert!(l.require(Purpose::ModelDownload).is_err());
    }
    #[test]
    fn paid_provider_requires_cost_acknowledgement() {
        let mut l = ConsentLedger::default();
        assert_eq!(l.grant(Purpose::CloudAi, "Fournisseur X", true, false, 1), Err(ConsentError::CostNotAcknowledged));
        assert!(!l.is_granted(Purpose::CloudAi));
        l.grant(Purpose::CloudAi, "Fournisseur X", true, true, 2).unwrap();
        assert!(l.is_granted(Purpose::CloudAi));
    }
    #[test]
    fn provider_must_be_named() {
        assert_eq!(ConsentLedger::default().grant(Purpose::Telemetry, "  ", false, false, 0), Err(ConsentError::ProviderRequired));
    }
    #[test]
    fn photo_sending_purposes_are_identified() {
        assert!(Purpose::CloudAi.sends_photos() && Purpose::GalleryHosting.sends_photos());
        assert!(!Purpose::Telemetry.sends_photos() && !Purpose::ModelDownload.sends_photos());
    }
    #[test]
    fn ledger_persists_through_store() {
        let d = std::env::temp_dir().join(format!("norelias-ledger-{}", std::process::id()));
        let s = crate::store::Store::new(&d);
        let mut l = ConsentLedger::default();
        l.grant(Purpose::Telemetry, "Norélias", false, false, 5).unwrap();
        s.save_ledger(&l).unwrap();
        assert_eq!(s.load_ledger().unwrap(), l);
        let _ = s.erase_all();
    }
}
