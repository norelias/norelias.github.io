//! Envoi de courriels transactionnels : interface `EmailSender`, constructeurs de requêtes pour deux fournisseurs,
//! classification des erreurs de livraison, file d'envoi avec reprises. **Aucun fournisseur n'est contacté ici** :
//! le transport HTTP réel est fourni par le binaire serveur (`HttpTransport`).
//!
//! Les formats d'API des fournisseurs ci-dessous sont écrits d'après leur documentation publique telle que rappelée,
//! **sans vérification contre les services réels** (aucun compte ni accès réseau dans cet environnement) : ils doivent être
//! validés par un envoi réel avant toute mise en production.

use serde::Serialize;
use std::cell::RefCell;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum MailKind { Verification, Welcome, PasswordRecovery, Security, Billing, AccountDeletion, EmailChangeNotice }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message { pub to: String, pub subject: String, pub html: String, pub text: String, pub kind: MailKind }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SendError {
    /// Erreur temporaire (réseau, 5xx, limite de débit) : on réessaie.
    Transient(String),
    /// Erreur définitive (adresse invalide, rebond, refus du fournisseur) : on ne réessaie pas.
    Permanent(String),
}

pub trait EmailSender {
    fn send(&self, m: &Message) -> Result<String, SendError>;
    /// Vrai pour tout expéditeur fictif ; **interdit en production**.
    fn is_simulated(&self) -> bool;
}

// ───────── Transport HTTP (à implémenter par le serveur) ─────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequest { pub url: String, pub headers: Vec<(String, String)>, pub body: String, pub content_type: &'static str }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse { pub status: u16, pub body: String }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportError(pub String);
pub trait HttpTransport { fn post(&self, req: &HttpRequest) -> Result<HttpResponse, TransportError>; }

/// 2xx = succès ; 408/425/429/5xx = temporaire ; autres 4xx = définitif.
pub fn classify_status(status: u16) -> Result<(), SendError> {
    match status { 200..=299 => Ok(()), 408 | 425 | 429 | 500..=599 => Err(SendError::Transient(format!("HTTP {status}"))), _ => Err(SendError::Permanent(format!("HTTP {status}"))) }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Provider { Postmark, Resend }

pub fn build_request(p: Provider, api_key: &str, from: &str, m: &Message) -> HttpRequest {
    match p {
        Provider::Postmark => HttpRequest { url: "https://api.postmarkapp.com/email".into(), content_type: "application/json",
            headers: vec![("X-Postmark-Server-Token".into(), api_key.into()), ("Accept".into(), "application/json".into())],
            body: serde_json::json!({"From": from, "To": m.to, "Subject": m.subject, "HtmlBody": m.html, "TextBody": m.text, "MessageStream": "outbound"}).to_string() },
        Provider::Resend => HttpRequest { url: "https://api.resend.com/emails".into(), content_type: "application/json",
            headers: vec![("Authorization".into(), format!("Bearer {api_key}"))],
            body: serde_json::json!({"from": from, "to": [m.to], "subject": m.subject, "html": m.html, "text": m.text}).to_string() },
    }
}

/// Expéditeur réel : nécessite un fournisseur, une clé d'API et un transport HTTP.
pub struct ProviderSender<T: HttpTransport> { pub provider: Provider, pub api_key: String, pub from: String, pub transport: T }
impl<T: HttpTransport> EmailSender for ProviderSender<T> {
    fn send(&self, m: &Message) -> Result<String, SendError> {
        let r = self.transport.post(&build_request(self.provider, &self.api_key, &self.from, m)).map_err(|e| SendError::Transient(e.0))?;
        classify_status(r.status)?;
        let v: serde_json::Value = serde_json::from_str(&r.body).unwrap_or_default();
        Ok(v.get("MessageID").or_else(|| v.get("id")).and_then(|x| x.as_str()).unwrap_or("").to_string())
    }
    fn is_simulated(&self) -> bool { false }
}

/// Expéditeur **fictif** pour les tests et le développement : enregistre les messages. Refusé en production.
#[derive(Default)]
pub struct RecordingSender { pub sent: RefCell<Vec<Message>>, /// Erreurs à produire aux prochains envois (la plus ancienne d'abord).
    pub script: RefCell<Vec<SendError>> }
impl EmailSender for RecordingSender {
    fn send(&self, m: &Message) -> Result<String, SendError> {
        if !self.script.borrow().is_empty() { return Err(self.script.borrow_mut().remove(0)); }
        self.sent.borrow_mut().push(m.clone());
        Ok(format!("sim-{}", self.sent.borrow().len()))
    }
    fn is_simulated(&self) -> bool { true }
}
impl RecordingSender {
    pub fn last_to(&self, to: &str) -> Option<Message> { self.sent.borrow().iter().rev().find(|m| m.to == to).cloned() }
}

// ───────── File d'envoi ─────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State { Pending, Sent(String), Failed(String), Undeliverable(String) }
#[derive(Debug, Clone)]
pub struct Item { pub id: u64, pub msg: Message, pub attempts: u32, pub next_try: u64, pub state: State }
#[derive(Debug, PartialEq, Eq)]
pub struct Event { pub id: u64, pub to: String, pub kind: MailKind, pub state: State }

pub const BACKOFF: [u64; 4] = [60, 300, 1800, 7200];
pub const MAX_ATTEMPTS: u32 = 5;

/// File **en mémoire uniquement** : les messages contiennent des codes, ils ne sont jamais écrits sur disque.
/// En cas de plantage, l'utilisateur redemande un code (renvoi possible après 60 s).
#[derive(Default)]
pub struct Outbox { items: Vec<Item>, next_id: u64 }
impl Outbox {
    pub fn push(&mut self, msg: Message, now: u64) -> u64 { self.next_id += 1; self.items.push(Item { id: self.next_id, msg, attempts: 0, next_try: now, state: State::Pending }); self.next_id }
    /// Tente les envois dus. Les succès sont retirés de la file (le code n'y reste pas).
    pub fn process(&mut self, sender: &dyn EmailSender, now: u64) -> Vec<Event> {
        let mut events = vec![];
        for it in self.items.iter_mut().filter(|i| i.state == State::Pending && i.next_try <= now) {
            it.attempts += 1;
            match sender.send(&it.msg) {
                Ok(pid) => it.state = State::Sent(pid),
                Err(SendError::Permanent(r)) => it.state = State::Undeliverable(r),
                Err(SendError::Transient(r)) => {
                    if it.attempts >= MAX_ATTEMPTS { it.state = State::Failed(r); } else { it.next_try = now + BACKOFF[(it.attempts as usize - 1).min(BACKOFF.len() - 1)]; }
                }
            }
            if it.state != State::Pending { events.push(Event { id: it.id, to: it.msg.to.clone(), kind: it.msg.kind, state: it.state.clone() }); }
        }
        self.items.retain(|i| i.state == State::Pending);
        events
    }
    pub fn pending(&self) -> usize { self.items.len() }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn msg() -> Message { Message { to: "a@b.co".into(), subject: "S".into(), html: "<p>Code 123456</p>".into(), text: "Code 123456".into(), kind: MailKind::Verification } }

    #[test]
    fn status_classification() {
        assert!(classify_status(200).is_ok() && classify_status(202).is_ok());
        for s in [408, 429, 500, 503] { assert!(matches!(classify_status(s), Err(SendError::Transient(_))), "{s}"); }
        for s in [400, 401, 403, 422] { assert!(matches!(classify_status(s), Err(SendError::Permanent(_))), "{s}"); }
    }
    #[test]
    fn request_shapes_for_each_provider() {
        let p = build_request(Provider::Postmark, "KEY", "Norélias <no-reply@mail.norelias.ca>", &msg());
        assert_eq!(p.url, "https://api.postmarkapp.com/email");
        assert!(p.headers.iter().any(|h| h.0 == "X-Postmark-Server-Token" && h.1 == "KEY"));
        let b: serde_json::Value = serde_json::from_str(&p.body).unwrap();
        assert_eq!((b["To"].as_str(), b["Subject"].as_str()), (Some("a@b.co"), Some("S")));
        let r = build_request(Provider::Resend, "KEY", "x@y.co", &msg());
        assert!(r.headers.iter().any(|h| h.0 == "Authorization" && h.1 == "Bearer KEY"));
        assert_eq!(serde_json::from_str::<serde_json::Value>(&r.body).unwrap()["to"][0], "a@b.co");
        assert!(!p.body.contains("KEY") && !r.body.contains("KEY"), "la clé n'est jamais dans le corps");
    }
    struct Fake(RefCell<Vec<Result<HttpResponse, TransportError>>>);
    impl HttpTransport for Fake { fn post(&self, _: &HttpRequest) -> Result<HttpResponse, TransportError> { self.0.borrow_mut().remove(0) } }
    #[test]
    fn provider_sender_maps_responses_and_transport_errors() {
        let mk = |r| ProviderSender { provider: Provider::Resend, api_key: "k".into(), from: "f@x.co".into(), transport: Fake(RefCell::new(vec![r])) };
        assert_eq!(mk(Ok(HttpResponse { status: 200, body: r#"{"id":"abc"}"#.into() })).send(&msg()), Ok("abc".into()));
        assert!(matches!(mk(Ok(HttpResponse { status: 422, body: "{}".into() })).send(&msg()), Err(SendError::Permanent(_))));
        assert!(matches!(mk(Ok(HttpResponse { status: 503, body: "".into() })).send(&msg()), Err(SendError::Transient(_))));
        assert!(matches!(mk(Err(TransportError("timeout".into()))).send(&msg()), Err(SendError::Transient(_))));
        assert!(!mk(Err(TransportError("x".into()))).is_simulated() && RecordingSender::default().is_simulated());
    }
    #[test]
    fn outbox_retries_with_backoff_then_succeeds_and_forgets_the_code() {
        let s = RecordingSender::default();
        s.script.borrow_mut().extend([SendError::Transient("a".into()), SendError::Transient("b".into())]);
        let mut o = Outbox::default();
        o.push(msg(), 0);
        assert!(o.process(&s, 0).is_empty() && o.pending() == 1);
        assert!(o.process(&s, 30).is_empty(), "pas avant le délai");
        assert!(o.process(&s, 60).is_empty());
        let ev = o.process(&s, 60 + 300);
        assert_eq!(ev.len(), 1);
        assert!(matches!(ev[0].state, State::Sent(_)) && o.pending() == 0, "retiré de la file une fois envoyé");
    }
    #[test]
    fn permanent_failure_is_reported_as_undeliverable_and_not_retried() {
        let s = RecordingSender::default();
        s.script.borrow_mut().push(SendError::Permanent("boîte inexistante".into()));
        let mut o = Outbox::default();
        o.push(msg(), 0);
        let ev = o.process(&s, 0);
        assert!(matches!(&ev[0].state, State::Undeliverable(r) if r.contains("inexistante")) && o.pending() == 0 && s.sent.borrow().is_empty());
    }
    #[test]
    fn too_many_transient_failures_end_as_failed() {
        let s = RecordingSender::default();
        for _ in 0..10 { s.script.borrow_mut().push(SendError::Transient("down".into())); }
        let mut o = Outbox::default();
        o.push(msg(), 0);
        let mut last = vec![]; let mut t = 0;
        for _ in 0..MAX_ATTEMPTS { last = o.process(&s, t); t += 10_000; }
        assert!(matches!(&last[0].state, State::Failed(_)) && o.pending() == 0);
    }
}
