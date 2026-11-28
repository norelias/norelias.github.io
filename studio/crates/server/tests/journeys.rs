//! Parcours complets du service d'authentification : inscription → vérification → activation → connexion → récupération,
//! avec protections anti-robot, limites, erreurs de livraison et garde de production.
//! Les fournisseurs (courriel, CAPTCHA) sont des **doubles de test** : ces tests prouvent la logique du serveur,
//! pas l'intégration avec un fournisseur réel.

use norelias_account::crypto::KdfCost;
use norelias_core::Lang;
use norelias_server::captcha::{CaptchaError, CaptchaVerifier, ScriptedCaptcha};
use norelias_server::env::{ConfigError, Environment};
use norelias_server::mail::{EmailSender, MailKind, Message, RecordingSender, SendError};
use norelias_server::service::*;
use norelias_server::templates::Brand;
use norelias_server::terms::LegalVersions;
use norelias_server::ApiError;

const PW: &str = "Correct-Horse-Battery-42";
const T0: u64 = 1_800_000_000;
const SECRET: &[u8] = b"0123456789abcdef0123456789abcdef";

fn legal(v: &str) -> LegalVersions { LegalVersions { terms: v.into(), privacy: "conf-1".into(), marketing_text: "mkt-1".into() } }
fn cfg() -> Config { Config { env: Environment::Development, secret: SECRET.to_vec(), brand: Brand::default(), legal: legal("cgu-1"), cost: KdfCost::TESTING } }
fn code_in(m: &Message) -> String { let d: String = m.subject.chars().take_while(|c| c.is_ascii_digit()).collect(); assert_eq!(d.len(), 6, "{}", m.subject); d }
fn known(e: &ApiError) -> &'static str { assert!(ERROR_KEYS.contains(&e.key), "clé inconnue : {}", e.key); e.key }
fn req<'r>(email: &'r str, tok: &'r str) -> RegisterRequest<'r> {
    RegisterRequest { email, password: PW, display_name: "Élodie", lang: Lang::Fr, captcha_token: tok, ip: Some("203.0.113.5"), accept_terms: true, accept_privacy: true, marketing: false }
}
struct Fx { sender: RecordingSender, captcha: ScriptedCaptcha }
impl Fx { fn new() -> Self { Fx { sender: RecordingSender::default(), captcha: ScriptedCaptcha::default() } } }

#[test]
fn full_journey_register_verify_activate_login() {
    let fx = Fx::new();
    let mut s = Server::new(cfg(), &fx.sender, &fx.captcha).unwrap();
    let r = s.register(&req("elodie@exemple.ca", "ok-1"), T0).unwrap();
    assert_eq!((r.delivery, r.retry_after), (Delivery::Sent, 60));
    let mail = fx.sender.last_to("elodie@exemple.ca").unwrap();
    assert_eq!(mail.kind, MailKind::Verification);
    let code = code_in(&mail);
    assert!(s.account_exists("elodie@exemple.ca") && !s.is_active("elodie@exemple.ca"), "créé mais NON actif");

    // interdit d'activer / de se connecter sans vérification, même avec le bon mot de passe
    let e = s.login("elodie@exemple.ca", PW, None, "Portable", None, Some("203.0.113.5"), T0 + 5).unwrap_err();
    assert_eq!(known(&e), "auth.email_not_verified");
    // jeton d'inscription requis : le code seul ne suffit pas
    assert_eq!(known(&s.verify_email("elodie@exemple.ca", "mauvais-jeton", &code, None, T0 + 6).unwrap_err()), "code.invalid");
    // mauvais code
    let wrong = if code == "000000" { "111111" } else { "000000" };
    assert_eq!(known(&s.verify_email("elodie@exemple.ca", &r.signup_token, wrong, None, T0 + 7).unwrap_err()), "code.wrong");
    // bon code + jeton
    let a = s.verify_email("elodie@exemple.ca", &r.signup_token, &code, None, T0 + 30).unwrap();
    assert_eq!(a.recovery_codes.len(), 10);
    assert!(s.is_active("elodie@exemple.ca"));
    assert_eq!(fx.sender.last_to("elodie@exemple.ca").unwrap().kind, MailKind::Welcome);
    // le code est à usage unique
    assert!(s.verify_email("elodie@exemple.ca", &r.signup_token, &code, None, T0 + 31).is_err());
    let l = s.login("elodie@exemple.ca", PW, None, "Portable", None, Some("203.0.113.5"), T0 + 40).unwrap();
    assert!(!l.token.is_empty() && !l.must_accept_terms);
}

#[test]
fn captcha_is_checked_server_side_before_anything_else() {
    let fx = Fx::new();
    let mut s = Server::new(cfg(), &fx.sender, &fx.captcha).unwrap();
    assert_eq!(known(&s.register(&req("a@exemple.ca", ""), T0).unwrap_err()), "captcha.required");
    assert_eq!(known(&s.register(&req("a@exemple.ca", "nimporte-quoi"), T0).unwrap_err()), "captcha.failed");
    assert!(!s.account_exists("a@exemple.ca") && fx.sender.sent.borrow().is_empty(), "aucun compte, aucun courriel sans preuve anti-robot");
    assert!(fx.captcha.calls.get() >= 2, "le vérificateur a bien été interrogé");
    // même avec des conditions refusées, le CAPTCHA passe en premier (aucune logique de compte avant)
    let mut r = req("a@exemple.ca", "");
    r.accept_terms = false;
    assert_eq!(known(&s.register(&r, T0).unwrap_err()), "captcha.required");
    // la récupération est protégée aussi
    assert_eq!(known(&s.request_recovery("a@exemple.ca", "", Some("203.0.113.5"), T0).unwrap_err()), "captcha.required");
    assert_eq!(known(&s.request_recovery("a@exemple.ca", "faux", Some("203.0.113.5"), T0).unwrap_err()), "captcha.failed");
}

struct Down;
impl CaptchaVerifier for Down { fn verify(&self, _: &str, _: Option<&str>) -> Result<(), CaptchaError> { Err(CaptchaError::Unavailable) } fn is_simulated(&self) -> bool { false } }
#[test]
fn captcha_provider_outage_fails_closed_with_translated_key() {
    let (sender, down) = (RecordingSender::default(), Down);
    let mut s = Server::new(cfg(), &sender, &down).unwrap();
    let e = s.register(&req("a@exemple.ca", "ok-1"), T0).unwrap_err();
    assert_eq!(known(&e), "captcha.unavailable");
    assert!(!s.account_exists("a@exemple.ca"));
}

#[test]
fn suspicious_logins_require_a_real_captcha() {
    let fx = Fx::new();
    let mut s = Server::new(cfg(), &fx.sender, &fx.captcha).unwrap();
    let r = s.register(&req("a@exemple.ca", "ok-1"), T0).unwrap();
    s.verify_email("a@exemple.ca", &r.signup_token, &code_in(&fx.sender.last_to("a@exemple.ca").unwrap()), None, T0 + 1).unwrap();
    let ip = Some("198.51.100.7");
    for i in 0..2 { assert_eq!(known(&s.login("a@exemple.ca", "mauvais", None, "d", None, ip, T0 + 10 + i).unwrap_err()), "auth.invalid"); }
    assert_eq!(known(&s.login("a@exemple.ca", PW, None, "d", None, ip, T0 + 20).unwrap_err()), "captcha.required", "après 2 échecs : preuve anti-robot exigée, même avec le bon mot de passe");
    assert_eq!(known(&s.login("a@exemple.ca", PW, None, "d", Some("pas-ok"), ip, T0 + 21).unwrap_err()), "captcha.failed");
    assert!(s.login("a@exemple.ca", PW, None, "d", Some("ok-3"), ip, T0 + 400).is_ok());
}

#[test]
fn acceptance_of_terms_and_privacy_is_mandatory_and_recorded() {
    let fx = Fx::new();
    let mut s = Server::new(cfg(), &fx.sender, &fx.captcha).unwrap();
    let mut r = req("a@exemple.ca", "ok-1");
    r.accept_terms = false;
    assert_eq!(known(&s.register(&r, T0).unwrap_err()), "terms.terms_required");
    r.accept_terms = true; r.accept_privacy = false;
    assert_eq!(known(&s.register(&r, T0).unwrap_err()), "terms.privacy_required");
    assert!(!s.account_exists("a@exemple.ca"));
    r.accept_privacy = true;
    s.register(&r, T0 + 5).unwrap();
    let c = s.consents_of("a@exemple.ca").unwrap();
    assert_eq!((c.acceptances[0].terms_version.as_str(), c.acceptances[0].privacy_version.as_str(), c.acceptances[0].accepted_at), ("cgu-1", "conf-1", T0 + 5));
    assert!(!c.marketing_granted(), "marketing décoché par défaut et distinct");
    // marketing coché explicitement : événement séparé et daté
    let mut r2 = req("b@exemple.ca", "ok-2"); r2.marketing = true;
    s.register(&r2, T0 + 6).unwrap();
    let c2 = s.consents_of("b@exemple.ca").unwrap();
    assert!(c2.marketing_granted() && c2.marketing[0].text_version == "mkt-1" && c2.marketing[0].at == T0 + 6);
}

#[test]
fn marketing_can_be_withdrawn_and_new_terms_require_reacceptance() {
    let fx = Fx::new();
    let mut s = Server::new(cfg(), &fx.sender, &fx.captcha).unwrap();
    let mut r = req("a@exemple.ca", "ok-1"); r.marketing = true;
    let reg = s.register(&r, T0).unwrap();
    s.verify_email("a@exemple.ca", &reg.signup_token, &code_in(&fx.sender.last_to("a@exemple.ca").unwrap()), None, T0 + 1).unwrap();
    let l = s.login("a@exemple.ca", PW, None, "d", None, None, T0 + 2).unwrap();
    s.set_marketing(&l.token, false, T0 + 3).unwrap();
    let c = s.consents_of("a@exemple.ca").unwrap();
    assert_eq!(c.marketing.iter().map(|e| e.granted).collect::<Vec<_>>(), vec![true, false], "historique conservé");
    // nouvelle version des conditions
    let mut s2 = Server::new(Config { legal: legal("cgu-2"), ..cfg() }, &fx.sender, &fx.captcha).unwrap();
    let dir = std::env::temp_dir().join(format!("nrl-srv-terms-{}", std::process::id()));
    s.save(&dir).unwrap(); s2.load(&dir).unwrap();
    let l2 = s2.login("a@exemple.ca", PW, None, "d2", None, None, T0 + 10).unwrap();
    assert!(l2.must_accept_terms);
    s2.accept_current_terms(&l2.token, T0 + 11).unwrap();
    assert!(!s2.login("a@exemple.ca", PW, None, "d2", None, None, T0 + 12).unwrap().must_accept_terms);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn code_expiry_resend_cooldown_invalidation_and_limits() {
    let fx = Fx::new();
    let mut s = Server::new(cfg(), &fx.sender, &fx.captcha).unwrap();
    let r = s.register(&req("a@exemple.ca", "ok-1"), T0).unwrap();
    let c1 = code_in(&fx.sender.last_to("a@exemple.ca").unwrap());
    let e = s.resend_code("a@exemple.ca", &r.signup_token, None, T0 + 30).unwrap_err();
    assert_eq!((known(&e), e.retry_after), ("code.cooldown", Some(30)), "renvoi impossible avant 60 s");
    assert_eq!(fx.sender.sent.borrow().len(), 1);
    s.resend_code("a@exemple.ca", &r.signup_token, None, T0 + 61).unwrap();
    let c2 = code_in(&fx.sender.last_to("a@exemple.ca").unwrap());
    assert_eq!(fx.sender.sent.borrow().len(), 2);
    if c1 != c2 { assert_eq!(known(&s.verify_email("a@exemple.ca", &r.signup_token, &c1, None, T0 + 62).unwrap_err()), "code.wrong", "l'ancien code est invalidé"); }
    // expiration à 10 minutes
    let e = s.verify_email("a@exemple.ca", &r.signup_token, &c2, None, T0 + 61 + 600).unwrap_err();
    assert_eq!(known(&e), "code.expired");
    // un nouvel envoi redonne un code valide
    s.resend_code("a@exemple.ca", &r.signup_token, None, T0 + 800).unwrap();
    let c3 = code_in(&fx.sender.last_to("a@exemple.ca").unwrap());
    // limite d'essais : 5 erreurs, puis même le bon code est refusé
    let wrong = if c3 == "000000" { "111111" } else { "000000" };
    for _ in 0..5 { let _ = s.verify_email("a@exemple.ca", &r.signup_token, wrong, None, T0 + 801); }
    assert_eq!(known(&s.verify_email("a@exemple.ca", &r.signup_token, &c3, None, T0 + 802).unwrap_err()), "code.too_many_attempts");
    // limite horaire des renvois
    let mut t = T0 + 900; let mut last = Ok(0);
    for _ in 0..6 { t += 61; last = s.resend_code("a@exemple.ca", &r.signup_token, None, t); if last.is_err() { break; } }
    assert_eq!(known(&last.unwrap_err()), "code.hourly_limit");
}

#[test]
fn address_can_be_corrected_before_validation() {
    let fx = Fx::new();
    let mut s = Server::new(cfg(), &fx.sender, &fx.captcha).unwrap();
    let r = s.register(&req("elodie@exmple.ca", "ok-1"), T0).unwrap(); // faute de frappe
    let typo_code = code_in(&fx.sender.last_to("elodie@exmple.ca").unwrap());
    s.change_pending_email("elodie@exmple.ca", &r.signup_token, "elodie@exemple.ca", None, T0 + 61).unwrap();
    let code = code_in(&fx.sender.last_to("elodie@exemple.ca").unwrap());
    assert!(!s.account_exists("elodie@exmple.ca") && s.account_exists("elodie@exemple.ca"));
    assert_eq!(known(&s.verify_email("elodie@exmple.ca", &r.signup_token, &typo_code, None, T0 + 62).unwrap_err()), "code.invalid", "l'ancienne adresse ne peut plus activer");
    assert_eq!(known(&s.verify_email("elodie@exemple.ca", &r.signup_token, if typo_code == code { "000000" } else { &typo_code }, None, T0 + 63).unwrap_err()), "code.wrong");
    s.verify_email("elodie@exemple.ca", &r.signup_token, &code, None, T0 + 64).unwrap();
    assert!(s.is_active("elodie@exemple.ca"));
    assert!(fx.sender.last_to("elodie@exmple.ca").unwrap().kind == MailKind::Verification, "l'ancienne adresse n'a reçu que le premier code");
    // nombre de corrections limité tant que le compte n'est pas vérifié
    let r2 = s.register(&req("x0@exemple.ca", "ok-1"), T0 + 100).unwrap();
    let mut cur = "x0@exemple.ca".to_string(); let mut t = T0 + 100; let mut res = Ok(0);
    for i in 1..=4 { t += 61; let n = format!("x{i}@exemple.ca"); res = s.change_pending_email(&cur, &r2.signup_token, &n, None, t); if res.is_ok() { cur = n; } }
    assert_eq!(known(&res.unwrap_err()), "rate.too_many_requests");
}

#[test]
fn pre_hijacking_is_blocked_by_the_signup_token() {
    let fx = Fx::new();
    let mut s = Server::new(cfg(), &fx.sender, &fx.captcha).unwrap();
    // l'attaquant inscrit l'adresse de la victime avec son propre mot de passe
    let mut atk = req("victime@exemple.ca", "ok-1"); atk.password = "Attaquant-Mot-De-Passe-77!";
    let reg = s.register(&atk, T0).unwrap();
    // la victime reçoit un code ; elle ne possède pas le jeton → ne peut pas activer le compte de l'attaquant
    let code = code_in(&fx.sender.last_to("victime@exemple.ca").unwrap());
    assert_eq!(known(&s.verify_email("victime@exemple.ca", "jeton-de-la-victime", &code, None, T0 + 5).unwrap_err()), "code.invalid");
    // l'attaquant ne connaît pas le code
    let wrong = if code == "000000" { "111111" } else { "000000" };
    assert_eq!(known(&s.verify_email("victime@exemple.ca", &reg.signup_token, wrong, None, T0 + 6).unwrap_err()), "code.wrong");
    assert!(!s.is_active("victime@exemple.ca"));
    assert!(s.login("victime@exemple.ca", "Attaquant-Mot-De-Passe-77!", None, "d", None, None, T0 + 7).is_err(), "aucune connexion sans activation");
}

#[test]
fn no_account_enumeration() {
    let fx = Fx::new();
    let mut s = Server::new(cfg(), &fx.sender, &fx.captcha).unwrap();
    let r = s.register(&req("a@exemple.ca", "ok-1"), T0).unwrap();
    s.verify_email("a@exemple.ca", &r.signup_token, &code_in(&fx.sender.last_to("a@exemple.ca").unwrap()), None, T0 + 1).unwrap();
    // inscription d'une adresse déjà active : réponse de même forme, le titulaire est averti, aucun second compte
    let again = s.register(&req("a@exemple.ca", "ok-2"), T0 + 100).unwrap();
    assert_eq!((again.retry_after, again.delivery), (60, Delivery::Sent));
    assert_eq!(fx.sender.last_to("a@exemple.ca").unwrap().kind, MailKind::Security);
    // récupération : même réponse pour compte inconnu et connu
    let known_r = s.request_recovery("a@exemple.ca", "ok-3", Some("1.1.1.1"), T0 + 200).unwrap();
    let unknown_r = s.request_recovery("inconnu@exemple.ca", "ok-4", Some("1.1.1.1"), T0 + 201).unwrap();
    assert_eq!(known_r, unknown_r);
    assert!(fx.sender.last_to("inconnu@exemple.ca").is_none());
    // connexion : même erreur
    let e1 = s.login("a@exemple.ca", "mauvais", None, "d", None, None, T0 + 300).unwrap_err();
    let e2 = s.login("inconnu@exemple.ca", "mauvais", None, "d", None, None, T0 + 301).unwrap_err();
    assert_eq!((known(&e1), known(&e2)), ("auth.invalid", "auth.invalid"));
    // renvoi pour un compte inconnu : même réponse, rien d'envoyé
    assert_eq!(s.resend_code("inconnu@exemple.ca", "x", None, T0 + 400).unwrap(), 60);
}

#[test]
fn email_change_requires_verification_of_the_new_address() {
    let fx = Fx::new();
    let mut s = Server::new(cfg(), &fx.sender, &fx.captcha).unwrap();
    let r = s.register(&req("old@exemple.ca", "ok-1"), T0).unwrap();
    s.verify_email("old@exemple.ca", &r.signup_token, &code_in(&fx.sender.last_to("old@exemple.ca").unwrap()), None, T0 + 1).unwrap();
    let l = s.login("old@exemple.ca", PW, None, "Portable", None, None, T0 + 2).unwrap();
    assert_eq!(known(&s.request_email_change(&l.token, "mauvais-mot-de-passe", "new@exemple.ca", T0 + 3).unwrap_err()), "auth.invalid", "mot de passe requis");
    s.request_email_change(&l.token, PW, "new@exemple.ca", T0 + 4).unwrap();
    let code = code_in(&fx.sender.last_to("new@exemple.ca").unwrap());
    assert_eq!(fx.sender.last_to("old@exemple.ca").unwrap().kind, MailKind::Security, "l'ancienne adresse est avertie");
    // tant que non vérifié : l'adresse n'a pas changé
    assert!(s.is_active("old@exemple.ca") && !s.account_exists("new@exemple.ca"));
    assert!(s.login("new@exemple.ca", PW, None, "d", None, None, T0 + 5).is_err());
    assert_eq!(known(&s.confirm_email_change(&l.token, if code == "000000" { "111111" } else { "000000" }, T0 + 6).unwrap_err()), "code.wrong");
    s.confirm_email_change(&l.token, &code, T0 + 7).unwrap();
    assert!(s.is_active("new@exemple.ca") && !s.account_exists("old@exemple.ca"));
    assert!(s.login("old@exemple.ca", PW, None, "d", None, None, T0 + 8).is_err());
    assert!(s.login("new@exemple.ca", PW, None, "d", None, None, T0 + 9).is_ok());
    assert!(s.devices(&l.token, T0 + 10).is_err(), "anciennes sessions révoquées après changement d'adresse");
}

#[test]
fn password_recovery_journey() {
    let fx = Fx::new();
    let mut s = Server::new(cfg(), &fx.sender, &fx.captcha).unwrap();
    let r = s.register(&req("a@exemple.ca", "ok-1"), T0).unwrap();
    s.verify_email("a@exemple.ca", &r.signup_token, &code_in(&fx.sender.last_to("a@exemple.ca").unwrap()), None, T0 + 1).unwrap();
    let l = s.login("a@exemple.ca", PW, None, "Portable", None, None, T0 + 2).unwrap();
    s.request_recovery("a@exemple.ca", "ok-9", Some("9.9.9.9"), T0 + 100).unwrap();
    let m = fx.sender.last_to("a@exemple.ca").unwrap();
    assert_eq!(m.kind, MailKind::PasswordRecovery);
    let code = code_in(&m);
    let new = "Nouveau-Mot-De-Passe-2026!";
    assert_eq!(known(&s.recover("a@exemple.ca", "999999", new, None, None, T0 + 110).unwrap_err()), "code.wrong");
    assert_eq!(known(&s.recover("a@exemple.ca", &code, "court", None, None, T0 + 111).unwrap_err()), "error.password.weak");
    s.recover("a@exemple.ca", &code, new, None, None, T0 + 112).unwrap();
    assert_eq!(fx.sender.last_to("a@exemple.ca").unwrap().kind, MailKind::Security);
    assert!(s.login("a@exemple.ca", PW, None, "d", None, None, T0 + 120).is_err(), "l'ancien mot de passe ne fonctionne plus");
    assert!(s.login("a@exemple.ca", new, None, "d", None, None, T0 + 130).is_ok());
    assert!(s.devices(&l.token, T0 + 140).is_err(), "sessions révoquées");
    assert_eq!(known(&s.recover("a@exemple.ca", &code, "Encore-Un-Mot-De-Passe-77!", None, None, T0 + 150).unwrap_err()), "code.invalid", "le code de récupération est à usage unique");
}

#[test]
fn two_factor_is_optional_and_email_alone_cannot_bypass_it() {
    use norelias_account::totp;
    let fx = Fx::new();
    let mut s = Server::new(cfg(), &fx.sender, &fx.captcha).unwrap();
    let r = s.register(&req("a@exemple.ca", "ok-1"), T0).unwrap();
    let act = s.verify_email("a@exemple.ca", &r.signup_token, &code_in(&fx.sender.last_to("a@exemple.ca").unwrap()), None, T0 + 1).unwrap();
    let l = s.login("a@exemple.ca", PW, None, "Portable", None, None, T0 + 2).unwrap(); // sans 2FA : facultatif
    let (secret_b32, _uri) = s.begin_two_factor(&l.token, PW, T0 + 3).unwrap();
    let secret = totp::base32_decode(&secret_b32).unwrap();
    s.confirm_two_factor(&l.token, PW, &totp::code_for_step(&secret, totp::step_at(T0 + 4), 6), T0 + 4).unwrap();
    assert_eq!(fx.sender.last_to("a@exemple.ca").unwrap().kind, MailKind::Security);
    assert_eq!(known(&s.login("a@exemple.ca", PW, None, "Portable", None, None, T0 + 100).unwrap_err()), "auth.totp_required");
    let c = totp::code_for_step(&secret, totp::step_at(T0 + 100), 6);
    assert!(s.login("a@exemple.ca", PW, Some(&c), "Portable", None, None, T0 + 100).is_ok());
    // récupération : le code reçu par courriel ne suffit PAS quand le 2FA est actif
    s.request_recovery("a@exemple.ca", "ok-5", Some("5.5.5.5"), T0 + 200).unwrap();
    let rc = code_in(&fx.sender.last_to("a@exemple.ca").unwrap());
    let new = "Nouveau-Mot-De-Passe-2026!";
    assert_eq!(known(&s.recover("a@exemple.ca", &rc, new, None, None, T0 + 201).unwrap_err()), "recovery.needs_second_factor");
    s.request_recovery("a@exemple.ca", "ok-6", Some("5.5.5.5"), T0 + 300).unwrap();
    let rc2 = code_in(&fx.sender.last_to("a@exemple.ca").unwrap());
    s.recover("a@exemple.ca", &rc2, new, Some(&act.recovery_codes[0]), None, T0 + 301).unwrap();
    assert!(s.login("a@exemple.ca", new, None, "d", None, None, T0 + 400).is_ok(), "avec un code de récupération, le 2FA est réinitialisé");
}

#[test]
fn delivery_failures_are_handled() {
    // erreur temporaire : compte créé en attente, courriel renvoyé automatiquement
    let fx = Fx::new();
    fx.sender.script.borrow_mut().push(SendError::Transient("fournisseur indisponible".into()));
    let mut s = Server::new(cfg(), &fx.sender, &fx.captcha).unwrap();
    let r = s.register(&req("a@exemple.ca", "ok-1"), T0).unwrap();
    assert_eq!((r.delivery, s.pending_mail()), (Delivery::Queued, 1));
    assert!(fx.sender.sent.borrow().is_empty());
    assert!(s.pump(T0 + 30).is_empty() && s.pending_mail() == 1, "pas avant le délai de reprise");
    let ev = s.pump(T0 + 61);
    assert_eq!(ev.len(), 1);
    assert_eq!(fx.sender.last_to("a@exemple.ca").unwrap().kind, MailKind::Verification);
    // erreur définitive (adresse inexistante) : message clair, aucun compte fantôme
    let fx2 = Fx::new();
    fx2.sender.script.borrow_mut().push(SendError::Permanent("boîte inexistante".into()));
    let mut s2 = Server::new(cfg(), &fx2.sender, &fx2.captcha).unwrap();
    let e = s2.register(&req("faute@exemple.ca", "ok-1"), T0).unwrap_err();
    assert_eq!(known(&e), "email.undeliverable");
    assert!(!s2.account_exists("faute@exemple.ca") && s2.pending_mail() == 0);
    s2.register(&req("faute@exemple.ca", "ok-1"), T0 + 1).unwrap();
}

struct RealLooking;
impl EmailSender for RealLooking { fn send(&self, _: &Message) -> Result<String, SendError> { Ok("id".into()) } fn is_simulated(&self) -> bool { false } }
struct RealCaptchaLooking;
impl CaptchaVerifier for RealCaptchaLooking { fn verify(&self, _: &str, _: Option<&str>) -> Result<(), CaptchaError> { Ok(()) } fn is_simulated(&self) -> bool { false } }

#[test]
fn nothing_simulated_in_production() {
    let prod = |secret: &[u8], url: &str| Config { env: Environment::Production, secret: secret.to_vec(), brand: Brand { base_url: url.into(), ..Brand::default() }, ..cfg() };
    let (sim_mail, sim_cap, real_mail, real_cap) = (RecordingSender::default(), ScriptedCaptcha::default(), RealLooking, RealCaptchaLooking);
    assert_eq!(Server::new(prod(SECRET, "https://norelias.ca"), &sim_mail, &real_cap).err(), Some(ConfigError::SimulatedEmailInProduction));
    assert_eq!(Server::new(prod(SECRET, "https://norelias.ca"), &real_mail, &sim_cap).err(), Some(ConfigError::SimulatedCaptchaInProduction));
    assert_eq!(Server::new(prod(&[0u8; 32], "https://norelias.ca"), &real_mail, &real_cap).err(), Some(ConfigError::WeakSecret));
    assert_eq!(Server::new(prod(b"court", "https://norelias.ca"), &real_mail, &real_cap).err(), Some(ConfigError::WeakSecret));
    assert_eq!(Server::new(prod(SECRET, "http://norelias.ca"), &real_mail, &real_cap).err(), Some(ConfigError::InsecureBaseUrl));
    assert!(Server::new(prod(SECRET, "https://norelias.ca"), &real_mail, &real_cap).is_ok());
    // en développement, les doubles sont permis
    assert!(Server::new(cfg(), &sim_mail, &sim_cap).is_ok());
}

#[test]
fn abuse_limits_per_ip() {
    let fx = Fx::new();
    let mut s = Server::new(cfg(), &fx.sender, &fx.captcha).unwrap();
    for i in 0..5 { s.register(&req(&format!("u{i}@exemple.ca"), "ok-1"), T0 + i).unwrap(); }
    let e = s.register(&req("u5@exemple.ca", "ok-1"), T0 + 10).unwrap_err();
    assert_eq!(known(&e), "rate.too_many_requests");
    assert!(e.retry_after.unwrap() > 0 && !s.account_exists("u5@exemple.ca"));
    let mut other = req("u5@exemple.ca", "ok-1"); other.ip = Some("198.51.100.99");
    assert!(s.register(&other, T0 + 11).is_ok(), "une autre adresse IP n'est pas affectée");
    // attaque de vérification par force brute : plafond par IP
    let r = s.register(&{ let mut q = req("v@exemple.ca", "ok-1"); q.ip = Some("192.0.2.1"); q }, T0 + 12).unwrap();
    let mut blocked = false;
    for i in 0..40 { if s.verify_email("v@exemple.ca", &r.signup_token, "000000", Some("192.0.2.77"), T0 + 20 + i).map_err(|e| e.key == "rate.too_many_requests").err() == Some(true) { blocked = true; break; } }
    assert!(blocked);
}

#[test]
fn new_device_alert_and_device_management() {
    let fx = Fx::new();
    let mut s = Server::new(cfg(), &fx.sender, &fx.captcha).unwrap();
    let r = s.register(&req("a@exemple.ca", "ok-1"), T0).unwrap();
    s.verify_email("a@exemple.ca", &r.signup_token, &code_in(&fx.sender.last_to("a@exemple.ca").unwrap()), None, T0 + 1).unwrap();
    let a = s.login("a@exemple.ca", PW, None, "MacBook", None, None, T0 + 2).unwrap();
    let n = fx.sender.sent.borrow().len();
    let b = s.login("a@exemple.ca", PW, None, "MacBook", None, None, T0 + 3).unwrap();
    assert_eq!(fx.sender.sent.borrow().len(), n, "même appareil : pas d'alerte");
    let c = s.login("a@exemple.ca", PW, None, "Téléphone", None, None, T0 + 4).unwrap();
    let alert = fx.sender.last_to("a@exemple.ca").unwrap();
    assert_eq!(alert.kind, MailKind::Security);
    assert!(alert.html.contains("Téléphone"));
    let devices = s.devices(&a.token, T0 + 5).unwrap();
    assert_eq!(devices.len(), 3);
    assert!(s.revoke_device(&a.token, &c.session_id, T0 + 6).unwrap());
    assert!(s.devices(&c.token, T0 + 7).is_err(), "appareil révoqué : plus d'accès");
    assert!(s.devices(&b.token, T0 + 8).is_ok());
}

#[test]
fn account_deletion_has_a_grace_period_and_confirmation_email() {
    let fx = Fx::new();
    let mut s = Server::new(cfg(), &fx.sender, &fx.captcha).unwrap();
    let r = s.register(&req("a@exemple.ca", "ok-1"), T0).unwrap();
    s.verify_email("a@exemple.ca", &r.signup_token, &code_in(&fx.sender.last_to("a@exemple.ca").unwrap()), None, T0 + 1).unwrap();
    let l = s.login("a@exemple.ca", PW, None, "d", None, None, T0 + 2).unwrap();
    assert!(s.request_deletion(&l.token, "mauvais", T0 + 3).is_err());
    let eff = s.request_deletion(&l.token, PW, T0 + 4).unwrap();
    assert_eq!(eff, T0 + 4 + 30 * 86_400);
    let m = fx.sender.last_to("a@exemple.ca").unwrap();
    assert_eq!(m.kind, MailKind::AccountDeletion);
    assert!(s.devices(&l.token, T0 + 5).is_err(), "sessions fermées");
    assert_eq!(s.purge_due(eff - 1), 0);
    assert_eq!(s.purge_due(eff), 1);
    assert!(!s.account_exists("a@exemple.ca"));
}

#[test]
fn persisted_state_contains_no_plaintext_secrets_and_survives_restart() {
    let fx = Fx::new();
    let mut s = Server::new(cfg(), &fx.sender, &fx.captcha).unwrap();
    let r = s.register(&req("a@exemple.ca", "ok-1"), T0).unwrap();
    let code = code_in(&fx.sender.last_to("a@exemple.ca").unwrap());
    let dir = std::env::temp_dir().join(format!("nrl-srv-state-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    s.save(&dir).unwrap();
    let raw = std::fs::read_to_string(dir.join("server_state.json")).unwrap();
    for secret in [PW, r.signup_token.as_str()] { assert!(!raw.contains(secret), "secret en clair dans l'état persisté"); }
    assert!(!raw.contains(&format!("\"{code}\"")), "le code n'est pas stocké en clair");
    #[cfg(unix)] { use std::os::unix::fs::PermissionsExt; assert_eq!(std::fs::metadata(dir.join("server_state.json")).unwrap().permissions().mode() & 0o777, 0o600); }
    // redémarrage : l'activation continue de fonctionner
    let mut s2 = Server::new(cfg(), &fx.sender, &fx.captcha).unwrap();
    s2.load(&dir).unwrap();
    assert!(s2.verify_email("a@exemple.ca", &r.signup_token, &code, None, T0 + 20).is_ok());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn every_error_key_has_a_translation_in_both_languages() {
    let cat = norelias_i18n::Catalog::builtin();
    for k in ERROR_KEYS { assert!(cat.has_key(k), "clé sans traduction : {k}"); for lang in ["fr", "en"] { assert_ne!(cat.t(&norelias_i18n::Locale::new(lang, None), k), *k, "{k} non traduite en {lang}"); } }
    assert!(cat.audit().is_empty(), "{:?}", cat.audit());
}
