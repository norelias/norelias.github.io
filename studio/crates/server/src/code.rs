//! Défis de vérification : code numérique à 6 chiffres, expiration 10 min, renvoi après 60 s, anciens codes invalidés,
//! tentatives et renvois limités. Le code n'est jamais stocké : seul un HMAC-SHA256 avec un secret du serveur l'est.

use hmac::{Hmac, KeyInit, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

pub const TTL_SECONDS: u64 = 10 * 60;
pub const RESEND_COOLDOWN: u64 = 60;
pub const MAX_ATTEMPTS: u32 = 5;
pub const MAX_SENDS_PER_HOUR: usize = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Purpose { Signup, ChangeEmail, Recovery }

/// Code à 6 chiffres uniforme (échantillonnage par rejet, sans biais de modulo), zéros initiaux conservés.
pub fn generate_code() -> String {
    loop {
        let mut b = [0u8; 4];
        getrandom::fill(&mut b).expect("aléa du système");
        let v = u32::from_le_bytes(b);
        if v < 4_000_000_000 { return format!("{:06}", v % 1_000_000); }
    }
}

fn mac(secret: &[u8], purpose: Purpose, email: &str, code: &str) -> String {
    let mut m = <Hmac<Sha256> as KeyInit>::new_from_slice(secret).expect("clé");
    m.update(format!("{purpose:?}|{}|{code}", email.to_lowercase()).as_bytes());
    norelias_account::crypto::hex(&m.finalize().into_bytes())
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Challenge {
    pub purpose: Purpose,
    /// Adresse à laquelle le code a été envoyé.
    pub email: String,
    code_mac: String,
    pub issued_at: u64,
    pub expires_at: u64,
    pub attempts: u32,
    /// Horodatages des envois dans la dernière heure (survit au remplacement du code).
    sends: Vec<u64>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum IssueError { Cooldown(u64), HourlyLimit(u64) }
#[derive(Debug, PartialEq, Eq)]
pub enum VerifyResult { Ok, Wrong { attempts_left: u32 }, Expired, TooManyAttempts, Malformed }

impl Challenge {
    /// Émet un code. Remplace (donc invalide) tout code précédent pour le même but, en respectant le délai de 60 s
    /// et la limite horaire, qui persistent d'un envoi à l'autre. Retourne le défi et le code **en clair, à envoyer une seule fois**.
    pub fn issue(prev: Option<&Challenge>, purpose: Purpose, email: &str, now: u64, secret: &[u8]) -> Result<(Challenge, String), IssueError> {
        let mut sends: Vec<u64> = prev.map(|p| p.sends.clone()).unwrap_or_default();
        sends.retain(|t| now.saturating_sub(*t) < 3600);
        if let Some(last) = sends.last() { if now.saturating_sub(*last) < RESEND_COOLDOWN { return Err(IssueError::Cooldown(RESEND_COOLDOWN - (now - last))); } }
        if sends.len() >= MAX_SENDS_PER_HOUR { return Err(IssueError::HourlyLimit(3600 - (now - sends[0]))); }
        sends.push(now);
        let code = generate_code();
        Ok((Challenge { purpose, email: email.to_lowercase(), code_mac: mac(secret, purpose, email, &code), issued_at: now, expires_at: now + TTL_SECONDS, attempts: 0, sends }, code))
    }

    pub fn verify(&mut self, input: &str, now: u64, secret: &[u8]) -> VerifyResult {
        let input = input.trim().replace([' ', '-'], "");
        if input.len() != 6 || !input.chars().all(|c| c.is_ascii_digit()) { return VerifyResult::Malformed; }
        if self.attempts >= MAX_ATTEMPTS { return VerifyResult::TooManyAttempts; }
        if now >= self.expires_at { return VerifyResult::Expired; }
        self.attempts += 1;
        if norelias_account::crypto::ct_eq(mac(secret, self.purpose, &self.email, &input).as_bytes(), self.code_mac.as_bytes()) { VerifyResult::Ok }
        else { VerifyResult::Wrong { attempts_left: MAX_ATTEMPTS - self.attempts } }
    }
    pub fn seconds_until_resend(&self, now: u64) -> u64 { self.sends.last().map_or(0, |l| RESEND_COOLDOWN.saturating_sub(now.saturating_sub(*l))) }
}

#[cfg(test)]
mod tests {
    use super::*;
    const K: &[u8] = b"0123456789abcdef0123456789abcdef";

    #[test]
    fn codes_are_six_digits_and_roughly_uniform() {
        let mut counts = [0u32; 10];
        for _ in 0..3000 { let c = generate_code(); assert_eq!(c.len(), 6); assert!(c.chars().all(|d| d.is_ascii_digit())); counts[(c.as_bytes()[0] - b'0') as usize] += 1; }
        assert!(counts.iter().all(|&n| (200..=420).contains(&n)), "premier chiffre non uniforme : {counts:?}");
        assert!((0..400).any(|_| generate_code().starts_with('0')), "les zéros initiaux existent et sont conservés");
    }
    #[test]
    fn valid_code_verifies_and_formatting_is_tolerated() {
        let (mut c, code) = Challenge::issue(None, Purpose::Signup, "A@B.co", 0, K).unwrap();
        assert_eq!(c.verify(&format!("{} {}", &code[..3], &code[3..]), 5, K), VerifyResult::Ok);
    }
    #[test]
    fn expires_after_ten_minutes_exactly() {
        let (mut c, code) = Challenge::issue(None, Purpose::Signup, "a@b.co", 100, K).unwrap();
        let mut c2 = c.clone();
        assert_eq!(c.verify(&code, 100 + TTL_SECONDS - 1, K), VerifyResult::Ok);
        assert_eq!(c2.verify(&code, 100 + TTL_SECONDS, K), VerifyResult::Expired);
    }
    #[test]
    fn attempts_are_limited_and_correct_code_is_then_refused() {
        let (mut c, code) = Challenge::issue(None, Purpose::Signup, "a@b.co", 0, K).unwrap();
        let wrong = if code == "000000" { "111111" } else { "000000" };
        for i in 1..=MAX_ATTEMPTS { assert_eq!(c.verify(wrong, 1, K), VerifyResult::Wrong { attempts_left: MAX_ATTEMPTS - i }); }
        assert_eq!(c.verify(&code, 2, K), VerifyResult::TooManyAttempts, "force brute impossible : 5 essais par code");
        assert_eq!(c.verify("12ab56", 2, K), VerifyResult::Malformed);
    }
    #[test]
    fn resend_cooldown_invalidation_and_hourly_cap() {
        let (c1, code1) = Challenge::issue(None, Purpose::Signup, "a@b.co", 0, K).unwrap();
        assert_eq!(c1.seconds_until_resend(10), 50);
        assert_eq!(Challenge::issue(Some(&c1), Purpose::Signup, "a@b.co", 59, K).unwrap_err(), IssueError::Cooldown(1));
        let (mut c2, code2) = Challenge::issue(Some(&c1), Purpose::Signup, "a@b.co", 60, K).unwrap();
        if code1 != code2 { assert_eq!(c2.clone().verify(&code1, 61, K), VerifyResult::Wrong { attempts_left: 4 }, "l'ancien code est invalidé"); }
        assert_eq!(c2.verify(&code2, 61, K), VerifyResult::Ok);
        let mut cur = c2; let mut t = 60;
        for _ in 0..MAX_SENDS_PER_HOUR - 2 { t += 61; cur = Challenge::issue(Some(&cur), Purpose::Signup, "a@b.co", t, K).unwrap().0; }
        assert!(matches!(Challenge::issue(Some(&cur), Purpose::Signup, "a@b.co", t + 61, K), Err(IssueError::HourlyLimit(_))));
        assert!(Challenge::issue(Some(&cur), Purpose::Signup, "a@b.co", 3700, K).is_ok(), "la limite se libère après une heure");
    }
    #[test]
    fn code_is_never_stored_and_is_bound_to_address_purpose_and_secret() {
        let (c, code) = Challenge::issue(None, Purpose::Recovery, "a@b.co", 0, K).unwrap();
        assert!(!serde_json::to_string(&c).unwrap().contains(&code));
        let mut other_purpose = Challenge { purpose: Purpose::Signup, ..c.clone() };
        assert!(matches!(other_purpose.verify(&code, 1, K), VerifyResult::Wrong { .. }));
        let mut other_secret = c.clone();
        assert!(matches!(other_secret.verify(&code, 1, b"zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz"), VerifyResult::Wrong { .. }), "une fuite de la base sans le secret ne permet pas de retrouver un code");
        let mut other_addr = Challenge { email: "x@y.co".into(), ..c };
        assert!(matches!(other_addr.verify(&code, 1, K), VerifyResult::Wrong { .. }));
    }
}
