//! Internationalisation de Norélias Studio Pro : catalogue central des textes, formats de nombres,
//! monnaies et dates, unités, taxes configurables, sens d'écriture. Aucun texte d'interface ne doit
//! être écrit dans un composant : il passe par `Catalog::t`.
//!
//! Lancement : français et anglais. Les autres langues se *branchent* en ajoutant un fichier
//! `locales/<langue>.json` ; les formats numériques sont déjà prévus pour es, de, it, pt.
//! Les crates plus anciennes (conseiller, Académie, questionnaire) ont encore des textes intégrés :
//! leur migration vers ce catalogue est inscrite dans la feuille de route.

pub mod format;
pub mod tax;
pub mod units;

use std::collections::{BTreeMap, BTreeSet};

pub use norelias_core::Lang;

/// Langue de l'interface (code BCP 47 principal).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Locale { pub lang: String, pub region: Option<String> }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction { Ltr, Rtl }

impl Locale {
    pub fn new(lang: &str, region: Option<&str>) -> Self { Self { lang: lang.to_lowercase(), region: region.map(|r| r.to_uppercase()) } }
    /// Accepte « fr », « fr-CA », « fr_CA.UTF-8 ».
    pub fn parse(tag: &str) -> Option<Self> {
        let t = tag.split('.').next()?.replace('_', "-");
        let mut it = t.split('-');
        let lang = it.next()?.trim();
        if lang.len() < 2 || lang.len() > 3 || !lang.chars().all(|c| c.is_ascii_alphabetic()) { return None; }
        let region = it.find(|p| p.len() == 2 && p.chars().all(|c| c.is_ascii_alphabetic()));
        Some(Self::new(lang, region))
    }
    pub fn direction(&self) -> Direction {
        if ["ar", "he", "fa", "ur", "ps", "yi", "dv"].contains(&self.lang.as_str()) { Direction::Rtl } else { Direction::Ltr }
    }
    pub fn tag(&self) -> String { match &self.region { Some(r) => format!("{}-{}", self.lang, r), None => self.lang.clone() } }
}

/// Catalogue de traductions avec repli : langue demandée → anglais → clé.
pub struct Catalog { langs: BTreeMap<String, BTreeMap<String, String>> }

impl Catalog {
    pub fn builtin() -> Self {
        let mut c = Catalog { langs: BTreeMap::new() };
        c.add("fr", include_str!("../locales/fr.json")).expect("fr.json valide");
        c.add("en", include_str!("../locales/en.json")).expect("en.json valide");
        c
    }
    pub fn add(&mut self, lang: &str, json: &str) -> Result<(), serde_json::Error> {
        let m: BTreeMap<String, String> = serde_json::from_str(json)?;
        self.langs.insert(lang.to_string(), m);
        Ok(())
    }
    pub fn languages(&self) -> Vec<&str> { self.langs.keys().map(String::as_str).collect() }
    pub fn has_key(&self, key: &str) -> bool { self.langs.values().any(|m| m.contains_key(key)) }
    pub fn keys(&self) -> BTreeSet<&str> { self.langs.values().flat_map(|m| m.keys().map(String::as_str)).collect() }

    pub fn t(&self, loc: &Locale, key: &str) -> String {
        self.langs.get(&loc.lang).and_then(|m| m.get(key))
            .or_else(|| self.langs.get("en").and_then(|m| m.get(key)))
            .cloned().unwrap_or_else(|| key.to_string())
    }
    /// Remplace `{nom}` par la valeur fournie.
    pub fn t_args(&self, loc: &Locale, key: &str, args: &[(&str, &str)]) -> String {
        args.iter().fold(self.t(loc, key), |s, (k, v)| s.replace(&format!("{{{k}}}"), v))
    }

    /// Contrôle de cohérence : clés manquantes par langue et paramètres `{x}` différents entre langues.
    pub fn audit(&self) -> Vec<String> {
        let all = self.keys();
        let mut issues = vec![];
        for (lang, m) in &self.langs {
            for k in &all { if !m.contains_key(*k) { issues.push(format!("{lang}: clé manquante {k}")); } }
        }
        let params = |s: &str| -> BTreeSet<String> {
            s.split('{').skip(1).filter_map(|p| p.split('}').next()).map(String::from).collect()
        };
        for k in &all {
            let sets: Vec<_> = self.langs.values().filter_map(|m| m.get(*k)).map(|s| params(s)).collect();
            if sets.windows(2).any(|w| w[0] != w[1]) { issues.push(format!("paramètres incohérents pour {k}")); }
        }
        issues
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catalogs_are_in_sync() { assert_eq!(Catalog::builtin().audit(), Vec::<String>::new()); }
    #[test]
    fn fallback_chain() {
        let c = Catalog::builtin();
        let es = Locale::parse("es-MX").unwrap();
        assert_eq!(c.t(&es, "help.close"), "Close", "langue absente : repli sur l'anglais");
        assert_eq!(c.t(&Locale::new("fr", None), "help.close"), "Fermer");
        assert_eq!(c.t(&es, "cle.inexistante"), "cle.inexistante");
    }
    #[test]
    fn arguments_are_substituted() {
        let c = Catalog::builtin();
        assert_eq!(c.t_args(&Locale::new("fr", None), "auth.locked", &[("seconds", "30")]), "Trop de tentatives. Réessayez dans 30 secondes.");
    }
    #[test]
    fn locale_parsing_and_direction() {
        assert_eq!(Locale::parse("fr_CA.UTF-8").unwrap().tag(), "fr-CA");
        assert!(Locale::parse("x").is_none() && Locale::parse("12-34").is_none());
        assert_eq!(Locale::parse("ar-EG").unwrap().direction(), Direction::Rtl);
        assert_eq!(Locale::parse("he").unwrap().direction(), Direction::Rtl);
        assert_eq!(Locale::parse("fr").unwrap().direction(), Direction::Ltr);
    }
    #[test]
    fn adding_a_language_needs_only_a_file() {
        let mut c = Catalog::builtin();
        c.add("es", r#"{"help.close":"Cerrar"}"#).unwrap();
        assert_eq!(c.t(&Locale::new("es", None), "help.close"), "Cerrar");
        assert!(!c.audit().is_empty(), "l'audit signale les clés espagnoles manquantes");
    }
}
