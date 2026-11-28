//! Mot de passe à usage unique basé sur le temps (RFC 6238, HMAC-SHA1, 6 chiffres, 30 s).

use hmac::{Hmac, KeyInit, Mac};
use sha1::Sha1;

const B32: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

pub fn base32_encode(data: &[u8]) -> String {
    let (mut out, mut buf, mut bits) = (String::new(), 0u32, 0);
    for &b in data {
        buf = (buf << 8) | b as u32;
        bits += 8;
        while bits >= 5 { bits -= 5; out.push(B32[((buf >> bits) & 31) as usize] as char); }
    }
    if bits > 0 { out.push(B32[((buf << (5 - bits)) & 31) as usize] as char); }
    out
}
pub fn base32_decode(s: &str) -> Option<Vec<u8>> {
    let (mut out, mut buf, mut bits) = (vec![], 0u32, 0);
    for c in s.trim_end_matches('=').chars() {
        let v = B32.iter().position(|&x| x as char == c.to_ascii_uppercase())? as u32;
        buf = (buf << 5) | v;
        bits += 5;
        if bits >= 8 { bits -= 8; out.push(((buf >> bits) & 0xFF) as u8); }
    }
    Some(out)
}

pub fn generate_secret() -> Vec<u8> { crate::crypto::random_bytes::<20>().to_vec() }

/// Code à `digits` chiffres pour le pas de temps `step`.
pub fn code_for_step(secret: &[u8], step: u64, digits: u32) -> String {
    let mut mac = <Hmac<Sha1> as KeyInit>::new_from_slice(secret).expect("clé HMAC");
    mac.update(&step.to_be_bytes());
    let h = mac.finalize().into_bytes();
    let off = (h[19] & 0x0f) as usize;
    let bin = u32::from_be_bytes([h[off] & 0x7f, h[off + 1], h[off + 2], h[off + 3]]);
    format!("{:0width$}", bin % 10u32.pow(digits), width = digits as usize)
}

pub fn step_at(unix: u64) -> u64 { unix / 30 }

/// Vérifie un code avec une tolérance d'un pas de part et d'autre. Retourne le pas validé ;
/// `last_used` interdit de rejouer un code déjà accepté.
pub fn verify(secret: &[u8], code: &str, unix: u64, last_used: Option<u64>) -> Option<u64> {
    let code = code.trim();
    if code.len() != 6 || !code.chars().all(|c| c.is_ascii_digit()) { return None; }
    let now = step_at(unix);
    for step in [now.wrapping_sub(1), now, now + 1] {
        if last_used.is_some_and(|l| step <= l) { continue; }
        if crate::crypto::ct_eq(code_for_step(secret, step, 6).as_bytes(), code.as_bytes()) { return Some(step); }
    }
    None
}

/// URI `otpauth://` pour l'enregistrement dans une application d'authentification.
pub fn otpauth_uri(secret: &[u8], account: &str, issuer: &str) -> String {
    let enc = |s: &str| s.bytes().map(|b| if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) { (b as char).to_string() } else { format!("%{b:02X}") }).collect::<String>();
    format!("otpauth://totp/{}:{}?secret={}&issuer={}&algorithm=SHA1&digits=6&period=30", enc(issuer), enc(account), base32_encode(secret), enc(issuer))
}

#[cfg(test)]
mod tests {
    use super::*;
    const RFC_SECRET: &[u8] = b"12345678901234567890";
    #[test]
    fn rfc6238_test_vectors() {
        // RFC 6238, annexe B (SHA-1, 8 chiffres)
        for (t, want) in [(59u64, "94287082"), (1111111109, "07081804"), (1111111111, "14050471"), (1234567890, "89005924"), (2000000000, "69279037"), (20000000000, "65353130")] {
            assert_eq!(code_for_step(RFC_SECRET, step_at(t), 8), want, "t={t}");
        }
        assert_eq!(code_for_step(RFC_SECRET, step_at(59), 6), "287082");
    }
    #[test]
    fn base32_roundtrip_and_known_value() {
        assert_eq!(base32_encode(b"foobar"), "MZXW6YTBOI");
        assert_eq!(base32_decode("MZXW6YTBOI").unwrap(), b"foobar");
        assert_eq!(base32_decode("mzxw6ytboi===").unwrap(), b"foobar");
        assert!(base32_decode("1!").is_none());
        let s = generate_secret();
        assert_eq!(base32_decode(&base32_encode(&s)).unwrap(), s);
    }
    #[test]
    fn verification_window_and_replay() {
        let now = 1_000_000u64;
        let c = code_for_step(RFC_SECRET, step_at(now), 6);
        let step = verify(RFC_SECRET, &c, now, None).unwrap();
        assert!(verify(RFC_SECRET, &c, now, Some(step)).is_none(), "un code déjà utilisé est refusé");
        assert!(verify(RFC_SECRET, &c, now + 30, None).is_some(), "tolérance d'un pas");
        assert!(verify(RFC_SECRET, &c, now + 120, None).is_none(), "expiré");
        assert!(verify(RFC_SECRET, "12345", now, None).is_none() && verify(RFC_SECRET, "abcdef", now, None).is_none());
    }
    #[test]
    fn uri_is_escaped() {
        let u = otpauth_uri(b"12345678901234567890", "a b@x.com", "Norélias Studio");
        assert!(u.starts_with("otpauth://totp/Nor%C3%A9lias%20Studio:a%20b%40x.com?secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ"));
    }
}
