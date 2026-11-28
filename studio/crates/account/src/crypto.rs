//! Primitives : aléa système, SHA-256, Argon2id, chiffrement authentifié, comparaison à temps constant.

use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::{aead::{Aead, KeyInit}, ChaCha20Poly1305};
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct KdfCost { pub m_kib: u32, pub t: u32, pub p: u32 }
impl KdfCost {
    /// Minimum recommandé par l'OWASP pour Argon2id (19 Mio, 2 itérations, 1 voie).
    pub const PRODUCTION: KdfCost = KdfCost { m_kib: 19_456, t: 2, p: 1 };
    /// Coût réduit pour les tests uniquement.
    pub const TESTING: KdfCost = KdfCost { m_kib: 64, t: 1, p: 1 };
}

pub fn random_bytes<const N: usize>() -> [u8; N] {
    let mut b = [0u8; N];
    getrandom::fill(&mut b).expect("aléa du système indisponible");
    b
}
pub fn hex(b: &[u8]) -> String { b.iter().map(|x| format!("{x:02x}")).collect() }
pub fn unhex(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 { return None; }
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(s.get(i..i + 2)?, 16).ok()).collect()
}
pub fn sha256_hex(data: &[u8]) -> String { hex(&Sha256::digest(data)) }
pub fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

fn kdf(password: &str, salt: &[u8], cost: KdfCost, out: &mut [u8]) -> Result<(), String> {
    let params = Params::new(cost.m_kib, cost.t, cost.p, Some(out.len())).map_err(|e| e.to_string())?;
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params).hash_password_into(password.as_bytes(), salt, out).map_err(|e| e.to_string())
}

/// Hachage de mot de passe : `argon2id:m,t,p:sel:hash` (le mot de passe n'est jamais conservé).
pub fn hash_password(password: &str, cost: KdfCost) -> String {
    let salt = random_bytes::<16>();
    let mut out = [0u8; 32];
    kdf(password, &salt, cost, &mut out).expect("paramètres Argon2 valides");
    format!("argon2id:{},{},{}:{}:{}", cost.m_kib, cost.t, cost.p, hex(&salt), hex(&out))
}
pub fn verify_password(password: &str, stored: &str) -> bool {
    let parts: Vec<&str> = stored.split(':').collect();
    if parts.len() != 4 || parts[0] != "argon2id" { return false; }
    let c: Vec<u32> = parts[1].split(',').filter_map(|x| x.parse().ok()).collect();
    let (Some(salt), Some(want), true) = (unhex(parts[2]), unhex(parts[3]), c.len() == 3) else { return false };
    let mut out = vec![0u8; want.len()];
    kdf(password, &salt, KdfCost { m_kib: c[0], t: c[1], p: c[2] }, &mut out).is_ok() && ct_eq(&out, &want)
}

/// Secret chiffré par une clé dérivée du mot de passe (ChaCha20-Poly1305). Sans le mot de passe, illisible.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Sealed { pub salt: String, pub nonce: String, pub data: String, pub cost: KdfCost }

pub fn seal(secret: &[u8], password: &str, cost: KdfCost) -> Sealed {
    let (salt, nonce) = (random_bytes::<16>(), random_bytes::<12>());
    let mut key = [0u8; 32];
    kdf(password, &salt, cost, &mut key).expect("kdf");
    let c = ChaCha20Poly1305::new(&key.into());
    let data = c.encrypt(&nonce.into(), secret).expect("chiffrement");
    Sealed { salt: hex(&salt), nonce: hex(&nonce), data: hex(&data), cost }
}
pub fn unseal(s: &Sealed, password: &str) -> Option<Vec<u8>> {
    let (salt, nonce, data) = (unhex(&s.salt)?, unhex(&s.nonce)?, unhex(&s.data)?);
    let nonce: [u8; 12] = nonce.try_into().ok()?;
    let mut key = [0u8; 32];
    kdf(password, &salt, s.cost, &mut key).ok()?;
    ChaCha20Poly1305::new(&key.into()).decrypt(&nonce.into(), data.as_ref()).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn password_hash_roundtrip_and_no_plaintext() {
        let h = hash_password("S3cret ünïcode ✓", KdfCost::TESTING);
        assert!(verify_password("S3cret ünïcode ✓", &h));
        assert!(!verify_password("s3cret ünïcode ✓", &h));
        assert!(!h.contains("S3cret"));
        assert_ne!(h, hash_password("S3cret ünïcode ✓", KdfCost::TESTING), "sel aléatoire");
    }
    #[test]
    fn malformed_hashes_are_rejected() {
        for bad in ["", "x", "argon2id:1,1,1:zz:zz", "md5:1,1,1:00:00", "argon2id:a,b,c:00:00"] { assert!(!verify_password("x", bad), "{bad}"); }
    }
    #[test]
    fn sealed_secret_needs_the_password_and_detects_tampering() {
        let s = seal(b"JBSWY3DPEHPK3PXP", "pw", KdfCost::TESTING);
        assert_eq!(unseal(&s, "pw").unwrap(), b"JBSWY3DPEHPK3PXP");
        assert!(unseal(&s, "autre").is_none());
        let mut t = s.clone();
        t.data = format!("00{}", &t.data[2..]);
        assert!(unseal(&t, "pw").is_none());
        assert!(!s.data.contains("4a4253"), "le secret n'apparaît pas en clair");
    }
    #[test]
    fn constant_time_eq_and_hex() {
        assert!(ct_eq(b"abc", b"abc") && !ct_eq(b"abc", b"abd") && !ct_eq(b"abc", b"ab"));
        assert_eq!(unhex(&hex(&[0, 255, 16])).unwrap(), vec![0, 255, 16]);
        assert!(unhex("abc").is_none() && unhex("zz").is_none());
    }
    #[test]
    fn random_values_differ() { assert_ne!(random_bytes::<16>(), random_bytes::<16>()); }
}
