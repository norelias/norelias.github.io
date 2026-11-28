//! SPF, DKIM et DMARC : génération des enregistrements DNS à publier et **validation de leur syntaxe**.
//!
//! Limite : ce module ne publie rien et n'interroge aucun DNS. Les enregistrements doivent être créés chez le
//! gestionnaire DNS du domaine (accès que seul son propriétaire possède) ; la valeur DKIM (clé publique) est fournie par le
//! fournisseur d'envoi. La vérification en conditions réelles se fait ensuite avec `dig` et les outils du fournisseur (voir ARCHITECTURE.md §9).

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DmarcPolicy { None, Quarantine, Reject }
impl DmarcPolicy { fn word(self) -> &'static str { match self { Self::None => "none", Self::Quarantine => "quarantine", Self::Reject => "reject" } } }

#[derive(Clone, Debug)]
pub struct DomainPlan {
    /// Domaine d'envoi. Recommandé : un sous-domaine dédié (ex. `mail.norelias.ca`) pour isoler la réputation.
    pub domain: String,
    /// Mécanismes `include:` fournis par le fournisseur d'envoi.
    pub spf_includes: Vec<String>,
    pub dkim_selector: String,
    pub dkim_public_key_b64: String,
    pub dmarc_rua: String,
    pub dmarc_policy: DmarcPolicy,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record { pub name: String, pub rtype: &'static str, pub value: String }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Issue { pub record: &'static str, pub message: String }
fn issue(record: &'static str, m: impl Into<String>) -> Issue { Issue { record, message: m.into() } }

impl DomainPlan {
    pub fn records(&self) -> Vec<Record> {
        let spf = format!("v=spf1 {}-all", self.spf_includes.iter().map(|i| format!("include:{i} ")).collect::<String>());
        vec![
            Record { name: self.domain.clone(), rtype: "TXT", value: spf },
            Record { name: format!("{}._domainkey.{}", self.dkim_selector, self.domain), rtype: "TXT", value: format!("v=DKIM1; k=rsa; p={}", self.dkim_public_key_b64) },
            Record { name: format!("_dmarc.{}", self.domain), rtype: "TXT", value: format!("v=DMARC1; p={}; rua=mailto:{}; adkim=s; aspf=s; pct=100", self.dmarc_policy.word(), self.dmarc_rua) },
        ]
    }
}

pub fn validate_spf(txt: &str) -> Vec<Issue> {
    let mut v = vec![];
    if txt.len() > 255 { v.push(issue("SPF", "chaîne TXT > 255 caractères : à découper en plusieurs chaînes")); }
    let mut it = txt.split_whitespace();
    if it.next() != Some("v=spf1") { v.push(issue("SPF", "doit commencer par « v=spf1 »")); return v; }
    let mut lookups = 0;
    let mut all: Option<char> = None;
    for t in it {
        let m = t.trim_start_matches(['+', '-', '~', '?']);
        if let Some(q) = t.strip_suffix("all").filter(|q| q.len() <= 1 && !t.contains(':')) { all = Some(q.chars().next().unwrap_or('+')); }
        else if ["include:", "a", "mx", "ptr", "exists:", "redirect="].iter().any(|p| m == *p || m.starts_with(p) && (p.ends_with(':') || p.ends_with('=') || m.starts_with(&format!("{p}:")) || m.starts_with(&format!("{p}/")))) { lookups += 1; if m.starts_with("ptr") { v.push(issue("SPF", "« ptr » est déconseillé")); } }
        else if !(m.starts_with("ip4:") || m.starts_with("ip6:")) { v.push(issue("SPF", format!("mécanisme inconnu : {t}"))); }
    }
    if lookups > 10 { v.push(issue("SPF", format!("{lookups} recherches DNS (> 10) : SPF invalide (permerror)"))); }
    match all { None => v.push(issue("SPF", "« -all » ou « ~all » final manquant")), Some('+') => v.push(issue("SPF", "« +all » autorise n'importe qui : dangereux")), Some('?') => v.push(issue("SPF", "« ?all » n'applique aucune politique")), _ => {} }
    v
}

pub fn validate_dmarc(txt: &str) -> Vec<Issue> {
    let mut v = vec![];
    let tags: Vec<(&str, &str)> = txt.split(';').filter_map(|t| t.trim().split_once('=')).map(|(k, x)| (k.trim(), x.trim())).collect();
    if tags.first() != Some(&("v", "DMARC1")) { v.push(issue("DMARC", "« v=DMARC1 » doit être la première balise")); return v; }
    let get = |k: &str| tags.iter().find(|t| t.0 == k).map(|t| t.1);
    match get("p") { Some("none") => v.push(issue("DMARC", "p=none : surveillance seulement, aucune protection (étape de départ acceptable)")), Some("quarantine") | Some("reject") => {}, _ => v.push(issue("DMARC", "balise p absente ou invalide")) }
    if let Some(p) = get("pct") { if !p.parse::<u32>().is_ok_and(|n| n <= 100) { v.push(issue("DMARC", "pct doit être entre 0 et 100")); } }
    if let Some(r) = get("rua") { for a in r.split(',') { if !a.trim().strip_prefix("mailto:").is_some_and(|e| e.contains('@') && e.contains('.')) { v.push(issue("DMARC", format!("rua invalide : {a}"))); } } } else { v.push(issue("DMARC", "rua absent : aucun rapport agrégé ne sera reçu")); }
    for k in ["adkim", "aspf"] { if let Some(x) = get(k) { if x != "r" && x != "s" { v.push(issue("DMARC", format!("{k} doit valoir r ou s"))); } } }
    v
}

fn b64_decoded_len(s: &str) -> Option<usize> {
    let s = s.trim_end_matches('=');
    if s.is_empty() || !s.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/') { return None; }
    Some(s.len() * 3 / 4)
}

pub fn validate_dkim(txt: &str) -> Vec<Issue> {
    let mut v = vec![];
    let tags: Vec<(&str, String)> = txt.split(';').filter_map(|t| t.trim().split_once('=')).map(|(k, x)| (k.trim(), x.split_whitespace().collect::<String>())).collect();
    let get = |k: &str| tags.iter().find(|t| t.0 == k).map(|t| t.1.as_str());
    if let Some(ver) = get("v") { if ver != "DKIM1" { v.push(issue("DKIM", "v doit valoir DKIM1")); } }
    let k = get("k").unwrap_or("rsa");
    if k != "rsa" && k != "ed25519" { v.push(issue("DKIM", "k doit valoir rsa ou ed25519")); }
    match get("p") {
        None => v.push(issue("DKIM", "balise p (clé publique) absente")),
        Some("") => v.push(issue("DKIM", "p vide : clé révoquée")),
        Some(p) => match b64_decoded_len(p) {
            None => v.push(issue("DKIM", "clé publique : base64 invalide")),
            Some(n) if k == "rsa" && n < 160 => v.push(issue("DKIM", "clé RSA trop courte")),
            Some(n) if k == "rsa" && n < 270 => v.push(issue("DKIM", "clé RSA 1024 bits : faible, 2048 bits recommandés")),
            _ => {}
        },
    }
    v
}

/// Compare les enregistrements trouvés dans le DNS (obtenus par l'appelant) au plan.
pub fn check_published(plan: &DomainPlan, found: &[(String, String)]) -> Vec<Issue> {
    let mut v = vec![];
    let at = |name: &str| -> Vec<&String> { found.iter().filter(|f| f.0.eq_ignore_ascii_case(name)).map(|f| &f.1).collect() };
    let spf: Vec<_> = at(&plan.domain).into_iter().filter(|t| t.starts_with("v=spf1")).collect();
    match spf.len() { 0 => v.push(issue("SPF", "aucun enregistrement SPF publié")), 1 => v.extend(validate_spf(spf[0])), _ => v.push(issue("SPF", "plusieurs enregistrements SPF : permerror")) }
    match at(&format!("{}._domainkey.{}", plan.dkim_selector, plan.domain)).first() { None => v.push(issue("DKIM", "aucune clé DKIM publiée pour ce sélecteur")), Some(t) => v.extend(validate_dkim(t)) }
    match at(&format!("_dmarc.{}", plan.domain)).first() { None => v.push(issue("DMARC", "aucun enregistrement DMARC publié")), Some(t) => v.extend(validate_dmarc(t)) }
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    const KEY2048: &str = "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAu1SU1LfVLPHCozMxH2Mo4lgOEePzNm0tRgeLezV6ffAt0gunVTLw7onLRnrq0/IzW7yWR7QkrmBL7jTKEn5u+qKhbwKfBstIs+bMY2Zkp18gnTxKLxoS2tFczGkPLPgizskuemMghRniWaoLcyehkd3qqGElvW/VDL5AaWTg0nLVkjRo9z+40RQzuVaE8AkAFmxZzow3x+VJYKdjykkJ0iT9wCS0DRTXu269V264Vf/3jvredZiKRkgwlL9xNAwxXFg0x/XFw005UWVRIkdgcKWTjpBP2dPwVZ4WWC+9aGVd+Gyn1o0CLelf4rEjGoXbAAEgAqeGUxrcIlbjXfbcmwIDAQAB";
    fn plan() -> DomainPlan { DomainPlan { domain: "mail.norelias.ca".into(), spf_includes: vec!["spf.exemple-fournisseur.net".into()], dkim_selector: "nrl1".into(), dkim_public_key_b64: KEY2048.into(),
        dmarc_rua: "dmarc@norelias.ca".into(), dmarc_policy: DmarcPolicy::Quarantine } }

    #[test]
    fn generated_records_are_valid_and_complete() {
        let r = plan().records();
        assert_eq!(r.len(), 3);
        assert_eq!(r[1].name, "nrl1._domainkey.mail.norelias.ca");
        assert_eq!(r[2].name, "_dmarc.mail.norelias.ca");
        assert!(validate_spf(&r[0].value).is_empty(), "{:?}", validate_spf(&r[0].value));
        assert!(validate_dkim(&r[1].value).is_empty());
        assert!(validate_dmarc(&r[2].value).is_empty());
        assert!(r[0].value.ends_with("-all") && r[2].value.contains("adkim=s"));
    }
    #[test]
    fn spf_problems_are_detected() {
        assert!(validate_spf("v=spf1 +all").iter().any(|i| i.message.contains("dangereux")));
        assert!(!validate_spf("v=spf1 include:a.net").is_empty());
        assert!(validate_spf("spf1 -all")[0].message.contains("commencer"));
        let many = format!("v=spf1 {}-all", (0..11).map(|i| format!("include:s{i}.net ")).collect::<String>());
        assert!(validate_spf(&many).iter().any(|i| i.message.contains("> 10")));
        assert!(validate_spf("v=spf1 foo:bar -all").iter().any(|i| i.message.contains("inconnu")));
        assert!(validate_spf("v=spf1 ip4:203.0.113.0/24 mx ~all").is_empty());
        assert!(validate_spf(&format!("v=spf1 ip4:1.2.3.4 {}-all", "ip4:5.6.7.8 ".repeat(25))).iter().any(|i| i.message.contains("255")));
    }
    #[test]
    fn dmarc_problems_are_detected() {
        assert!(validate_dmarc("p=reject; v=DMARC1")[0].message.contains("première"));
        assert!(validate_dmarc("v=DMARC1; p=none; rua=mailto:a@b.co").iter().any(|i| i.message.contains("surveillance")));
        assert!(validate_dmarc("v=DMARC1; p=bogus").iter().any(|i| i.message.contains("invalide")));
        assert!(validate_dmarc("v=DMARC1; p=reject; pct=150; rua=mailto:a@b.co").iter().any(|i| i.message.contains("pct")));
        assert!(validate_dmarc("v=DMARC1; p=reject; rua=a@b.co").iter().any(|i| i.message.contains("rua")));
        assert!(validate_dmarc("v=DMARC1; p=reject; rua=mailto:a@b.co; aspf=x").iter().any(|i| i.message.contains("aspf")));
        assert!(validate_dmarc("v=DMARC1; p=reject").iter().any(|i| i.message.contains("aucun rapport")));
    }
    #[test]
    fn dkim_problems_are_detected() {
        assert!(validate_dkim("v=DKIM1; k=rsa; p=").iter().any(|i| i.message.contains("révoquée")));
        assert!(validate_dkim("v=DKIM1; k=rsa").iter().any(|i| i.message.contains("absente")));
        assert!(validate_dkim("v=DKIM1; k=dsa; p=AAAA").iter().any(|i| i.message.contains("rsa ou ed25519")));
        assert!(validate_dkim("k=rsa; p=@@@").iter().any(|i| i.message.contains("base64")));
        assert!(validate_dkim(&format!("v=DKIM1; p={}", &KEY2048[..220])).iter().any(|i| i.message.contains("1024") || i.message.contains("courte")));
        assert!(validate_dkim(&format!("v=DKIM1; k=rsa; p={}", KEY2048.as_bytes().chunks(60).map(|c| std::str::from_utf8(c).unwrap()).collect::<Vec<_>>().join(" "))).is_empty(), "espaces dans p tolérés (chaînes TXT concaténées)");
    }
    #[test]
    fn published_records_are_compared_to_the_plan() {
        let p = plan();
        let good: Vec<(String, String)> = p.records().into_iter().map(|r| (r.name, r.value)).collect();
        assert!(check_published(&p, &good).is_empty());
        assert_eq!(check_published(&p, &[]).len(), 3, "rien de publié : trois problèmes");
        let mut dup = good.clone();
        dup.push((p.domain.clone(), "v=spf1 -all".into()));
        assert!(check_published(&p, &dup).iter().any(|i| i.message.contains("plusieurs")));
    }
}
