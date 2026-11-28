//! Taxes **configurables** : aucun taux n'est imposé. Les préréglages sont des points de départ
//! à faire confirmer par un comptable ; les taux légaux changent.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tax { pub name: String, /// Taux en parties par million (1 % = 10 000 ; 9,975 % = 99 750).
    pub rate_ppm: u32, /// Taxe appliquée sur le prix + taxes précédentes (cumulative) ?
    pub compound: bool }

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaxConfig { pub taxes: Vec<Tax>, pub prices_include_tax: bool }

#[derive(Debug, PartialEq, Eq)]
pub struct Breakdown { pub net: i64, pub taxes: Vec<(String, i64)>, pub total: i64 }

fn pct(base: i64, ppm: u32) -> i64 { (base * ppm as i64 + 500_000) / 1_000_000 } // arrondi au plus proche (montants positifs)

impl TaxConfig {
    /// Calcule les taxes sur un montant HT en cents.
    pub fn on_net(&self, net: i64) -> Breakdown {
        let mut running = net;
        let mut out = vec![];
        for t in &self.taxes {
            let base = if t.compound { running } else { net };
            let amt = pct(base, t.rate_ppm);
            running += amt;
            out.push((t.name.clone(), amt));
        }
        Breakdown { net, taxes: out, total: running }
    }
    /// Extrait le HT d'un montant TTC (par recherche, exact au cent près).
    pub fn net_from_gross(&self, gross: i64) -> Breakdown {
        let (mut lo, mut hi) = (0i64, gross);
        while lo < hi { let mid = (lo + hi + 1) / 2; if self.on_net(mid).total <= gross { lo = mid } else { hi = mid - 1 } }
        self.on_net(lo)
    }
}

/// Préréglages d'exemple — **à vérifier avec un comptable avant usage**.
pub fn preset(name: &str) -> Option<TaxConfig> {
    let t = |n: &str, ppm, c| Tax { name: n.into(), rate_ppm: ppm, compound: c };
    Some(TaxConfig { prices_include_tax: false, taxes: match name {
        "CA-QC" => vec![t("TPS", 50_000, false), t("TVQ", 99_750, false)],
        "CA-ON" => vec![t("TVH", 130_000, false)],
        "none" => vec![],
        _ => return None,
    } })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn simple_and_compound() {
        let c = TaxConfig { taxes: vec![Tax { name: "A".into(), rate_ppm: 50_000, compound: false }, Tax { name: "B".into(), rate_ppm: 100_000, compound: true }], prices_include_tax: false };
        let b = c.on_net(10_000);
        assert_eq!(b.taxes, vec![("A".into(), 500), ("B".into(), 1050)]);
        assert_eq!(b.total, 11_550);
    }
    #[test]
    fn gross_to_net_is_consistent() {
        let c = preset("CA-ON").unwrap();
        for gross in [113, 1_130, 9_999, 123_457] {
            let b = c.net_from_gross(gross);
            assert!(b.total <= gross && c.on_net(b.net + 1).total > gross);
        }
    }
    #[test]
    fn quebec_preset_uses_exact_rate() {
        let b = preset("CA-QC").unwrap().on_net(10_000);
        assert_eq!(b.taxes, vec![("TPS".into(), 500), ("TVQ".into(), 998)]); // 997,5 ¢ arrondi à 998 ¢
    }
    #[test]
    fn no_tax_and_unknown_preset() {
        assert_eq!(preset("none").unwrap().on_net(500).total, 500);
        assert!(preset("zz").is_none());
    }
}
