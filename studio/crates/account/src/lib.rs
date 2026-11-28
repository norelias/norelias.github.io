//! Compte, authentification et profil de contact de Norélias Studio Pro — **couche locale**.
//!
//! Ce qui est réel ici : robustesse des mots de passe, hachage Argon2id, TOTP (RFC 6238), codes de
//! récupération, limitation des tentatives, sessions et révocation d'appareils, validation des
//! courriels / téléphones (E.164) / adresses par pays, parcours d'inscription en 4 étapes, export et
//! suppression des données personnelles.
//!
//! Ce qui n'existe PAS (et n'est jamais simulé) : le service en ligne. La vérification du courriel,
//! la récupération par courriel et la synchronisation exigent un serveur ; elles sont décrites dans
//! ARCHITECTURE.md (contrat `AuthBackend`) et signalées « non disponibles » dans l'interface.
//! Le mode local fonctionne entièrement hors connexion.

pub mod address;
pub mod crypto;
pub mod limiter;
pub mod password;
pub mod phone;
pub mod record;
pub mod session;
pub mod signup;
pub mod totp;
pub mod validate;

/// Erreur de champ : `field` identifie le champ, `key` une clé du catalogue i18n.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldError { pub field: &'static str, pub key: &'static str }
