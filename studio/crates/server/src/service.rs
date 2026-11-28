//! Service d'authentification côté serveur.
//!
//! Règles appliquées (toutes testées) :
//! * **le CAPTCHA est vérifié côté serveur avant toute autre logique** (inscription, récupération, connexions suspectes) ;
//! * un compte **n'est actif qu'après vérification** du courriel : pas de session, pas de connexion avant ;
//! * l'activation exige **à la fois** le code reçu par courriel **et** un jeton secret remis au client qui s'est inscrit
//!   (contre le « pré-détournement » : quelqu'un qui inscrit l'adresse d'autrui ne peut pas la faire activer à sa place) ;
//! * réponses identiques pour adresses connues et inconnues (pas d'énumération de comptes) ;
//! * un changement d'adresse n'a d'effet qu'après vérification de la **nouvelle** adresse, l'ancienne est avertie ;
//! * les codes et le jeton d'inscription ne sont jamais stockés en clair.
//!
//! Persistance : fichier JSON atomique (processus unique). Une base SQL transactionnelle est requise pour plusieurs instances.

use crate::captcha::CaptchaVerifier;
use crate::code::{Challenge, IssueError, Purpose, VerifyResult, RESEND_COOLDOWN, TTL_SECONDS};
use crate::env::{self, ConfigError, Environment};
use crate::mail::{EmailSender, Event, MailKind, Message, Outbox, State};
use crate::templates::{render, Brand, SecurityEvent, Template};
use crate::terms::{Consents, LegalVersions};
use crate::ApiError;
use norelias_account::crypto::{hex, random_bytes, sha256_hex, ct_eq, KdfCost};
use norelias_account::record::{AccountRecord, LoginOutcome};
use norelias_account::validate;
use norelias_core::Lang;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Toutes les clés d'erreur que le service peut produire (testées contre le catalogue i18n).
pub const ERROR_KEYS: &[&str] = &["captcha.required", "captcha.failed", "captcha.unavailable", "code.wrong", "code.expired", "code.too_many_attempts", "code.malformed", "code.invalid",
    "code.cooldown", "code.hourly_limit", "terms.terms_required", "terms.privacy_required", "email.undeliverable", "auth.email_not_verified", "recovery.needs_second_factor",
    "rate.too_many_requests", "error.generic", "error.email.invalid", "error.password.weak", "auth.invalid", "auth.locked", "auth.totp_required"];

pub const GRACE_DELETION_DAYS: u64 = 30;
pub const MAX_ADDRESS_CHANGES_WHILE_PENDING: u32 = 3;

pub struct Config { pub env: Environment, pub secret: Vec<u8>, pub brand: Brand, pub legal: LegalVersions, pub cost: KdfCost }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum Status { Pending, Active }

#[derive(Clone, Serialize, Deserialize)]
struct PendingChange { new_email: String, challenge: Challenge }

#[derive(Clone, Serialize, Deserialize)]
struct Account {
    rec: AccountRecord,
    status: Status,
    signup_token_hash: String,
    challenge: Option<Challenge>,
    change: Option<PendingChange>,
    recovery: Option<Challenge>,
    consents: Consents,
    lang: Lang,
    address_changes: u32,
    deletion_effective: Option<u64>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Delivery { Sent, Queued }
#[derive(Debug)]
pub struct Registered { /// Secret à conserver côté client jusqu'à l'activation.
    pub signup_token: String, pub retry_after: u64, pub delivery: Delivery }
#[derive(Debug)]
pub struct Activated { /// Affichés une seule fois.
    pub recovery_codes: Vec<String> }
#[derive(Debug)]
pub struct LoggedIn { pub session_id: String, pub token: String, pub must_accept_terms: bool }

pub struct RegisterRequest<'r> { pub email: &'r str, pub password: &'r str, pub display_name: &'r str, pub lang: Lang, pub captcha_token: &'r str, pub ip: Option<&'r str>,
    pub accept_terms: bool, pub accept_privacy: bool, pub marketing: bool }

#[derive(Default)]
struct Windows { m: HashMap<(String, String), Vec<u64>> }
impl Windows {
    fn count(&mut self, action: &str, ip: &str, window: u64, now: u64) -> usize { let v = self.m.entry((action.into(), ip.into())).or_default(); v.retain(|t| now.saturating_sub(*t) < window); v.len() }
    fn hit(&mut self, action: &str, ip: &str, max: usize, window: u64, now: u64) -> Result<(), u64> {
        let n = self.count(action, ip, window, now);
        let v = self.m.get_mut(&(action.to_string(), ip.to_string())).unwrap();
        if n >= max { return Err(window - (now - v[0])); }
        v.push(now);
        Ok(())
    }
    fn add(&mut self, action: &str, ip: &str, now: u64) { self.m.entry((action.into(), ip.into())).or_default().push(now); }
}

pub struct Server<'a> {
    cfg: Config,
    sender: &'a dyn EmailSender,
    captcha: &'a dyn CaptchaVerifier,
    accounts: HashMap<String, Account>,
    outbox: Outbox,
    windows: Windows,
}

fn norm(e: &str) -> String { e.trim().to_lowercase() }
fn rate(retry: u64) -> ApiError { ApiError::after("rate.too_many_requests", retry) }

impl<'a> Server<'a> {
    /// Refuse de démarrer en production avec un service simulé, un secret faible ou une URL non HTTPS.
    pub fn new(cfg: Config, sender: &'a dyn EmailSender, captcha: &'a dyn CaptchaVerifier) -> Result<Self, ConfigError> {
        env::check(cfg.env, sender.is_simulated(), captcha.is_simulated(), &cfg.secret, &cfg.brand.base_url)?;
        Ok(Self { cfg, sender, captcha, accounts: HashMap::new(), outbox: Outbox::default(), windows: Windows::default() })
    }

    // ───────── envoi ─────────
    fn deliver(&mut self, to: &str, tpl: Template, lang: Lang, now: u64) -> Delivery2 {
        let msg: Message = render(&tpl, lang, to, &self.cfg.brand);
        let id = self.outbox.push(msg, now);
        let events = self.outbox.process(self.sender, now);
        self.apply_events(&events);
        match events.iter().find(|e| e.id == id).map(|e| &e.state) {
            Some(State::Sent(_)) => Delivery2::Sent,
            Some(State::Undeliverable(_)) => Delivery2::Undeliverable,
            Some(State::Failed(_)) => Delivery2::Queued,
            _ => Delivery2::Queued,
        }
    }
    fn apply_events(&mut self, _events: &[Event]) {}
    /// À appeler périodiquement : reprend les envois en attente. Retourne les événements (rebonds, échecs définitifs).
    pub fn pump(&mut self, now: u64) -> Vec<Event> { let ev = self.outbox.process(self.sender, now); self.apply_events(&ev); ev }
    pub fn pending_mail(&self) -> usize { self.outbox.pending() }

    fn captcha_ok(&self, token: &str, ip: Option<&str>) -> Result<(), ApiError> { self.captcha.verify(token, ip).map_err(|e| ApiError::new(e.key())) }

    fn issue(&self, prev: Option<&Challenge>, purpose: Purpose, email: &str, now: u64) -> Result<(Challenge, String), ApiError> {
        Challenge::issue(prev, purpose, email, now, &self.cfg.secret).map_err(|e| match e { IssueError::Cooldown(s) => ApiError::after("code.cooldown", s), IssueError::HourlyLimit(s) => ApiError::after("code.hourly_limit", s) })
    }
    fn verify_challenge(&self, ch: &mut Challenge, code: &str, now: u64) -> Result<(), ApiError> {
        match ch.verify(code, now, &self.cfg.secret) {
            VerifyResult::Ok => Ok(()), VerifyResult::Wrong { .. } => Err(ApiError::new("code.wrong")), VerifyResult::Expired => Err(ApiError::new("code.expired")),
            VerifyResult::TooManyAttempts => Err(ApiError::new("code.too_many_attempts")), VerifyResult::Malformed => Err(ApiError::new("code.malformed")),
        }
    }
    fn token_hash(t: &str) -> String { sha256_hex(t.as_bytes()) }
    fn limit(&mut self, action: &str, ip: Option<&str>, max: usize, window: u64, now: u64) -> Result<(), ApiError> { self.windows.hit(action, ip.unwrap_or("?"), max, window, now).map_err(rate) }

    // ───────── inscription ─────────
    pub fn register(&mut self, r: &RegisterRequest, now: u64) -> Result<Registered, ApiError> {
        self.limit("register", r.ip, 5, 3600, now)?;
        self.captcha_ok(r.captcha_token, r.ip)?; // d'abord : aucune logique de compte avant la preuve anti-robot
        let consents = Consents::at_signup(r.accept_terms, r.accept_privacy, r.marketing, &self.cfg.legal, now).map_err(|e| ApiError::new(e.key()))?;
        let email = norm(r.email);
        if !validate::email(&email) { return Err(ApiError::new("error.email.invalid")); }
        let token = hex(&random_bytes::<32>());
        if self.accounts.get(&email).is_some_and(|a| a.status == Status::Active) {
            // Même réponse qu'une inscription réussie ; le titulaire est averti par courriel.
            self.deliver(&email, Template::Security(SecurityEvent::AccountExists), r.lang, now);
            return Ok(Registered { signup_token: token, retry_after: RESEND_COOLDOWN, delivery: Delivery::Sent });
        }
        let (rec, _codes) = AccountRecord::create(&email, r.password, r.display_name, self.cfg.cost, now).map_err(|_| ApiError::new("error.password.weak"))?;
        let (challenge, code) = self.issue(None, Purpose::Signup, &email, now)?;
        let name = Some(r.display_name.to_string()).filter(|n| !n.trim().is_empty());
        let d = self.deliver(&email, Template::Verification { name, code, minutes: (TTL_SECONDS / 60) as u32 }, r.lang, now);
        if d == Delivery2::Undeliverable { return Err(ApiError::new("email.undeliverable")); }
        self.accounts.insert(email, Account { rec, status: Status::Pending, signup_token_hash: Self::token_hash(&token), challenge: Some(challenge), change: None, recovery: None, consents, lang: r.lang, address_changes: 0, deletion_effective: None });
        Ok(Registered { signup_token: token, retry_after: RESEND_COOLDOWN, delivery: if d == Delivery2::Sent { Delivery::Sent } else { Delivery::Queued } })
    }

    fn pending_mut(&mut self, email: &str, token: &str) -> Option<&mut Account> {
        let h = Self::token_hash(token);
        self.accounts.get_mut(&norm(email)).filter(|a| a.status == Status::Pending && ct_eq(a.signup_token_hash.as_bytes(), h.as_bytes()))
    }

    /// Renvoi du code (60 s minimum entre deux envois, 5 par heure). Réponse identique si le compte n'existe pas.
    pub fn resend_code(&mut self, email: &str, signup_token: &str, ip: Option<&str>, now: u64) -> Result<u64, ApiError> {
        self.limit("resend", ip, 10, 3600, now)?;
        let email = norm(email);
        let Some(a) = self.pending_mut(&email, signup_token) else { return Ok(RESEND_COOLDOWN) };
        let prev = a.challenge.clone();
        let (lang, name) = (a.lang, Some(a.rec.display_name.clone()).filter(|n| !n.is_empty()));
        let (ch, code) = self.issue(prev.as_ref(), Purpose::Signup, &email, now)?;
        self.accounts.get_mut(&email).unwrap().challenge = Some(ch);
        let d = self.deliver(&email, Template::Verification { name, code, minutes: (TTL_SECONDS / 60) as u32 }, lang, now);
        if d == Delivery2::Undeliverable { return Err(ApiError::new("email.undeliverable")); }
        Ok(RESEND_COOLDOWN)
    }

    /// Corriger l'adresse avant validation : l'ancien code est invalidé, un nouveau est envoyé à la nouvelle adresse.
    pub fn change_pending_email(&mut self, email: &str, signup_token: &str, new_email: &str, ip: Option<&str>, now: u64) -> Result<u64, ApiError> {
        self.limit("change_pending", ip, 10, 3600, now)?;
        let (old, new) = (norm(email), norm(new_email));
        if !validate::email(&new) { return Err(ApiError::new("error.email.invalid")); }
        let Some(a) = self.pending_mut(&old, signup_token) else { return Ok(RESEND_COOLDOWN) };
        if a.address_changes >= MAX_ADDRESS_CHANGES_WHILE_PENDING { return Err(rate(3600)); }
        let lang = a.lang;
        if self.accounts.get(&new).is_some_and(|x| x.status == Status::Active) {
            self.deliver(&new, Template::Security(SecurityEvent::AccountExists), lang, now);
            return Ok(RESEND_COOLDOWN);
        }
        let mut acc = self.accounts.remove(&old).unwrap();
        let (ch, code) = match self.issue(acc.challenge.as_ref(), Purpose::Signup, &new, now) { Ok(x) => x, Err(e) => { self.accounts.insert(old, acc); return Err(e); } };
        acc.rec.email = new.clone();
        acc.challenge = Some(ch);
        acc.address_changes += 1;
        let name = Some(acc.rec.display_name.clone()).filter(|n| !n.is_empty());
        self.accounts.insert(new.clone(), acc);
        let d = self.deliver(&new, Template::Verification { name, code, minutes: (TTL_SECONDS / 60) as u32 }, lang, now);
        if d == Delivery2::Undeliverable { return Err(ApiError::new("email.undeliverable")); }
        Ok(RESEND_COOLDOWN)
    }

    /// Active le compte : code valide **et** jeton d'inscription.
    pub fn verify_email(&mut self, email: &str, signup_token: &str, code: &str, ip: Option<&str>, now: u64) -> Result<Activated, ApiError> {
        self.limit("verify", ip, 30, 3600, now)?;
        let email = norm(email);
        let Some(a) = self.pending_mut(&email, signup_token) else { return Err(ApiError::new("code.invalid")) };
        let Some(mut ch) = a.challenge.clone() else { return Err(ApiError::new("code.invalid")) };
        let res = self.verify_challenge(&mut ch, code, now);
        let a = self.accounts.get_mut(&email).unwrap();
        a.challenge = Some(ch);
        res?;
        a.status = Status::Active;
        a.rec.email_verified = true;
        a.challenge = None;
        let codes = a.rec.issue_recovery_codes();
        let (lang, name) = (a.lang, Some(a.rec.display_name.clone()).filter(|n| !n.is_empty()));
        self.deliver(&email, Template::Welcome { name }, lang, now);
        Ok(Activated { recovery_codes: codes })
    }

    // ───────── connexion ─────────
    pub fn login(&mut self, email: &str, password: &str, totp: Option<&str>, device: &str, captcha_token: Option<&str>, ip: Option<&str>, now: u64) -> Result<LoggedIn, ApiError> {
        let (email, ipk) = (norm(email), ip.unwrap_or("?").to_string());
        let ip_fail = self.windows.count("login_fail", &ipk, 3600, now);
        let acct_fail = self.accounts.get(&email).map_or(0, |a| a.rec.failed_logins());
        if ip_fail >= 3 || acct_fail >= 2 {
            // connexion suspecte : preuve anti-robot réelle exigée
            self.captcha_ok(captcha_token.unwrap_or(""), ip)?;
        }
        let Some(a) = self.accounts.get_mut(&email) else {
            let _ = norelias_account::crypto::verify_password(password, "argon2id:64,1,1:00:00"); // égalise le temps de réponse
            self.windows.add("login_fail", &ipk, now);
            return Err(ApiError::new("auth.invalid"));
        };
        match a.rec.login(&email, password, totp, device, now) {
            LoginOutcome::Locked { seconds } => Err(ApiError::after("auth.locked", seconds)),
            LoginOutcome::Invalid => { self.windows.add("login_fail", &ipk, now); Err(ApiError::new("auth.invalid")) }
            LoginOutcome::TotpRequired => Err(ApiError::new("auth.totp_required")),
            LoginOutcome::Success { session_id, token } => {
                if a.status == Status::Pending {
                    // mot de passe correct mais courriel non vérifié : on annule la session créée, aucun accès
                    a.rec.sessions.revoke(&session_id);
                    return Err(ApiError::new("auth.email_not_verified"));
                }
                let first_device = a.rec.sessions.list().iter().filter(|s| s.id != session_id).all(|s| s.device != device);
                let had_before = a.rec.sessions.list().len() > 1;
                let (lang, must) = (a.lang, a.consents.needs_reacceptance(&self.cfg.legal));
                if first_device && had_before {
                    self.deliver(&email, Template::Security(SecurityEvent::NewDevice { device: device.chars().take(80).collect(), when: format!("t={now}") }), lang, now);
                }
                Ok(LoggedIn { session_id, token, must_accept_terms: must })
            }
        }
    }

    fn by_session(&mut self, token: &str, now: u64) -> Option<String> {
        for (k, a) in self.accounts.iter_mut() { if a.status == Status::Active && a.rec.sessions.validate(token, now).is_some() { return Some(k.clone()); } }
        None
    }

    // ───────── changement d'adresse (compte actif) ─────────
    pub fn request_email_change(&mut self, session: &str, password: &str, new_email: &str, now: u64) -> Result<u64, ApiError> {
        let key = self.by_session(session, now).ok_or(ApiError::new("auth.invalid"))?;
        let new = norm(new_email);
        if !validate::email(&new) { return Err(ApiError::new("error.email.invalid")); }
        self.accounts.get_mut(&key).unwrap().rec.confirm_password(password, now).map_err(|_| ApiError::new("auth.invalid"))?;
        let lang = self.accounts[&key].lang;
        if self.accounts.contains_key(&new) {
            self.deliver(&new, Template::Security(SecurityEvent::AccountExists), lang, now);
            return Ok(RESEND_COOLDOWN); // même réponse : pas d'énumération
        }
        let prev = self.accounts[&key].change.as_ref().map(|c| c.challenge.clone());
        let (ch, code) = self.issue(prev.as_ref(), Purpose::ChangeEmail, &new, now)?;
        self.accounts.get_mut(&key).unwrap().change = Some(PendingChange { new_email: new.clone(), challenge: ch });
        let d = self.deliver(&new, Template::Verification { name: None, code, minutes: (TTL_SECONDS / 60) as u32 }, lang, now);
        if d == Delivery2::Undeliverable { self.accounts.get_mut(&key).unwrap().change = None; return Err(ApiError::new("email.undeliverable")); }
        self.deliver(&key, Template::Security(SecurityEvent::EmailChangeRequested { new_email: new }), lang, now);
        Ok(RESEND_COOLDOWN)
    }
    pub fn confirm_email_change(&mut self, session: &str, code: &str, now: u64) -> Result<(), ApiError> {
        let key = self.by_session(session, now).ok_or(ApiError::new("auth.invalid"))?;
        let mut ch = self.accounts[&key].change.clone().ok_or(ApiError::new("code.invalid"))?;
        let res = self.verify_challenge(&mut ch.challenge, code, now);
        self.accounts.get_mut(&key).unwrap().change = Some(ch.clone());
        res?;
        let mut acc = self.accounts.remove(&key).unwrap();
        let (lang, new) = (acc.lang, ch.new_email.clone());
        acc.rec.email = new.clone();
        acc.rec.email_verified = true;
        acc.change = None;
        acc.rec.sessions.revoke_all_except(None); // l'utilisateur se reconnecte avec la nouvelle adresse
        self.accounts.insert(new.clone(), acc);
        self.deliver(&key, Template::Security(SecurityEvent::EmailChanged { new_email: new.clone() }), lang, now);
        self.deliver(&new, Template::Security(SecurityEvent::EmailChanged { new_email: new.clone() }), lang, now);
        Ok(())
    }

    // ───────── récupération ─────────
    /// Réponse toujours identique (existence du compte non révélée). CAPTCHA obligatoire.
    pub fn request_recovery(&mut self, email: &str, captcha_token: &str, ip: Option<&str>, now: u64) -> Result<u64, ApiError> {
        self.limit("recovery", ip, 5, 3600, now)?;
        self.captcha_ok(captcha_token, ip)?;
        let email = norm(email);
        let Some(a) = self.accounts.get(&email).filter(|a| a.status == Status::Active) else { return Ok(RESEND_COOLDOWN) };
        let (prev, lang) = (a.recovery.clone(), a.lang);
        if let Ok((ch, code)) = self.issue(prev.as_ref(), Purpose::Recovery, &email, now) {
            self.accounts.get_mut(&email).unwrap().recovery = Some(ch);
            self.deliver(&email, Template::PasswordRecovery { code, minutes: (TTL_SECONDS / 60) as u32 }, lang, now);
        }
        Ok(RESEND_COOLDOWN)
    }
    /// Réinitialisation : code reçu par courriel ; si le 2FA est actif, un code de récupération est aussi exigé.
    pub fn recover(&mut self, email: &str, code: &str, new_password: &str, recovery_code: Option<&str>, ip: Option<&str>, now: u64) -> Result<(), ApiError> {
        self.limit("recover", ip, 20, 3600, now)?;
        let email = norm(email);
        let Some(a) = self.accounts.get(&email).filter(|a| a.status == Status::Active) else { return Err(ApiError::new("code.invalid")) };
        let mut ch = a.recovery.clone().ok_or(ApiError::new("code.invalid"))?;
        let second = a.rec.totp_enabled();
        let res = self.verify_challenge(&mut ch, code, now);
        self.accounts.get_mut(&email).unwrap().recovery = Some(ch);
        res?;
        let lang = self.accounts[&email].lang;
        let a = self.accounts.get_mut(&email).unwrap();
        if second {
            let rc = recovery_code.ok_or(ApiError::new("recovery.needs_second_factor"))?;
            a.rec.reset_with_recovery(rc, new_password, now).map_err(|_| ApiError::new("error.password.weak"))?;
        } else {
            a.rec.reset_password_email_verified(new_password, now).map_err(|_| ApiError::new("error.password.weak"))?;
        }
        a.recovery = None;
        self.deliver(&email, Template::Security(SecurityEvent::PasswordChanged), lang, now);
        Ok(())
    }

    // ───────── authentification à deux facteurs (facultative) ─────────
    /// Démarre l'enrôlement : retourne (secret base32, URI otpauth). Actif seulement après `confirm_two_factor`.
    pub fn begin_two_factor(&mut self, session: &str, password: &str, now: u64) -> Result<(String, String), ApiError> {
        let key = self.by_session(session, now).ok_or(ApiError::new("auth.invalid"))?;
        let issuer = self.cfg.brand.product.clone();
        self.accounts.get_mut(&key).unwrap().rec.begin_totp(password, &issuer, now).map_err(|_| ApiError::new("auth.invalid"))
    }
    pub fn confirm_two_factor(&mut self, session: &str, password: &str, code: &str, now: u64) -> Result<(), ApiError> {
        let key = self.by_session(session, now).ok_or(ApiError::new("auth.invalid"))?;
        self.accounts.get_mut(&key).unwrap().rec.confirm_totp(password, code, now).map_err(|_| ApiError::new("code.wrong"))?;
        let lang = self.accounts[&key].lang;
        self.deliver(&key, Template::Security(SecurityEvent::TwoFactorEnabled), lang, now);
        Ok(())
    }
    pub fn disable_two_factor(&mut self, session: &str, password: &str, now: u64) -> Result<(), ApiError> {
        let key = self.by_session(session, now).ok_or(ApiError::new("auth.invalid"))?;
        self.accounts.get_mut(&key).unwrap().rec.disable_totp(password, now).map_err(|_| ApiError::new("auth.invalid"))?;
        let lang = self.accounts[&key].lang;
        self.deliver(&key, Template::Security(SecurityEvent::TwoFactorDisabled), lang, now);
        Ok(())
    }

    // ───────── consentements, appareils, suppression ─────────
    pub fn set_marketing(&mut self, session: &str, granted: bool, now: u64) -> Result<(), ApiError> {
        let key = self.by_session(session, now).ok_or(ApiError::new("auth.invalid"))?;
        let legal = self.cfg.legal.clone();
        self.accounts.get_mut(&key).unwrap().consents.set_marketing(granted, &legal, now);
        Ok(())
    }
    pub fn accept_current_terms(&mut self, session: &str, now: u64) -> Result<(), ApiError> {
        let key = self.by_session(session, now).ok_or(ApiError::new("auth.invalid"))?;
        let legal = self.cfg.legal.clone();
        self.accounts.get_mut(&key).unwrap().consents.accept_current(&legal, now);
        Ok(())
    }
    pub fn devices(&mut self, session: &str, now: u64) -> Result<Vec<(String, String)>, ApiError> {
        let key = self.by_session(session, now).ok_or(ApiError::new("auth.invalid"))?;
        Ok(self.accounts[&key].rec.sessions.list().iter().map(|s| (s.id.clone(), s.device.clone())).collect())
    }
    pub fn revoke_device(&mut self, session: &str, id: &str, now: u64) -> Result<bool, ApiError> {
        let key = self.by_session(session, now).ok_or(ApiError::new("auth.invalid"))?;
        Ok(self.accounts.get_mut(&key).unwrap().rec.sessions.revoke(id))
    }
    pub fn consents_of(&self, email: &str) -> Option<&Consents> { self.accounts.get(&norm(email)).map(|a| &a.consents) }

    /// Suppression : mot de passe confirmé, courriel de confirmation, sessions fermées, effacement après 30 jours.
    pub fn request_deletion(&mut self, session: &str, password: &str, now: u64) -> Result<u64, ApiError> {
        let key = self.by_session(session, now).ok_or(ApiError::new("auth.invalid"))?;
        let a = self.accounts.get_mut(&key).unwrap();
        a.rec.confirm_password(password, now).map_err(|_| ApiError::new("auth.invalid"))?;
        let eff = now + GRACE_DELETION_DAYS * 86_400;
        a.deletion_effective = Some(eff);
        a.rec.sessions.revoke_all_except(None);
        let lang = a.lang;
        let days = (eff / 86_400) as i64;
        let (y, m, d) = civil_from_days(days);
        self.deliver(&key, Template::AccountDeletion { effective: (y, m, d) }, lang, now);
        Ok(eff)
    }
    /// Efface les comptes dont le délai est écoulé. Retourne le nombre de comptes supprimés.
    pub fn purge_due(&mut self, now: u64) -> usize { let n = self.accounts.len(); self.accounts.retain(|_, a| a.deletion_effective.is_none_or(|e| e > now)); n - self.accounts.len() }
    pub fn account_exists(&self, email: &str) -> bool { self.accounts.contains_key(&norm(email)) }
    pub fn is_active(&self, email: &str) -> bool { self.accounts.get(&norm(email)).is_some_and(|a| a.status == Status::Active) }

    // ───────── persistance ─────────
    pub fn save(&self, dir: &std::path::Path) -> std::io::Result<()> {
        use std::io::Write;
        std::fs::create_dir_all(dir)?;
        let tmp = dir.join("server_state.json.tmp");
        { let mut f = std::fs::File::create(&tmp)?; f.write_all(&serde_json::to_vec(&self.accounts)?)?; f.sync_all()?; }
        #[cfg(unix)] { use std::os::unix::fs::PermissionsExt; std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o600))?; }
        std::fs::rename(tmp, dir.join("server_state.json"))
    }
    pub fn load(&mut self, dir: &std::path::Path) -> std::io::Result<()> { self.accounts = serde_json::from_slice(&std::fs::read(dir.join("server_state.json"))?)?; Ok(()) }
}

#[derive(Debug, PartialEq, Eq)]
enum Delivery2 { Sent, Queued, Undeliverable }

/// Date civile (UTC) d'un nombre de jours depuis 1970-01-01.
fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719468; let era = z.div_euclid(146097); let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    ((yoe + era * 400 + if m <= 2 { 1 } else { 0 }) as i32, m, d)
}

#[allow(dead_code)]
fn kind_is_security(k: MailKind) -> bool { matches!(k, MailKind::Security | MailKind::EmailChangeNotice) }
