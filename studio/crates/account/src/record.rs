//! Compte **local** : identifiants, 2FA, codes de récupération, sessions. Stockage atomique.
//! Mode local : fonctionne hors connexion. `email_verified` reste faux tant qu'aucun service en ligne ne l'a confirmé.

use crate::crypto::{self, hash_password, random_bytes, seal, sha256_hex, unseal, verify_password, KdfCost, Sealed};
use crate::limiter::RateLimiter;
use crate::session::Sessions;
use crate::{password, totp, validate};
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TotpState { sealed: Sealed, enabled: bool, last_step: Option<u64> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountRecord {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pw_hash: String,
    totp: Option<TotpState>,
    recovery: Vec<String>,
    limiter: RateLimiter,
    pub sessions: Sessions,
    pub created_at: u64,
    pub email_verified: bool,
    cost: KdfCost,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LoginOutcome { Success { session_id: String, token: String }, TotpRequired, Locked { seconds: u64 }, Invalid }

#[derive(Debug, PartialEq, Eq)]
pub enum AccountError { InvalidEmail, WeakPassword, SamePassword, Locked(u64), Invalid, NoTotp }

fn norm_email(e: &str) -> String { e.trim().to_lowercase() }
fn new_recovery_codes() -> Vec<String> {
    (0..10).map(|_| { let s = totp::base32_encode(&random_bytes::<7>()); format!("{}-{}", &s[..5], &s[5..10]) }).collect()
}
fn rec_hash(code: &str) -> String { sha256_hex(code.replace(['-', ' '], "").to_uppercase().as_bytes()) }

impl AccountRecord {
    /// Crée le compte. Retourne les codes de récupération, **affichés une seule fois**.
    pub fn create(email: &str, password: &str, display_name: &str, cost: KdfCost, now: u64) -> Result<(Self, Vec<String>), AccountError> {
        let email = norm_email(email);
        if !validate::email(&email) { return Err(AccountError::InvalidEmail); }
        if !password::strength(password, &[&email, display_name]).acceptable { return Err(AccountError::WeakPassword); }
        let codes = new_recovery_codes();
        let rec = AccountRecord { id: crypto::hex(&random_bytes::<8>()), email, display_name: display_name.trim().into(), pw_hash: hash_password(password, cost), totp: None,
            recovery: codes.iter().map(|c| rec_hash(c)).collect(), limiter: RateLimiter::default(), sessions: Sessions::default(), created_at: now, email_verified: false, cost };
        Ok((rec, codes))
    }

    pub fn login(&mut self, email: &str, password: &str, totp_code: Option<&str>, device: &str, now: u64) -> LoginOutcome {
        if let Some(seconds) = self.limiter.blocked_for(now) { return LoginOutcome::Locked { seconds }; }
        // Le mot de passe est toujours vérifié, même si le courriel diffère : durée et réponse identiques.
        let pw_ok = verify_password(password, &self.pw_hash);
        if !pw_ok || norm_email(email) != self.email { self.limiter.record_failure(now); return LoginOutcome::Invalid; }
        if let Some(t) = self.totp.as_mut().filter(|t| t.enabled) {
            let Some(code) = totp_code else { return LoginOutcome::TotpRequired };
            let secret = unseal(&t.sealed, password);
            match secret.and_then(|s| totp::verify(&s, code, now, t.last_step)) {
                Some(step) => t.last_step = Some(step),
                None => { self.limiter.record_failure(now); return LoginOutcome::Invalid; }
            }
        }
        self.limiter.record_success();
        let (session_id, token) = self.sessions.create(device, now);
        LoginOutcome::Success { session_id, token }
    }

    fn check_password(&mut self, pw: &str, now: u64) -> Result<(), AccountError> {
        if let Some(s) = self.limiter.blocked_for(now) { return Err(AccountError::Locked(s)); }
        if verify_password(pw, &self.pw_hash) { Ok(()) } else { self.limiter.record_failure(now); Err(AccountError::Invalid) }
    }

    /// Prépare le 2FA : retourne (secret base32, URI otpauth). Il n'est actif qu'après `confirm_totp`.
    pub fn begin_totp(&mut self, pw: &str, issuer: &str, now: u64) -> Result<(String, String), AccountError> {
        self.check_password(pw, now)?;
        let secret = totp::generate_secret();
        self.totp = Some(TotpState { sealed: seal(&secret, pw, self.cost), enabled: false, last_step: None });
        Ok((totp::base32_encode(&secret), totp::otpauth_uri(&secret, &self.email, issuer)))
    }
    pub fn confirm_totp(&mut self, pw: &str, code: &str, now: u64) -> Result<(), AccountError> {
        self.check_password(pw, now)?;
        let t = self.totp.as_mut().ok_or(AccountError::NoTotp)?;
        let secret = unseal(&t.sealed, pw).ok_or(AccountError::Invalid)?;
        let step = totp::verify(&secret, code, now, None).ok_or(AccountError::Invalid)?;
        t.enabled = true;
        t.last_step = Some(step);
        Ok(())
    }
    pub fn totp_enabled(&self) -> bool { self.totp.as_ref().is_some_and(|t| t.enabled) }
    pub fn disable_totp(&mut self, pw: &str, now: u64) -> Result<(), AccountError> { self.check_password(pw, now)?; self.totp = None; Ok(()) }

    /// Change le mot de passe : rechiffre le secret 2FA et révoque **toutes** les sessions.
    pub fn change_password(&mut self, old: &str, new: &str, now: u64) -> Result<(), AccountError> {
        self.check_password(old, now)?;
        if old == new { return Err(AccountError::SamePassword); }
        if !password::strength(new, &[&self.email, &self.display_name]).acceptable { return Err(AccountError::WeakPassword); }
        if let Some(t) = self.totp.as_mut() {
            let secret = unseal(&t.sealed, old).ok_or(AccountError::Invalid)?;
            t.sealed = seal(&secret, new, self.cost);
        }
        self.pw_hash = hash_password(new, self.cost);
        self.sessions.revoke_all_except(None);
        Ok(())
    }

    /// Récupération : un code à usage unique réinitialise le mot de passe. Le 2FA est **désactivé** (son secret
    /// était chiffré par l'ancien mot de passe) et toutes les sessions sont révoquées. Soumis à la limitation de tentatives.
    pub fn reset_with_recovery(&mut self, code: &str, new_pw: &str, now: u64) -> Result<(), AccountError> {
        if let Some(s) = self.limiter.blocked_for(now) { return Err(AccountError::Locked(s)); }
        let h = rec_hash(code);
        let Some(i) = self.recovery.iter().position(|r| crypto::ct_eq(r.as_bytes(), h.as_bytes())) else { self.limiter.record_failure(now); return Err(AccountError::Invalid) };
        if !password::strength(new_pw, &[&self.email, &self.display_name]).acceptable { return Err(AccountError::WeakPassword); }
        self.recovery.remove(i);
        self.pw_hash = hash_password(new_pw, self.cost);
        self.totp = None;
        self.sessions.revoke_all_except(None);
        self.limiter.record_success();
        Ok(())
    }
    pub fn recovery_codes_left(&self) -> usize { self.recovery.len() }
    pub fn regenerate_recovery(&mut self, pw: &str, now: u64) -> Result<Vec<String>, AccountError> {
        self.check_password(pw, now)?;
        let codes = new_recovery_codes();
        self.recovery = codes.iter().map(|c| rec_hash(c)).collect();
        Ok(codes)
    }

    /// Export des données personnelles (portabilité). Aucun secret : ni hachage, ni 2FA, ni jetons.
    pub fn export_json(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id, "email": self.email, "display_name": self.display_name, "created_at": self.created_at,
            "email_verified": self.email_verified, "two_factor_enabled": self.totp_enabled(),
            "recovery_codes_remaining": self.recovery.len(),
            "devices": self.sessions.list().iter().map(|s| serde_json::json!({"id": s.id, "device": s.device, "created": s.created, "last_seen": s.last_seen})).collect::<Vec<_>>(),
        })
    }
}

pub struct AccountStore { dir: PathBuf }
impl AccountStore {
    pub fn new(dir: impl Into<PathBuf>) -> Self { Self { dir: dir.into() } }
    fn path(&self) -> PathBuf { self.dir.join("account.json") }
    pub fn exists(&self) -> bool { self.path().exists() }
    pub fn save(&self, a: &AccountRecord) -> std::io::Result<()> {
        fs::create_dir_all(&self.dir)?;
        let tmp = self.dir.join("account.json.tmp");
        { let mut f = fs::File::create(&tmp)?; f.write_all(serde_json::to_string_pretty(a)?.as_bytes())?; f.sync_all()?; }
        #[cfg(unix)] { use std::os::unix::fs::PermissionsExt; fs::set_permissions(&tmp, fs::Permissions::from_mode(0o600))?; }
        fs::rename(tmp, self.path())
    }
    pub fn load(&self) -> std::io::Result<Option<AccountRecord>> {
        if !self.exists() { return Ok(None); }
        Ok(Some(serde_json::from_slice(&fs::read(self.path())?)?))
    }
    /// Suppression du compte : écrasement puis suppression (limites du SSD : voir SECURITY.md).
    pub fn delete(&self) -> std::io::Result<bool> {
        let p = self.path();
        if !p.exists() { return Ok(false); }
        let len = fs::metadata(&p)?.len() as usize;
        { let mut f = fs::File::create(&p)?; f.write_all(&vec![0u8; len])?; f.sync_all()?; }
        fs::remove_file(p)?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::limiter::{FREE_ATTEMPTS, LOCK_SECONDS};
    const C: KdfCost = KdfCost::TESTING;
    const PW: &str = "Correct-Horse-Battery-42";
    fn acct() -> AccountRecord { AccountRecord::create("Jean@Exemple.CA", PW, "Jean", C, 0).unwrap().0 }
    fn ok(o: LoginOutcome) -> String { match o { LoginOutcome::Success { token, .. } => token, x => panic!("{x:?}") } }

    #[test]
    fn creation_rules_and_no_plaintext() {
        assert!(matches!(AccountRecord::create("pas-un-courriel", PW, "", C, 0), Err(AccountError::InvalidEmail)));
        assert!(matches!(AccountRecord::create("a@b.co", "password", "", C, 0), Err(AccountError::WeakPassword)));
        let (a, codes) = AccountRecord::create("a@b.co", PW, "A", C, 0).unwrap();
        let json = serde_json::to_string(&a).unwrap();
        assert!(!json.contains(PW) && !json.contains(&codes[0]), "ni mot de passe ni code de récupération en clair");
        assert_eq!((codes.len(), a.email_verified), (10, false));
    }
    #[test]
    fn login_flow_and_generic_errors() {
        let mut a = acct();
        let tok = ok(a.login("jean@exemple.ca", PW, None, "Portable", 10));
        assert_eq!(a.sessions.validate(&tok, 11).is_some(), true);
        assert_eq!(a.login("jean@exemple.ca", "mauvais", None, "x", 20), LoginOutcome::Invalid);
        assert_eq!(a.login("autre@exemple.ca", PW, None, "x", 21), LoginOutcome::Invalid, "même réponse : pas d'énumération de comptes");
    }
    #[test]
    fn brute_force_is_throttled_then_locked() {
        let mut a = acct();
        for i in 0..FREE_ATTEMPTS { assert_eq!(a.login("jean@exemple.ca", "x", None, "d", 100 + i as u64), LoginOutcome::Invalid); }
        assert!(matches!(a.login("jean@exemple.ca", "x", None, "d", 103), LoginOutcome::Invalid));
        assert!(matches!(a.login("jean@exemple.ca", PW, None, "d", 103), LoginOutcome::Locked { .. }), "même le bon mot de passe est refusé pendant le délai");
        let mut t = 103;
        for _ in 0..4 { t += 100; a.login("jean@exemple.ca", "x", None, "d", t); }
        assert!(matches!(a.login("jean@exemple.ca", PW, None, "d", t + 1), LoginOutcome::Locked { seconds } if seconds > 60));
        ok(a.login("jean@exemple.ca", PW, None, "d", t + LOCK_SECONDS + 1));
    }
    #[test]
    fn two_factor_enrolment_login_and_replay() {
        let mut a = acct();
        let (secret_b32, uri) = a.begin_totp(PW, "Norélias", 1000).unwrap();
        assert!(uri.contains(&secret_b32) && !a.totp_enabled());
        let secret = totp::base32_decode(&secret_b32).unwrap();
        assert!(a.confirm_totp(PW, "000000", 1000).is_err());
        let code = totp::code_for_step(&secret, totp::step_at(1000), 6);
        a.confirm_totp(PW, &code, 1000).unwrap();
        assert!(a.totp_enabled());
        assert_eq!(a.login("jean@exemple.ca", PW, None, "d", 2000), LoginOutcome::TotpRequired);
        let c2 = totp::code_for_step(&secret, totp::step_at(2000), 6);
        ok(a.login("jean@exemple.ca", PW, Some(&c2), "d", 2000));
        assert_eq!(a.login("jean@exemple.ca", PW, Some(&c2), "d", 2001), LoginOutcome::Invalid, "code déjà utilisé");
        assert!(!serde_json::to_string(&a).unwrap().contains(&secret_b32), "secret 2FA chiffré au repos");
    }
    #[test]
    fn password_change_reseals_2fa_and_revokes_sessions() {
        let mut a = acct();
        let (sb, _) = a.begin_totp(PW, "N", 0).unwrap();
        let secret = totp::base32_decode(&sb).unwrap();
        a.confirm_totp(PW, &totp::code_for_step(&secret, totp::step_at(0), 6), 0).unwrap();
        let (id, tok) = a.sessions.create("vieux", 0);
        let new = "Une-Phrase-Longue-Et-Sûre-7";
        assert_eq!(a.change_password(PW, PW, 5), Err(AccountError::SamePassword));
        assert_eq!(a.change_password(PW, "court", 5), Err(AccountError::WeakPassword));
        a.change_password(PW, new, 5).unwrap();
        assert!(a.sessions.validate(&tok, 6).is_none() && !a.sessions.list().iter().any(|s| s.id == id), "sessions révoquées");
        let c = totp::code_for_step(&secret, totp::step_at(3000), 6);
        ok(a.login("jean@exemple.ca", new, Some(&c), "d", 3000));
        assert_eq!(a.login("jean@exemple.ca", PW, None, "d", 4000), LoginOutcome::Invalid);
    }
    #[test]
    fn recovery_code_is_single_use_and_disables_2fa() {
        let (mut a, codes) = AccountRecord::create("a@b.co", PW, "A", C, 0).unwrap();
        a.begin_totp(PW, "N", 0).unwrap();
        let (_, tok) = a.sessions.create("d", 0);
        assert_eq!(a.reset_with_recovery("AAAAA-AAAAA", "Nouveau-Mot-De-Passe-99", 1), Err(AccountError::Invalid));
        a.reset_with_recovery(&codes[3].to_lowercase(), "Nouveau-Mot-De-Passe-99", 2).unwrap();
        assert_eq!(a.recovery_codes_left(), 9);
        assert!(!a.totp_enabled() && a.sessions.validate(&tok, 3).is_none());
        assert_eq!(a.reset_with_recovery(&codes[3], "Autre-Mot-De-Passe-77x", 4), Err(AccountError::Invalid), "code déjà consommé");
        ok(a.login("a@b.co", "Nouveau-Mot-De-Passe-99", None, "d", 5));
        let fresh = a.regenerate_recovery("Nouveau-Mot-De-Passe-99", 6).unwrap();
        assert_eq!((fresh.len(), a.recovery_codes_left()), (10, 10));
        assert!(a.reset_with_recovery(&codes[0], "Encore-Un-Autre-Mot-55z", 7).is_err(), "anciens codes invalidés");
    }
    #[test]
    fn export_contains_no_secrets() {
        let mut a = acct();
        let (sb, _) = a.begin_totp(PW, "N", 0).unwrap();
        let (_, tok) = a.sessions.create("Portable", 0);
        let j = a.export_json().to_string();
        for secret in [PW, &sb, &tok, "argon2id"] { assert!(!j.contains(secret), "fuite : {secret}"); }
        assert!(j.contains("jean@exemple.ca") && j.contains("Portable"));
    }
    #[test]
    fn store_roundtrip_permissions_and_deletion() {
        let d = std::env::temp_dir().join(format!("norelias-acct-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        let s = AccountStore::new(&d);
        assert!(s.load().unwrap().is_none());
        let a = acct();
        s.save(&a).unwrap();
        let mut b = s.load().unwrap().unwrap();
        ok(b.login("jean@exemple.ca", PW, None, "d", 1));
        #[cfg(unix)] { use std::os::unix::fs::PermissionsExt; assert_eq!(fs::metadata(d.join("account.json")).unwrap().permissions().mode() & 0o777, 0o600); }
        assert!(s.delete().unwrap() && !s.exists() && !s.delete().unwrap());
        let _ = fs::remove_dir_all(&d);
    }
}
