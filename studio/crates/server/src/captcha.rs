//! Protection anti-robot **réelle** : vérification côté serveur du jeton auprès du fournisseur (Cloudflare Turnstile,
//! hCaptcha, Google reCAPTCHA v2/v3). Jamais une simple case à cocher : sans jeton validé par le fournisseur, la requête est refusée.
//! Échec fermé : si le fournisseur est injoignable, l'inscription est refusée (message traduit + solution de secours).
//!
//! Adresses et champs d'après la documentation publique des fournisseurs (rappel, **non vérifiés en conditions réelles**).

use crate::mail::{HttpRequest, HttpTransport};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Provider { Turnstile, HCaptcha, ReCaptchaV2, ReCaptchaV3 { min_score: f32 } }

impl Provider {
    pub fn verify_url(self) -> &'static str {
        match self { Provider::Turnstile => "https://challenges.cloudflare.com/turnstile/v0/siteverify", Provider::HCaptcha => "https://api.hcaptcha.com/siteverify",
            Provider::ReCaptchaV2 | Provider::ReCaptchaV3 { .. } => "https://www.google.com/recaptcha/api/siteverify" }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CaptchaError { MissingToken, MalformedToken, Rejected(Vec<String>), ScoreTooLow, HostnameMismatch, BadResponse, Unavailable }

impl CaptchaError {
    /// Clé i18n du message affiché à l'utilisateur.
    pub fn key(&self) -> &'static str { match self { Self::MissingToken => "captcha.required", Self::Unavailable => "captcha.unavailable", _ => "captcha.failed" } }
}

pub trait CaptchaVerifier {
    fn verify(&self, token: &str, remote_ip: Option<&str>) -> Result<(), CaptchaError>;
    fn is_simulated(&self) -> bool;
}

pub fn check_token_shape(t: &str) -> Result<(), CaptchaError> {
    if t.trim().is_empty() { return Err(CaptchaError::MissingToken); }
    if t.len() > 4096 || t.chars().any(|c| c.is_control() || c.is_whitespace()) { return Err(CaptchaError::MalformedToken); }
    Ok(())
}

fn urlenc(s: &str) -> String { s.bytes().map(|b| if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) { (b as char).to_string() } else { format!("%{b:02X}") }).collect() }

pub fn build_request(p: Provider, secret: &str, token: &str, remote_ip: Option<&str>) -> HttpRequest {
    let mut body = format!("secret={}&response={}", urlenc(secret), urlenc(token));
    if let Some(ip) = remote_ip { body += &format!("&remoteip={}", urlenc(ip)); }
    HttpRequest { url: p.verify_url().into(), headers: vec![], body, content_type: "application/x-www-form-urlencoded" }
}

pub fn parse_response(p: Provider, body: &str, expected_hostname: Option<&str>) -> Result<(), CaptchaError> {
    let v: serde_json::Value = serde_json::from_str(body).map_err(|_| CaptchaError::BadResponse)?;
    let success = v.get("success").and_then(|s| s.as_bool()).ok_or(CaptchaError::BadResponse)?;
    if !success {
        let codes = v.get("error-codes").and_then(|c| c.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
        return Err(CaptchaError::Rejected(codes));
    }
    if let (Some(exp), Some(h)) = (expected_hostname, v.get("hostname").and_then(|h| h.as_str())) { if !h.eq_ignore_ascii_case(exp) { return Err(CaptchaError::HostnameMismatch); } }
    if let Provider::ReCaptchaV3 { min_score } = p {
        let score = v.get("score").and_then(|s| s.as_f64()).ok_or(CaptchaError::BadResponse)? as f32;
        if score < min_score { return Err(CaptchaError::ScoreTooLow); }
    }
    Ok(())
}

/// Vérificateur réel. Nécessite une clé secrète (jamais dans le dépôt ni le client) et un transport HTTP.
pub struct RealCaptcha<T: HttpTransport> { pub provider: Provider, pub secret: String, pub expected_hostname: Option<String>, pub transport: T }
impl<T: HttpTransport> CaptchaVerifier for RealCaptcha<T> {
    fn verify(&self, token: &str, ip: Option<&str>) -> Result<(), CaptchaError> {
        check_token_shape(token)?;
        let r = self.transport.post(&build_request(self.provider, &self.secret, token, ip)).map_err(|_| CaptchaError::Unavailable)?;
        if r.status != 200 { return Err(CaptchaError::Unavailable); }
        parse_response(self.provider, &r.body, self.expected_hostname.as_deref())
    }
    fn is_simulated(&self) -> bool { false }
}

/// Vérificateur **fictif** (tests/développement) : accepte uniquement des jetons « ok-… ». Refusé en production.
pub struct ScriptedCaptcha { pub calls: std::cell::Cell<u32> }
impl Default for ScriptedCaptcha { fn default() -> Self { Self { calls: std::cell::Cell::new(0) } } }
impl CaptchaVerifier for ScriptedCaptcha {
    fn verify(&self, token: &str, _: Option<&str>) -> Result<(), CaptchaError> {
        self.calls.set(self.calls.get() + 1);
        check_token_shape(token)?;
        if token.starts_with("ok-") { Ok(()) } else { Err(CaptchaError::Rejected(vec!["invalid-input-response".into()])) }
    }
    fn is_simulated(&self) -> bool { true }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mail::{HttpResponse, TransportError};
    use std::cell::RefCell;

    #[test]
    fn token_shape_is_checked_before_any_network_call() {
        assert_eq!(check_token_shape(""), Err(CaptchaError::MissingToken));
        assert_eq!(check_token_shape("  "), Err(CaptchaError::MissingToken));
        assert_eq!(check_token_shape("a b"), Err(CaptchaError::MalformedToken));
        assert_eq!(check_token_shape(&"x".repeat(5000)), Err(CaptchaError::MalformedToken));
        assert!(check_token_shape("0.AbC-_123").is_ok());
    }
    #[test]
    fn requests_target_each_provider_and_escape_values() {
        for (p, host) in [(Provider::Turnstile, "challenges.cloudflare.com"), (Provider::HCaptcha, "hcaptcha.com"), (Provider::ReCaptchaV2, "google.com/recaptcha")] {
            assert!(build_request(p, "S", "T", None).url.contains(host));
        }
        let r = build_request(Provider::Turnstile, "se&cret", "to=ken", Some("203.0.113.9"));
        assert_eq!(r.body, "secret=se%26cret&response=to%3Dken&remoteip=203.0.113.9");
        assert_eq!(r.content_type, "application/x-www-form-urlencoded");
    }
    #[test]
    fn responses_are_parsed_strictly() {
        let ok = r#"{"success":true,"hostname":"norelias.ca","challenge_ts":"2026-11-28T10:00:00Z","error-codes":[]}"#;
        assert!(parse_response(Provider::Turnstile, ok, Some("norelias.ca")).is_ok());
        assert_eq!(parse_response(Provider::Turnstile, ok, Some("evil.example")), Err(CaptchaError::HostnameMismatch), "jeton émis pour un autre site");
        assert_eq!(parse_response(Provider::HCaptcha, r#"{"success":false,"error-codes":["timeout-or-duplicate"]}"#, None), Err(CaptchaError::Rejected(vec!["timeout-or-duplicate".into()])));
        for bad in ["", "<html>", "{}", r#"{"success":"yes"}"#] { assert_eq!(parse_response(Provider::Turnstile, bad, None), Err(CaptchaError::BadResponse), "{bad:?}"); }
        let v3 = Provider::ReCaptchaV3 { min_score: 0.5 };
        assert!(parse_response(v3, r#"{"success":true,"score":0.9}"#, None).is_ok());
        assert_eq!(parse_response(v3, r#"{"success":true,"score":0.1}"#, None), Err(CaptchaError::ScoreTooLow));
        assert_eq!(parse_response(v3, r#"{"success":true}"#, None), Err(CaptchaError::BadResponse), "score absent : refus");
    }
    struct T(RefCell<Vec<Result<HttpResponse, TransportError>>>, RefCell<u32>);
    impl HttpTransport for T { fn post(&self, _: &HttpRequest) -> Result<HttpResponse, TransportError> { *self.1.borrow_mut() += 1; self.0.borrow_mut().remove(0) } }
    fn real(r: Result<HttpResponse, TransportError>) -> RealCaptcha<T> { RealCaptcha { provider: Provider::Turnstile, secret: "s".into(), expected_hostname: None, transport: T(RefCell::new(vec![r]), RefCell::new(0)) } }

    #[test]
    fn real_verifier_fails_closed() {
        assert!(real(Ok(HttpResponse { status: 200, body: r#"{"success":true}"#.into() })).verify("tok", None).is_ok());
        assert_eq!(real(Ok(HttpResponse { status: 200, body: r#"{"success":false,"error-codes":["invalid-input-response"]}"#.into() })).verify("tok", None).unwrap_err().key(), "captcha.failed");
        let down = real(Err(TransportError("timeout".into())));
        assert_eq!(down.verify("tok", None), Err(CaptchaError::Unavailable), "fournisseur injoignable : refus, jamais d'acceptation par défaut");
        assert_eq!(real(Ok(HttpResponse { status: 500, body: "".into() })).verify("tok", None), Err(CaptchaError::Unavailable));
        let none = real(Ok(HttpResponse { status: 200, body: r#"{"success":true}"#.into() }));
        assert_eq!(none.verify("", None).unwrap_err().key(), "captcha.required");
        assert_eq!(*none.transport.1.borrow(), 0, "aucune requête sans jeton");
    }
    #[test]
    fn scripted_verifier_is_marked_simulated() {
        let s = ScriptedCaptcha::default();
        assert!(s.is_simulated() && s.verify("ok-1", None).is_ok() && s.verify("nope", None).is_err());
    }
}
