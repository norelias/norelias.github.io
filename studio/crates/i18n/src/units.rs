//! Unités de mesure métriques et impériales (affichage ; le stockage interne reste en millimètres).

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Units { Metric, Imperial }

pub const MM_PER_INCH: f64 = 25.4;

/// Dimension d'impression pour affichage : (valeur, symbole).
pub fn length_display(mm: f64, units: Units) -> (f64, &'static str) {
    match units { Units::Metric => (mm / 10.0, "cm"), Units::Imperial => (mm / MM_PER_INCH, "in") }
}
/// Saisie utilisateur → millimètres.
pub fn length_to_mm(value: f64, units: Units) -> f64 {
    match units { Units::Metric => value * 10.0, Units::Imperial => value * MM_PER_INCH }
}
/// Résolution : pixels par pouce ↔ pixels par cm.
pub fn ppi_to_ppcm(ppi: f64) -> f64 { ppi / 2.54 }
/// Pixels nécessaires pour imprimer `mm` à `ppi`.
pub fn pixels_for(mm: f64, ppi: f64) -> u32 { (mm / MM_PER_INCH * ppi).round() as u32 }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn conversions_roundtrip() {
        for u in [Units::Metric, Units::Imperial] {
            let (v, _) = length_display(254.0, u);
            assert!((length_to_mm(v, u) - 254.0).abs() < 1e-9);
        }
        assert_eq!(length_display(254.0, Units::Imperial), (10.0, "in"));
        assert_eq!(length_display(254.0, Units::Metric), (25.4, "cm"));
    }
    #[test]
    fn print_pixels() {
        // 8×10 po à 300 ppp = 2400×3000 px
        assert_eq!(pixels_for(8.0 * MM_PER_INCH, 300.0), 2400);
        assert_eq!(pixels_for(10.0 * MM_PER_INCH, 300.0), 3000);
        assert!((ppi_to_ppcm(300.0) - 118.11).abs() < 0.01);
    }
}
