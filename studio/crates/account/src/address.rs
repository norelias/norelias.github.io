//! Adresses internationales : formats par pays, validation du code postal et de la région, mise en forme
//! postale, autocomplétion via un fournisseur **soumis au consentement**, saisie manuelle et mode hors connexion.
//!
//! Couverture : formats détaillés pour CA, US, FR, GB, DE, ES, IT, PT, BE, CH, NL, AU, MX, BR, JP.
//! Tout autre pays utilise un format générique **sans validation de code postal** (jamais de refus à tort).

use crate::FieldError;
use norelias_onboarding::privacy::{ConsentLedger, Purpose};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Address {
    /// Code ISO 3166-1 alpha-2.
    pub country: String,
    pub line1: String,
    pub line2: String,
    pub city: String,
    pub region: String,
    pub postal_code: String,
}
impl Address {
    pub fn is_empty(&self) -> bool { [&self.line1, &self.line2, &self.city, &self.region, &self.postal_code].iter().all(|s| s.trim().is_empty()) }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegionKind { Province, State, Region, County, Prefecture, None }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout { CityRegionPostal, PostalCity, CityThenPostal, Japan }

#[derive(Clone, Debug)]
pub struct Format {
    pub country: &'static str,
    pub region_kind: RegionKind,
    /// Codes de région valides (vide = texte libre).
    pub regions: &'static [&'static str],
    pub zip_label: bool,
    pub postal_patterns: &'static [&'static str],
    pub layout: Layout,
}

const CA: &[&str] = &["AB", "BC", "MB", "NB", "NL", "NS", "NT", "NU", "ON", "PE", "QC", "SK", "YT"];
const US: &[&str] = &["AL", "AK", "AZ", "AR", "CA", "CO", "CT", "DE", "DC", "FL", "GA", "HI", "ID", "IL", "IN", "IA", "KS", "KY", "LA", "ME", "MD", "MA", "MI", "MN", "MS", "MO",
    "MT", "NE", "NV", "NH", "NJ", "NM", "NY", "NC", "ND", "OH", "OK", "OR", "PA", "RI", "SC", "SD", "TN", "TX", "UT", "VT", "VA", "WA", "WV", "WI", "WY", "PR", "GU", "VI", "AS", "MP"];
const AU: &[&str] = &["ACT", "NSW", "NT", "QLD", "SA", "TAS", "VIC", "WA"];

pub fn format_for(country: &str) -> Format {
    use Layout::*; use RegionKind as R;
    let f = |c, region_kind, regions, zip_label, postal_patterns, layout| Format { country: c, region_kind, regions, zip_label, postal_patterns, layout };
    match country.to_uppercase().as_str() {
        "CA" => f("CA", R::Province, CA, false, &["A9A9A9"], CityRegionPostal),
        "US" => f("US", R::State, US, true, &["99999", "99999-9999"], CityRegionPostal),
        "FR" => f("FR", R::None, &[], false, &["99999"], PostalCity),
        "DE" => f("DE", R::None, &[], false, &["99999"], PostalCity),
        "ES" => f("ES", R::Region, &[], false, &["99999"], PostalCity),
        "IT" => f("IT", R::Region, &[], false, &["99999"], PostalCity),
        "PT" => f("PT", R::None, &[], false, &["9999-999"], PostalCity),
        "BE" => f("BE", R::None, &[], false, &["9999"], PostalCity),
        "CH" => f("CH", R::None, &[], false, &["9999"], PostalCity),
        "NL" => f("NL", R::None, &[], false, &["9999AA"], PostalCity),
        "GB" => f("GB", R::County, &[], false, &["A99AA", "A999AA", "AA99AA", "AA999AA", "A9A9AA", "AA9A9AA"], CityThenPostal),
        "AU" => f("AU", R::State, AU, false, &["9999"], CityRegionPostal),
        "MX" => f("MX", R::State, &[], false, &["99999"], PostalCity),
        "BR" => f("BR", R::State, &[], false, &["99999-999"], PostalCity),
        "JP" => f("JP", R::Prefecture, &[], false, &["999-9999"], Japan),
        _ => f("", R::Region, &[], false, &[], CityRegionPostal),
    }
}

impl Format {
    pub fn region_label_key(&self) -> &'static str {
        match self.region_kind { RegionKind::Province => "field.region.province", RegionKind::State => "field.region.state", RegionKind::County => "field.region.county",
            RegionKind::Prefecture => "field.region.prefecture", _ => "field.region.region" }
    }
    pub fn postal_label_key(&self) -> &'static str { if self.zip_label { "field.zip_code" } else { "field.postal_code" } }
    pub fn shows_region(&self) -> bool { self.region_kind != RegionKind::None }
}

/// Motif : `A` = lettre, `9` = chiffre, autre caractère = littéral. Espaces et casse ignorés.
fn matches_pattern(pattern: &str, s: &str) -> bool {
    let s: Vec<char> = s.chars().filter(|c| !c.is_whitespace()).flat_map(|c| c.to_uppercase()).collect();
    let p: Vec<char> = pattern.chars().filter(|c| !c.is_whitespace()).collect();
    s.len() == p.len() && s.iter().zip(&p).all(|(c, m)| match m { 'A' => c.is_ascii_alphabetic(), '9' => c.is_ascii_digit(), l => c == l })
}

pub fn postal_ok(country: &str, code: &str) -> bool {
    let f = format_for(country);
    if f.postal_patterns.is_empty() { return code.chars().count() <= 12 && !code.chars().any(|c| c.is_control()); }
    if !f.postal_patterns.iter().any(|p| matches_pattern(p, code)) { return false; }
    if f.country == "CA" {
        let c: Vec<char> = code.chars().filter(|c| !c.is_whitespace()).flat_map(|c| c.to_uppercase()).collect();
        return !"DFIOQUWZ".contains(c[0]) && !"DFIOQU".contains(c[2]) && !"DFIOQU".contains(c[4]);
    }
    true
}

/// Met en forme les champs (majuscules des codes, séparateurs).
pub fn normalize(a: &Address) -> Address {
    let mut n = Address { country: a.country.trim().to_uppercase(), line1: a.line1.trim().into(), line2: a.line2.trim().into(), city: a.city.trim().into(),
        region: a.region.trim().into(), postal_code: a.postal_code.trim().to_uppercase() };
    let f = format_for(&n.country);
    if !f.regions.is_empty() { n.region = n.region.to_uppercase(); }
    if f.country == "CA" { let c: String = n.postal_code.chars().filter(|c| !c.is_whitespace()).collect(); if c.len() == 6 { n.postal_code = format!("{} {}", &c[..3], &c[3..]); } }
    if f.country == "GB" || f.country == "NL" { let c: String = n.postal_code.chars().filter(|c| !c.is_whitespace()).collect(); if c.len() > 3 { n.postal_code = format!("{} {}", &c[..c.len() - 3], &c[c.len() - 3..]); } }
    n
}

/// Validation : une adresse vide est valide (champ facultatif). Une adresse partielle n'est jamais refusée,
/// seules les valeurs renseignées sont contrôlées.
pub fn validate(a: &Address) -> Vec<FieldError> {
    let mut e = vec![];
    if a.is_empty() && a.country.trim().is_empty() { return e; }
    let country = a.country.trim().to_uppercase();
    if country.len() != 2 || country.parse::<phonenumber::country::Id>().is_err() { e.push(FieldError { field: "country", key: "error.required" }); return e; }
    let f = format_for(&country);
    if !a.postal_code.trim().is_empty() && !postal_ok(&country, &a.postal_code) { e.push(FieldError { field: "postal_code", key: "error.postal.invalid" }); }
    if !f.regions.is_empty() && !a.region.trim().is_empty() && !f.regions.contains(&a.region.trim().to_uppercase().as_str()) { e.push(FieldError { field: "region", key: "error.required" }); }
    for (field, v, max) in [("line1", &a.line1, 120), ("line2", &a.line2, 80), ("city", &a.city, 80), ("region", &a.region, 80)] {
        if !crate::validate::text(v, max) { e.push(FieldError { field, key: "error.too_long" }); }
    }
    e
}

/// Lignes d'une étiquette postale selon la convention du pays (sans le nom du pays).
pub fn format_lines(a: &Address) -> Vec<String> {
    let a = normalize(a);
    let f = format_for(&a.country);
    let join = |parts: &[&str]| parts.iter().filter(|p| !p.is_empty()).cloned().collect::<Vec<_>>().join(" ");
    let mut v: Vec<String> = vec![];
    match f.layout {
        Layout::Japan => { if !a.postal_code.is_empty() { v.push(format!("〒{}", a.postal_code)); } v.push(join(&[&a.region, &a.city])); v.push(join(&[&a.line1, &a.line2])); }
        Layout::CityRegionPostal => { v.push(a.line1.clone()); v.push(a.line2.clone()); v.push(join(&[&a.city, &a.region, &a.postal_code])); }
        Layout::PostalCity => { v.push(a.line1.clone()); v.push(a.line2.clone()); v.push(join(&[&a.postal_code, &a.city])); }
        Layout::CityThenPostal => { v.push(a.line1.clone()); v.push(a.line2.clone()); v.push(a.city.clone()); v.push(a.region.clone()); v.push(a.postal_code.clone()); }
    }
    v.into_iter().filter(|l| !l.trim().is_empty()).collect()
}

// ───────────────────────── Autocomplétion ─────────────────────────

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Suggestion { pub id: String, pub label: String }

#[derive(Debug, PartialEq, Eq)]
pub enum LookupError { ConsentRequired, Offline, QueryTooShort, Provider(String) }

/// Fournisseur d'autocomplétion externe (à choisir : voir ARCHITECTURE.md). Aucun fournisseur n'est intégré à ce jour.
pub trait AddressProvider {
    fn name(&self) -> &str;
    fn suggest(&self, query: &str, country: Option<&str>, limit: usize) -> Result<Vec<Suggestion>, LookupError>;
    fn resolve(&self, id: &str) -> Result<Address, LookupError>;
}

/// Fournisseur « absent » : toujours hors connexion → l'interface bascule sur la saisie manuelle.
pub struct OfflineProvider;
impl AddressProvider for OfflineProvider {
    fn name(&self) -> &str { "hors connexion" }
    fn suggest(&self, _: &str, _: Option<&str>, _: usize) -> Result<Vec<Suggestion>, LookupError> { Err(LookupError::Offline) }
    fn resolve(&self, _: &str) -> Result<Address, LookupError> { Err(LookupError::Offline) }
}

/// Porte d'accès : **aucune requête n'atteint le fournisseur sans consentement `AddressLookup`**.
pub struct Lookup<'a> { pub provider: &'a dyn AddressProvider, pub ledger: &'a ConsentLedger }
impl Lookup<'_> {
    pub fn suggest(&self, query: &str, country: Option<&str>) -> Result<Vec<Suggestion>, LookupError> {
        self.ledger.require(Purpose::AddressLookup).map_err(|_| LookupError::ConsentRequired)?;
        if query.trim().chars().count() < 3 { return Err(LookupError::QueryTooShort); }
        let mut v = self.provider.suggest(query.trim(), country, 8)?;
        v.truncate(8);
        Ok(v)
    }
    /// Résout une suggestion et retourne l'adresse normalisée avec les éventuelles erreurs à corriger.
    pub fn select(&self, id: &str) -> Result<(Address, Vec<FieldError>), LookupError> {
        self.ledger.require(Purpose::AddressLookup).map_err(|_| LookupError::ConsentRequired)?;
        let a = normalize(&self.provider.resolve(id)?);
        let errs = validate(&a);
        Ok((a, errs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    fn addr(c: &str, l1: &str, city: &str, r: &str, p: &str) -> Address { Address { country: c.into(), line1: l1.into(), line2: "".into(), city: city.into(), region: r.into(), postal_code: p.into() } }

    #[test]
    fn postal_codes_per_country() {
        for (c, ok) in [("CA", "H2Y 1C6"), ("CA", "h2y1c6"), ("US", "10001"), ("US", "10001-1234"), ("FR", "75001"), ("GB", "SW1A 1AA"), ("GB", "M1 1AE"), ("DE", "10115"),
                        ("PT", "1000-001"), ("NL", "1012 AB"), ("JP", "100-0001"), ("BR", "01310-100"), ("AU", "2000"), ("IE", "D02 X285"), ("ZZ", "whatever")] { assert!(postal_ok(c, ok), "{c} {ok}"); }
        for (c, bad) in [("CA", "D2Y 1C6"), ("CA", "H2Y 1CO"), ("CA", "12345"), ("US", "1234"), ("FR", "7500"), ("FR", "75O01"), ("GB", "12345"), ("PT", "1000001"), ("JP", "1000001")] { assert!(!postal_ok(c, bad), "{c} {bad}"); }
    }
    #[test]
    fn regions_and_labels() {
        let ca = format_for("CA");
        assert_eq!((ca.region_label_key(), ca.postal_label_key()), ("field.region.province", "field.postal_code"));
        assert_eq!(format_for("US").postal_label_key(), "field.zip_code");
        assert!(!format_for("FR").shows_region());
        assert!(validate(&addr("CA", "1 rue X", "Montréal", "QC", "H2Y 1C6")).is_empty());
        assert_eq!(validate(&addr("CA", "1 rue X", "Montréal", "XX", "H2Y 1C6")), vec![FieldError { field: "region", key: "error.required" }]);
        assert_eq!(validate(&addr("CA", "", "", "", "99999"))[0].key, "error.postal.invalid");
    }
    #[test]
    fn empty_and_unknown_are_never_rejected_wrongly() {
        assert!(validate(&Address::default()).is_empty(), "adresse facultative");
        assert!(validate(&addr("ZZ", "x", "y", "z", "")).is_empty() == false, "ZZ n'est pas un pays");
        assert!(validate(&addr("NO", "Karl Johans gate 1", "Oslo", "", "0154")).is_empty(), "pays sans format détaillé : jamais refusé");
        assert_eq!(validate(&addr("", "1 rue", "", "", ""))[0].field, "country");
    }
    #[test]
    fn normalisation() {
        let n = normalize(&addr("ca", " 1 rue X ", "Montréal", "qc", "h2y1c6"));
        assert_eq!((n.country.as_str(), n.region.as_str(), n.postal_code.as_str()), ("CA", "QC", "H2Y 1C6"));
        assert_eq!(normalize(&addr("GB", "", "", "", "sw1a1aa")).postal_code, "SW1A 1AA");
    }
    #[test]
    fn postal_layouts_follow_local_conventions() {
        assert_eq!(format_lines(&addr("CA", "1 rue X", "Montréal", "qc", "h2y1c6")), vec!["1 rue X", "Montréal QC H2Y 1C6"]);
        assert_eq!(format_lines(&addr("FR", "1 rue de Rivoli", "Paris", "", "75001")), vec!["1 rue de Rivoli", "75001 Paris"]);
        assert_eq!(format_lines(&addr("GB", "10 Downing Street", "London", "", "sw1a2aa")), vec!["10 Downing Street", "London", "SW1A 2AA"]);
        assert_eq!(format_lines(&addr("JP", "1-1", "Chiyoda", "Tokyo", "100-0001")), vec!["〒100-0001", "Tokyo Chiyoda", "1-1"]);
    }

    struct Mock { calls: Cell<u32> }
    impl AddressProvider for Mock {
        fn name(&self) -> &str { "mock" }
        fn suggest(&self, q: &str, _: Option<&str>, _: usize) -> Result<Vec<Suggestion>, LookupError> {
            self.calls.set(self.calls.get() + 1);
            Ok((0..12).map(|i| Suggestion { id: format!("id{i}"), label: format!("{q} #{i}") }).collect())
        }
        fn resolve(&self, _: &str) -> Result<Address, LookupError> { self.calls.set(self.calls.get() + 1); Ok(addr("ca", "1 rue Sainte-Catherine", "Montréal", "qc", "h2x1k4")) }
    }

    #[test]
    fn provider_is_never_called_without_consent() {
        let (m, ledger) = (Mock { calls: Cell::new(0) }, ConsentLedger::default());
        let l = Lookup { provider: &m, ledger: &ledger };
        assert_eq!(l.suggest("1 rue Sainte", Some("CA")), Err(LookupError::ConsentRequired));
        assert_eq!(l.select("id1"), Err(LookupError::ConsentRequired));
        assert_eq!(m.calls.get(), 0, "aucune donnée saisie n'a atteint le fournisseur");
    }
    #[test]
    fn suggestions_then_selection_fill_the_form() {
        let (m, mut ledger) = (Mock { calls: Cell::new(0) }, ConsentLedger::default());
        ledger.grant(Purpose::AddressLookup, "mock", false, false, 1).unwrap();
        let l = Lookup { provider: &m, ledger: &ledger };
        assert_eq!(l.suggest("ab", None), Err(LookupError::QueryTooShort));
        let s = l.suggest("1 rue Sainte", None).unwrap();
        assert_eq!(s.len(), 8, "résultats bornés");
        let (a, errs) = l.select(&s[0].id).unwrap();
        assert!(errs.is_empty());
        assert_eq!((a.region.as_str(), a.postal_code.as_str()), ("QC", "H2X 1K4"));
    }
    #[test]
    fn offline_falls_back_to_manual_entry() {
        let mut ledger = ConsentLedger::default();
        ledger.grant(Purpose::AddressLookup, "x", false, false, 1).unwrap();
        assert_eq!(Lookup { provider: &OfflineProvider, ledger: &ledger }.suggest("1 rue X", None), Err(LookupError::Offline));
        // la saisie manuelle reste entièrement validée localement
        assert!(validate(&addr("FR", "1 rue X", "Lyon", "", "69001")).is_empty());
    }
}
