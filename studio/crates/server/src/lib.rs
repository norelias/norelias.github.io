//! Logique **côté serveur** de l'authentification de Norélias Studio Pro : vérification obligatoire du courriel,
//! codes à 6 chiffres, protection anti-robot, courriels transactionnels, conditions d'utilisation, récupération.
//!
//! Ce crate est la logique pure, testée. Il ne contient **aucun client réseau** : l'envoi réel de courriels et la
//! vérification CAPTCHA passent par les interfaces `EmailSender`, `CaptchaVerifier` et `HttpTransport`, que le
//! binaire serveur de production devra implémenter avec un vrai fournisseur et de vraies clés.
//!
//! **Rien de simulé en production** : `Server::new` refuse tout service marqué simulé quand l'environnement
//! est `Production` (testé). Ce qui reste à brancher est listé dans `ARCHITECTURE.md` §9.

pub mod captcha;
pub mod code;
pub mod dns;
pub mod env;
pub mod mail;
pub mod service;
pub mod templates;
pub mod terms;

/// Erreur d'API : clé i18n traduisible + délai éventuel avant de réessayer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiError { pub key: &'static str, pub retry_after: Option<u64> }
impl ApiError { pub fn new(key: &'static str) -> Self { Self { key, retry_after: None } } pub fn after(key: &'static str, s: u64) -> Self { Self { key, retry_after: Some(s) } } }
