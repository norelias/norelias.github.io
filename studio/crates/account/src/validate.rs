//! Validations de champs. Volontairement permissives sur la forme (on ne tente pas de refuser des adresses réelles)
//! et strictes sur la sécurité (pas de caractères de contrôle, longueurs bornées).

fn no_control(s: &str) -> bool { !s.chars().any(|c| c.is_control()) }

/// Courriel : une seule arobase, partie locale ≤ 64, domaine à étiquettes valides contenant un point. Unicode accepté.
pub fn email(s: &str) -> bool {
    let s = s.trim();
    if s.len() > 254 || !no_control(s) || s.chars().any(|c| c.is_whitespace()) { return false; }
    let Some((local, domain)) = s.split_once('@') else { return false };
    if local.is_empty() || local.len() > 64 || domain.contains('@') || local.starts_with('.') || local.ends_with('.') || local.contains("..") { return false; }
    if local.chars().any(|c| "()<>[]\\,;:\"".contains(c)) { return false; }
    let labels: Vec<&str> = domain.split('.').collect();
    labels.len() >= 2 && labels.iter().all(|l| !l.is_empty() && l.len() <= 63 && !l.starts_with('-') && !l.ends_with('-') && l.chars().all(|c| c.is_alphanumeric() || c == '-'))
        && labels.last().is_some_and(|t| t.chars().count() >= 2)
}

/// Adresse Web http(s) uniquement ; sans espaces ni caractères de contrôle ; un hôte avec point.
pub fn web_url(s: &str) -> bool {
    let s = s.trim();
    if s.len() > 2048 || !no_control(s) || s.contains(char::is_whitespace) { return false; }
    let Some(rest) = s.strip_prefix("https://").or_else(|| s.strip_prefix("http://")) else { return false };
    let host = rest.split(['/', '?', '#']).next().unwrap_or("");
    let host = host.rsplit('@').next().unwrap_or("").split(':').next().unwrap_or("");
    host.contains('.') && !host.starts_with('.') && !host.ends_with('.') && !host.contains("..")
}

/// Texte libre borné, sans caractères de contrôle (nom, ville…).
pub fn text(s: &str, max_chars: usize) -> bool { s.chars().count() <= max_chars && no_control(s) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn emails() {
        for ok in ["a@b.co", "jean.tremblay+photo@exemple.ca", "élodie@société.fr", "x@sub.domain.org"] { assert!(email(ok), "{ok}"); }
        for bad in ["", "a", "a@b", "@b.co", "a@@b.co", "a b@c.co", "a@b..co", ".a@b.co", "a.@b.co", "a@-b.co", "a@b.c", "a(b)@c.co", &format!("{}@x.co", "a".repeat(65))] { assert!(!email(bad), "{bad:?}"); }
    }
    #[test]
    fn surrounding_whitespace_is_trimmed() { assert!(email(" a@b.co\n")); assert!(!email("a@b.co\n.x")); }
    #[test]
    fn urls() {
        for ok in ["https://norelias.ca", "http://exemple.com/a?b=1#c", "https://www.exemple.co.uk:8080/x"] { assert!(web_url(ok), "{ok}"); }
        for bad in ["norelias.ca", "javascript:alert(1)", "ftp://x.com", "https://", "https://localhost", "https://a b.com", "https://.com", "data:text/html,x"] { assert!(!web_url(bad), "{bad}"); }
    }
    #[test]
    fn free_text() { assert!(text("Café ☕", 10) && !text("a\u{0}b", 10) && !text("abcdef", 3)); }
}
