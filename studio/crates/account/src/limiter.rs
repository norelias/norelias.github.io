//! Limitation des tentatives de connexion : délai exponentiel puis verrouillage temporaire.
//! L'horloge est injectée (`now` en secondes) : testable sans attente.

use serde::{Deserialize, Serialize};

pub const FREE_ATTEMPTS: u32 = 3;
pub const LOCK_AFTER: u32 = 6;
pub const LOCK_SECONDS: u64 = 15 * 60;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RateLimiter { failures: u32, blocked_until: u64 }

impl RateLimiter {
    /// `Some(secondes_restantes)` si une tentative est interdite maintenant.
    pub fn blocked_for(&self, now: u64) -> Option<u64> { (self.blocked_until > now).then(|| self.blocked_until - now) }
    pub fn record_failure(&mut self, now: u64) {
        self.failures += 1;
        let wait = if self.failures >= LOCK_AFTER { LOCK_SECONDS }
                   else if self.failures > FREE_ATTEMPTS { 1u64 << (self.failures - FREE_ATTEMPTS) } else { 0 };
        self.blocked_until = now + wait;
    }
    pub fn record_success(&mut self) { *self = Self::default(); }
    pub fn failures(&self) -> u32 { self.failures }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn backoff_then_lock_then_reset() {
        let mut l = RateLimiter::default();
        for _ in 0..FREE_ATTEMPTS { l.record_failure(100); assert_eq!(l.blocked_for(100), None); }
        l.record_failure(100); assert_eq!(l.blocked_for(100), Some(2));
        l.record_failure(100); assert_eq!(l.blocked_for(100), Some(4));
        assert_eq!(l.blocked_for(104), None, "le délai expire");
        l.record_failure(200); assert_eq!(l.blocked_for(200), Some(LOCK_SECONDS), "verrouillage temporaire");
        assert_eq!(l.blocked_for(200 + LOCK_SECONDS), None);
        l.record_success();
        assert_eq!((l.failures(), l.blocked_for(0)), (0, None));
    }
}
