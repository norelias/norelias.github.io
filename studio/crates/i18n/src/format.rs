//! Formats de nombres, de monnaies et de dates selon la langue.

use crate::Locale;

const NNBSP: char = '\u{202F}'; // espace fine insécable (milliers en français)
const NBSP: char = '\u{00A0}';

struct Sep { dec: char, group: Option<char> }

fn sep(loc: &Locale) -> Sep {
    match loc.lang.as_str() {
        "fr" => Sep { dec: ',', group: Some(NNBSP) },
        "de" | "es" | "it" | "pt" | "nl" => {
            // l'espagnol (es-MX) et l'anglais utilisent le point décimal ; es-ES/de/it/pt la virgule
            if loc.lang == "es" && matches!(loc.region.as_deref(), Some("MX") | Some("US") | Some("PR")) { Sep { dec: '.', group: Some(',') } }
            else if loc.lang == "de" && loc.region.as_deref() == Some("CH") { Sep { dec: '.', group: Some('’') } }
            else { Sep { dec: ',', group: Some('.') } }
        }
        _ => Sep { dec: '.', group: Some(',') },
    }
}

pub fn format_number(v: f64, decimals: usize, loc: &Locale) -> String {
    let s = sep(loc);
    let neg = v < 0.0 && format!("{:.*}", decimals, v.abs()).chars().any(|c| c.is_ascii_digit() && c != '0');
    let txt = format!("{:.*}", decimals, v.abs());
    let (int, frac) = txt.split_once('.').map_or((txt.as_str(), ""), |(a, b)| (a, b));
    let mut grouped = String::new();
    for (i, ch) in int.chars().enumerate() {
        if i > 0 && (int.len() - i) % 3 == 0 { if let Some(g) = s.group { grouped.push(g); } }
        grouped.push(ch);
    }
    let mut out = String::new();
    if neg { out.push('−'); }
    out.push_str(&grouped);
    if decimals > 0 { out.push(s.dec); out.push_str(frac); }
    out
}

fn symbol(code: &str) -> &str {
    match code { "CAD" | "USD" | "AUD" | "NZD" | "MXN" => "$", "EUR" => "€", "GBP" => "£", "CHF" => "CHF", "JPY" => "¥", "BRL" => "R$", _ => code }
}
fn minor_digits(code: &str) -> usize { if matches!(code, "JPY" | "KRW") { 0 } else { 2 } }

/// Montant en unités mineures (cents) → texte. Français : « 1 234,56 $ » ; anglais : « $1,234.56 ».
pub fn format_money(minor: i64, currency: &str, loc: &Locale) -> String {
    let d = minor_digits(currency);
    let value = minor as f64 / 10f64.powi(d as i32);
    let n = format_number(value, d, loc);
    let sym = symbol(currency);
    match loc.lang.as_str() {
        "fr" | "de" | "es" | "it" | "pt" if !(loc.lang == "es" && matches!(loc.region.as_deref(), Some("MX"))) =>
            format!("{n}{NBSP}{sym}"),
        _ => if let Some(rest) = n.strip_prefix('−') { format!("−{sym}{rest}") } else { format!("{sym}{n}") },
    }
}

const MONTHS_FR: [&str; 12] = ["janvier", "février", "mars", "avril", "mai", "juin", "juillet", "août", "septembre", "octobre", "novembre", "décembre"];
const MONTHS_EN: [&str; 12] = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];

pub fn format_date_long(y: i32, m: u32, d: u32, loc: &Locale) -> Option<String> {
    if !(1..=12).contains(&m) || !(1..=days_in_month(y, m)).contains(&d) { return None; }
    Some(match loc.lang.as_str() {
        "fr" => format!("{}{} {} {y}", d, if d == 1 { "er" } else { "" }, MONTHS_FR[m as usize - 1]),
        _ => format!("{} {d}, {y}", MONTHS_EN[m as usize - 1]),
    })
}
/// Date numérique : fr-CA/ISO « 2026-11-28 », fr « 28/11/2026 », en-US « 11/28/2026 », en-GB « 28/11/2026 ».
pub fn format_date_numeric(y: i32, m: u32, d: u32, loc: &Locale) -> Option<String> {
    if !(1..=12).contains(&m) || !(1..=days_in_month(y, m)).contains(&d) { return None; }
    Some(match (loc.lang.as_str(), loc.region.as_deref()) {
        ("fr", Some("CA")) | ("sv", _) => format!("{y:04}-{m:02}-{d:02}"),
        ("en", Some("US")) => format!("{m:02}/{d:02}/{y:04}"),
        ("en", Some("CA")) => format!("{y:04}-{m:02}-{d:02}"),
        ("de", _) => format!("{d:02}.{m:02}.{y:04}"),
        _ => format!("{d:02}/{m:02}/{y:04}"),
    })
}
pub fn days_in_month(y: i32, m: u32) -> u32 {
    match m { 1 | 3 | 5 | 7 | 8 | 10 | 12 => 31, 4 | 6 | 9 | 11 => 30, 2 => if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 { 29 } else { 28 }, _ => 0 }
}
/// Année civile (UTC) d'un horodatage Unix — utile pour le filigrane « © {année} ».
pub fn year_from_unix(secs: u64) -> i32 {
    let z = (secs / 86400) as i64 + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (yoe + era * 400 + if m <= 2 { 1 } else { 0 }) as i32
}

/// Fuseaux horaires IANA : validation de forme + liste de fuseaux courants. Une base complète
/// (chrono-tz) est prévue ; ce contrôle n'est PAS exhaustif.
pub const COMMON_TIMEZONES: [&str; 24] = [
    "America/Toronto", "America/Montreal", "America/Vancouver", "America/Edmonton", "America/Winnipeg", "America/Halifax",
    "America/St_Johns", "America/New_York", "America/Chicago", "America/Denver", "America/Los_Angeles", "America/Mexico_City",
    "America/Sao_Paulo", "Europe/London", "Europe/Paris", "Europe/Brussels", "Europe/Berlin", "Europe/Madrid", "Europe/Rome",
    "Europe/Lisbon", "Europe/Zurich", "Asia/Tokyo", "Australia/Sydney", "UTC",
];
pub fn timezone_known(id: &str) -> bool { COMMON_TIMEZONES.contains(&id) }
pub fn timezone_well_formed(id: &str) -> bool {
    id == "UTC" || { let p: Vec<&str> = id.split('/').collect(); (2..=3).contains(&p.len()) && p.iter().all(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '+')) }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn l(t: &str) -> Locale { Locale::parse(t).unwrap() }
    #[test]
    fn numbers_by_locale() {
        assert_eq!(format_number(1234567.891, 2, &l("en-US")), "1,234,567.89");
        assert_eq!(format_number(1234567.891, 2, &l("fr-CA")), "1\u{202F}234\u{202F}567,89");
        assert_eq!(format_number(1234.5, 1, &l("de-DE")), "1.234,5");
        assert_eq!(format_number(-0.001, 2, &l("en")), "0.00", "pas de « −0,00 »");
        assert_eq!(format_number(12.0, 0, &l("fr")), "12");
    }
    #[test]
    fn money_by_locale() {
        assert_eq!(format_money(123456, "CAD", &l("fr-CA")), "1\u{202F}234,56\u{A0}$");
        assert_eq!(format_money(123456, "USD", &l("en-US")), "$1,234.56");
        assert_eq!(format_money(-5000, "EUR", &l("en")), "−€50.00");
        assert_eq!(format_money(1500, "JPY", &l("en")), "¥1,500");
        assert_eq!(format_money(99, "EUR", &l("fr-FR")), "0,99\u{A0}€");
    }
    #[test]
    fn dates() {
        assert_eq!(format_date_long(2026, 11, 28, &l("fr")).unwrap(), "28 novembre 2026");
        assert_eq!(format_date_long(2026, 2, 1, &l("fr")).unwrap(), "1er février 2026");
        assert_eq!(format_date_long(2026, 11, 28, &l("en")).unwrap(), "November 28, 2026");
        assert_eq!(format_date_numeric(2026, 11, 28, &l("en-US")).unwrap(), "11/28/2026");
        assert_eq!(format_date_numeric(2026, 11, 28, &l("fr-CA")).unwrap(), "2026-11-28");
        assert!(format_date_long(2026, 2, 29, &l("fr")).is_none());
        assert!(format_date_long(2024, 2, 29, &l("fr")).is_some());
    }
    #[test]
    fn year_from_timestamp() {
        assert_eq!(year_from_unix(0), 1970);
        assert_eq!(year_from_unix(1_767_225_600), 2026); // 2026-01-01T00:00:00Z
        assert_eq!(year_from_unix(1_767_225_599), 2025);
        assert_eq!(year_from_unix(951_782_400), 2000); // 2000-02-29
    }
    #[test]
    fn timezones() {
        assert!(timezone_known("America/Toronto") && !timezone_known("Mars/Olympus"));
        assert!(timezone_well_formed("America/Argentina/Buenos_Aires") && !timezone_well_formed("../etc/passwd") && !timezone_well_formed("Toronto"));
    }
}
