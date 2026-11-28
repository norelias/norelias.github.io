//! Numéros de téléphone : normalisation E.164 par la bibliothèque `phonenumber` (métadonnées libphonenumber).

use phonenumber::{country, Mode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Phone { pub e164: String, pub country_code: u16, pub national: String }

#[derive(Debug, PartialEq, Eq)]
pub enum PhoneError { Empty, Unparseable, Invalid }

/// `default_region` : code ISO 3166-1 alpha-2 utilisé si le numéro n'a pas d'indicatif (+…).
pub fn parse(input: &str, default_region: Option<&str>) -> Result<Phone, PhoneError> {
    let s = input.trim();
    if s.is_empty() { return Err(PhoneError::Empty); }
    let region = default_region.and_then(|r| r.to_uppercase().parse::<country::Id>().ok());
    let n = phonenumber::parse(region, s).map_err(|_| PhoneError::Unparseable)?;
    if !phonenumber::is_valid(&n) { return Err(PhoneError::Invalid); }
    Ok(Phone { e164: n.format().mode(Mode::E164).to_string(), country_code: n.code().value(), national: n.format().mode(Mode::National).to_string() })
}

/// Vérifie la forme E.164 stricte : « + » puis 8 à 15 chiffres, sans 0 initial.
pub fn is_e164(s: &str) -> bool {
    let Some(d) = s.strip_prefix('+') else { return false };
    (8..=15).contains(&d.len()) && d.chars().all(|c| c.is_ascii_digit()) && !d.starts_with('0')
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normalises_to_e164() {
        assert_eq!(parse("(514) 555-0199", Some("CA")).unwrap().e164, "+15145550199");
        assert_eq!(parse("514 398 7000", Some("CA")).unwrap().e164, "+15143987000");
        assert_eq!(parse("+33 1 42 68 53 00", None).unwrap().e164, "+33142685300");
        assert_eq!(parse("01 42 68 53 00", Some("FR")).unwrap().e164, "+33142685300");
        assert_eq!(parse("+44 20 7946 0958", None).unwrap().country_code, 44);
    }
    #[test]
    fn rejects_invalid() {
        assert_eq!(parse("", None), Err(PhoneError::Empty));
        assert_eq!(parse("abc", Some("CA")), Err(PhoneError::Unparseable));
        assert_eq!(parse("123", Some("CA")), Err(PhoneError::Invalid));
        assert!(parse("514 398 7000", None).is_err(), "sans indicatif ni pays : ambigu");
        assert!(parse("+1 000 000 0000", None).is_err());
    }
    #[test]
    fn e164_shape() {
        assert!(is_e164("+15143987000") && is_e164("+33142685300"));
        for bad in ["15143987000", "+0123456789", "+1234567", "+1234567890123456", "+1 514", "+12a45678"] { assert!(!is_e164(bad), "{bad}"); }
    }
}
