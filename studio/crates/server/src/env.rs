//! Environnement d'exécution et garde « rien de simulé en production ».

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Environment { Production, Development }

#[derive(Debug, PartialEq, Eq)]
pub enum ConfigError { SimulatedEmailInProduction, SimulatedCaptchaInProduction, WeakSecret, InsecureBaseUrl }

/// Valide la configuration. En production : services réels uniquement, secret serveur de ≥ 32 octets non nul, URL HTTPS.
pub fn check(env: Environment, email_simulated: bool, captcha_simulated: bool, secret: &[u8], base_url: &str) -> Result<(), ConfigError> {
    if secret.len() < 32 || secret.iter().all(|b| *b == 0) { return Err(ConfigError::WeakSecret); }
    if env == Environment::Production {
        if email_simulated { return Err(ConfigError::SimulatedEmailInProduction); }
        if captcha_simulated { return Err(ConfigError::SimulatedCaptchaInProduction); }
        if !base_url.starts_with("https://") { return Err(ConfigError::InsecureBaseUrl); }
    }
    Ok(())
}
