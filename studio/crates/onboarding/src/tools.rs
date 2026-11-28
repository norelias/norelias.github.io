//! Registre unique des outils. Source de vérité pour les espaces de travail, l'aide au survol
//! et la matrice des fonctionnalités. Un outil `Planned` n'est **jamais** présenté comme utilisable.

use crate::profile::Mode;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// Moteur réel intégré et testé.
    Available,
    /// Prévu : visible uniquement dans la feuille de route, jamais comme bouton actif.
    Planned,
}

#[derive(Clone, Copy, Debug)]
pub struct ToolDef {
    pub id: &'static str,
    pub panel: &'static str,
    /// Niveau à partir duquel l'outil est mis en avant par défaut.
    pub prominent_from: Mode,
    pub status: Status,
}

const fn t(id: &'static str, panel: &'static str, prominent_from: Mode, status: Status) -> ToolDef {
    ToolDef { id, panel, prominent_from, status }
}
use Mode::{Beginner as B, Expert as E, Intermediate as I};
use Status::{Available as A, Planned as P};

pub static TOOLS: &[ToolDef] = &[
    // Moteur réel (norelias-core) — les dix opérations
    t("exposure", "basic", B, A), t("contrast", "basic", B, A), t("white_balance", "basic", B, A),
    t("saturation", "basic", B, A), t("highlights", "tone", I, A), t("shadows", "tone", I, A),
    t("denoise", "detail", I, A), t("sharpen", "detail", I, A), t("rotate", "geometry", B, A), t("crop", "geometry", B, A),
    // Assistance réelle
    t("advisor", "assistant", B, A), t("academy", "help", B, A),
    // Prévus (aucun moteur intégré à ce jour)
    t("histogram", "basic", B, P), t("tone_curve", "tone", E, P), t("hsl", "color", I, P), t("color_grading", "color", E, P),
    t("lens_correction", "geometry", I, P), t("local_masks", "masks", I, P), t("layers", "layers", E, P),
    t("portrait_skin", "portrait", I, P), t("dodge_burn", "portrait", E, P), t("ai_denoise", "ai", I, P),
    t("object_removal", "ai", I, P), t("catalog", "library", B, P), t("export", "export", B, P),
    t("batch", "export", E, P), t("print", "print", I, P), t("gallery", "client", I, P), t("tethering", "capture", E, P),
];

pub fn get(id: &str) -> Option<&'static ToolDef> { TOOLS.iter().find(|t| t.id == id) }
pub fn available() -> impl Iterator<Item = &'static ToolDef> { TOOLS.iter().filter(|t| t.status == Status::Available) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ids_are_unique() {
        let mut ids: Vec<_> = TOOLS.iter().map(|t| t.id).collect();
        ids.sort();
        let n = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), n);
    }
    #[test]
    fn every_engine_operation_has_an_available_tool_and_vice_versa() {
        for name in norelias_core::Op::NAMES {
            assert_eq!(get(name).map(|t| t.status), Some(Status::Available), "opération moteur sans outil : {name}");
        }
        // Tout outil « disponible » est soit une opération du moteur, soit une aide réelle.
        for t in available() {
            assert!(norelias_core::Op::NAMES.contains(&t.id) || ["advisor", "academy"].contains(&t.id), "outil disponible sans moteur : {}", t.id);
        }
    }
}
