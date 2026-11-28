//! Intégration des nouveaux utilisateurs de Norélias Studio Pro : questionnaire, profil local,
//! espaces de travail adaptatifs, raccourcis familiers, import de presets et confidentialité.
//!
//! Garanties de conception (toutes couvertes par des tests) :
//! * aucune donnée ne quitte l'ordinateur sans consentement explicite journalisé (`privacy`) ;
//! * aucun compte requis ; le profil est un fichier JSON local, modifiable à tout moment (`store`) ;
//! * un niveau d'utilisateur ne supprime jamais un outil : il change seulement sa visibilité (`workspace`) ;
//! * les outils non intégrés sont déclarés `Planned` et ne sont jamais présentés comme fonctionnels (`tools`).

pub mod compat;
pub mod privacy;
pub mod questionnaire;
pub mod profile;
pub mod shortcuts;
pub mod store;
pub mod tools;
pub mod workspace;

pub use profile::{Mode, Profile, Skill, Specialty};
