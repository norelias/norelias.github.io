//! Sessions et appareils : jeton aléatoire de 256 bits, seul son hachage SHA-256 est conservé.

use crate::crypto::{ct_eq, hex, random_bytes, sha256_hex};
use serde::{Deserialize, Serialize};

pub const IDLE_SECONDS: u64 = 30 * 86_400;
pub const ABSOLUTE_SECONDS: u64 = 90 * 86_400;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session { pub id: String, pub device: String, pub token_hash: String, pub created: u64, pub last_seen: u64 }

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sessions { list: Vec<Session> }

impl Sessions {
    /// Crée une session et retourne le jeton **en clair, une seule fois**.
    pub fn create(&mut self, device: &str, now: u64) -> (String, String) {
        let token = hex(&random_bytes::<32>());
        let id = hex(&random_bytes::<6>());
        let device: String = device.chars().filter(|c| !c.is_control()).take(80).collect();
        self.list.push(Session { id: id.clone(), device, token_hash: sha256_hex(token.as_bytes()), created: now, last_seen: now });
        (id, token)
    }
    /// Valide un jeton ; met à jour `last_seen`. Les sessions expirées sont supprimées.
    pub fn validate(&mut self, token: &str, now: u64) -> Option<String> {
        self.list.retain(|s| now.saturating_sub(s.last_seen) <= IDLE_SECONDS && now.saturating_sub(s.created) <= ABSOLUTE_SECONDS);
        let h = sha256_hex(token.as_bytes());
        let s = self.list.iter_mut().find(|s| ct_eq(s.token_hash.as_bytes(), h.as_bytes()))?;
        s.last_seen = now;
        Some(s.id.clone())
    }
    pub fn revoke(&mut self, id: &str) -> bool { let n = self.list.len(); self.list.retain(|s| s.id != id); self.list.len() != n }
    pub fn revoke_all_except(&mut self, keep: Option<&str>) -> usize { let n = self.list.len(); self.list.retain(|s| Some(s.id.as_str()) == keep); n - self.list.len() }
    pub fn list(&self) -> &[Session] { &self.list }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn token_roundtrip_and_hash_only() {
        let mut s = Sessions::default();
        let (id, tok) = s.create("MacBook de Jean", 1000);
        assert_eq!(s.validate(&tok, 1001), Some(id));
        assert!(s.validate("faux", 1001).is_none());
        assert!(!serde_json::to_string(&s).unwrap().contains(&tok), "le jeton en clair n'est jamais stocké");
    }
    #[test]
    fn expiry_idle_and_absolute() {
        let mut s = Sessions::default();
        let (_, t1) = s.create("a", 0);
        assert!(s.validate(&t1, IDLE_SECONDS + 1).is_none(), "inactivité");
        let (_, t2) = s.create("b", 0);
        for day in 1..=89 { assert!(s.validate(&t2, day * 86_400).is_some()); }
        assert!(s.validate(&t2, ABSOLUTE_SECONDS + 1).is_none(), "durée maximale absolue");
    }
    #[test]
    fn revocation() {
        let mut s = Sessions::default();
        let (a, ta) = s.create("a", 0);
        let (b, tb) = s.create("b", 0);
        let (_c, tc) = s.create("c", 0);
        assert!(s.revoke(&a) && !s.revoke(&a));
        assert!(s.validate(&ta, 1).is_none());
        assert_eq!(s.revoke_all_except(Some(&b)), 1);
        assert!(s.validate(&tb, 2).is_some() && s.validate(&tc, 2).is_none());
        assert_eq!(s.list().len(), 1);
    }
    #[test]
    fn device_label_is_sanitised() {
        let mut s = Sessions::default();
        s.create(&format!("x\n\u{0}{}", "é".repeat(200)), 0);
        assert!(s.list()[0].device.chars().count() <= 80 && !s.list()[0].device.contains('\n'));
    }
}
