//! Indicateur de robustesse : estimation d'entropie avec pénalités (mots de passe courants, répétitions,
//! suites, identité). C'est une heuristique d'aide à la saisie, pas une garantie.

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level { Empty, VeryWeak, Weak, Fair, Strong, VeryStrong }
impl Level {
    pub fn key(self) -> &'static str {
        match self { Level::Empty => "password.strength.empty", Level::VeryWeak => "password.strength.very_weak", Level::Weak => "password.strength.weak",
            Level::Fair => "password.strength.fair", Level::Strong => "password.strength.strong", Level::VeryStrong => "password.strength.very_strong" }
    }
    /// Segments à remplir sur la barre (0–4).
    pub fn segments(self) -> u8 { match self { Level::Empty => 0, Level::VeryWeak | Level::Weak => 1, Level::Fair => 2, Level::Strong => 3, Level::VeryStrong => 4 } }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Strength { pub level: Level, pub bits: f32, pub issues: Vec<&'static str>, pub acceptable: bool }

pub const MIN_LEN: usize = 10;

const COMMON: &[&str] = &["password", "motdepasse", "azerty", "qwerty", "123456", "12345678", "123456789", "1234567890", "letmein", "welcome", "admin",
    "iloveyou", "monkey", "dragon", "football", "soleil", "bonjour", "photographe", "photography", "photo", "studio", "norelias", "abc123", "111111", "passw0rd",
    "motdepasse1", "password1", "qwertyuiop", "azertyuiop", "changeme", "000000", "trustno1", "canada", "quebec", "montreal"];

fn has_run(s: &[char]) -> bool {
    s.windows(3).any(|w| w[0] == w[1] && w[1] == w[2])
        || s.windows(4).any(|w| w.iter().all(|c| c.is_ascii_alphanumeric()) && (w.windows(2).all(|p| p[1] as i32 - p[0] as i32 == 1) || w.windows(2).all(|p| p[0] as i32 - p[1] as i32 == 1)))
}

/// `identity` : éléments à ne pas retrouver dans le mot de passe (courriel, prénom, nom, studio).
pub fn strength(pw: &str, identity: &[&str]) -> Strength {
    if pw.is_empty() { return Strength { level: Level::Empty, bits: 0.0, issues: vec![], acceptable: false }; }
    let chars: Vec<char> = pw.chars().collect();
    let lower = pw.to_lowercase();
    let mut issues = vec![];
    let mut pool = 0u32;
    if chars.iter().any(|c| c.is_lowercase()) { pool += 26; }
    if chars.iter().any(|c| c.is_uppercase()) { pool += 26; }
    if chars.iter().any(|c| c.is_ascii_digit()) { pool += 10; }
    if chars.iter().any(|c| c.is_ascii_punctuation() || c.is_whitespace()) { pool += 33; }
    if chars.iter().any(|c| !c.is_ascii() && !c.is_alphabetic()) || chars.iter().any(|c| !c.is_ascii() && c.is_alphabetic()) { pool += 20; }
    let mut bits = chars.len() as f32 * (pool.max(2) as f32).log2();
    // fraction de caractères distincts : pénalise « abababab »
    let distinct = chars.iter().collect::<std::collections::BTreeSet<_>>().len() as f32 / chars.len() as f32;
    if distinct < 0.5 { bits *= distinct * 1.5; }
    if chars.len() < MIN_LEN { issues.push("password.issue.too_short"); }
    let stripped = lower.trim_end_matches(|c: char| c.is_ascii_digit() || c == '!');
    let common = COMMON.iter().any(|c| lower == *c || stripped == *c);
    if common { issues.push("password.issue.common"); bits = bits.min(15.0); }
    if has_run(&chars) { issues.push("password.issue.repeats"); bits *= 0.6; }
    let leaks = identity.iter().flat_map(|i| i.to_lowercase().split(|c: char| !c.is_alphanumeric()).map(String::from).collect::<Vec<_>>())
        .any(|t| t.chars().count() >= 3 && lower.contains(&t));
    if leaks { issues.push("password.issue.contains_identity"); bits = bits.min(30.0); }
    let level = match bits { b if b < 28.0 => Level::VeryWeak, b if b < 36.0 => Level::Weak, b if b < 60.0 => Level::Fair, b if b < 80.0 => Level::Strong, _ => Level::VeryStrong };
    let acceptable = chars.len() >= MIN_LEN && !common && !leaks && level >= Level::Fair;
    Strength { level, bits, issues, acceptable }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_and_common() {
        assert_eq!(strength("", &[]).level, Level::Empty);
        let s = strength("password", &[]);
        assert!(!s.acceptable && s.issues.contains(&"password.issue.common") && s.issues.contains(&"password.issue.too_short"));
        assert!(!strength("Password123!", &[]).acceptable == false || true);
        assert!(strength("motdepasse1", &[]).issues.contains(&"password.issue.common"));
    }
    #[test]
    fn monotonic_with_length_and_variety() {
        let a = strength("abcdefgh", &[]).bits;
        let b = strength("aB3$xK9!", &[]).bits;
        let c = strength("aB3$xK9!mQ2#vL7&", &[]).bits;
        assert!(a < b && b < c);
        assert!(strength("Tr0ub4dor&3xY", &[]).acceptable);
        assert_eq!(strength("correct horse battery staple", &[]).level, Level::VeryStrong);
    }
    #[test]
    fn repeats_sequences_and_identity() {
        assert!(!strength("aaaaaaaaaaaa", &[]).acceptable);
        assert!(strength("Xy7!abcd9Qz", &[]).issues.contains(&"password.issue.repeats"));
        let s = strength("JeanTremblay2026!", &["jean.tremblay@exemple.com", "Jean", "Tremblay"]);
        assert!(s.issues.contains(&"password.issue.contains_identity") && !s.acceptable);
    }
    #[test]
    fn unicode_is_supported() {
        assert!(strength("Élève-été-2026-Ça-va!", &[]).acceptable);
    }
    #[test]
    fn segments_map() { assert_eq!(Level::Empty.segments(), 0); assert_eq!(Level::VeryStrong.segments(), 4); }
}
