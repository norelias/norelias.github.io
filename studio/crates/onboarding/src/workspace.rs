//! Espaces de travail adaptatifs. Les trois modes pilotent la **présentation** uniquement :
//! même moteur, mêmes outils. `Visibility` n'a volontairement aucune variante « supprimé ».

use crate::profile::{Mode, Specialty};
use crate::tools::{Status, TOOLS};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Visibility {
    /// Visible et déployé.
    Prominent,
    /// Visible mais replié.
    Collapsed,
    /// Accessible par recherche d'outils, menu « Plus d'outils » et raccourci.
    OnDemand,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsDetail { Essential, Standard, Full }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Explanations { Plain, Concise, None }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Recommendations { Proactive, OnRequest, Badge }

#[derive(Clone, Debug)]
pub struct Layout {
    pub mode: Mode,
    pub specialty: Option<Specialty>,
    pub tools: Vec<(&'static str, Visibility, Status)>,
    pub settings_detail: SettingsDetail,
    pub explanations: Explanations,
    pub shortcut_hints: bool,
    pub recommendations: Recommendations,
    pub tutorial_prompts: bool,
}

/// Outils mis en avant pour une spécialité (promotion, jamais exclusion).
pub fn pins(s: Specialty) -> &'static [&'static str] {
    use Specialty::*;
    match s {
        Portrait | Studio => &["advisor", "portrait_skin", "dodge_burn", "local_masks", "white_balance"],
        Wedding | Events | Family | Newborn => &["white_balance", "highlights", "shadows", "portrait_skin", "batch", "denoise"],
        Landscape => &["rotate", "highlights", "shadows", "tone_curve", "hsl", "crop"],
        Wildlife | Sports => &["denoise", "sharpen", "crop", "exposure", "batch"],
        Product | RealEstate => &["white_balance", "rotate", "lens_correction", "crop", "batch"],
        Street => &["contrast", "crop", "rotate", "tone_curve"],
        School => &["batch", "white_balance", "crop", "print", "gallery"],
        Other => &[],
    }
}

pub fn visibility(tool_prominent_from: Mode, mode: Mode, pinned: bool) -> Visibility {
    if pinned || mode == Mode::Expert { return Visibility::Prominent; }
    if tool_prominent_from <= mode { return Visibility::Prominent; }
    // un cran au-dessus du niveau courant : replié ; plus haut : à la demande
    if tool_prominent_from as u8 - mode as u8 == 1 { Visibility::Collapsed } else { Visibility::OnDemand }
}

impl Layout {
    pub fn build(mode: Mode, specialties: &[Specialty], aids_enabled: bool) -> Layout {
        let primary = specialties.first().copied();
        let pinned: Vec<&str> = specialties.iter().flat_map(|s| pins(*s).iter().copied()).collect();
        let mut tools: Vec<_> = TOOLS.iter()
            .map(|t| (t.id, visibility(t.prominent_from, mode, pinned.contains(&t.id)), t.status))
            .collect();
        // les outils promus par la spécialité passent en tête
        tools.sort_by_key(|(id, v, _)| (*v != Visibility::Prominent, !pinned.contains(id)));
        Layout {
            mode,
            specialty: primary,
            tools,
            settings_detail: match mode { Mode::Beginner => SettingsDetail::Essential, Mode::Intermediate => SettingsDetail::Standard, Mode::Expert => SettingsDetail::Full },
            explanations: match (aids_enabled, mode) { (false, _) | (_, Mode::Expert) => Explanations::None, (true, Mode::Beginner) => Explanations::Plain, (true, Mode::Intermediate) => Explanations::Concise },
            shortcut_hints: aids_enabled && mode != Mode::Beginner,
            recommendations: match (aids_enabled, mode) { (true, Mode::Beginner) => Recommendations::Proactive, (true, Mode::Intermediate) => Recommendations::OnRequest, _ => Recommendations::Badge },
            tutorial_prompts: aids_enabled && mode == Mode::Beginner,
        }
    }
    /// Identifiants de tous les outils accessibles (quel que soit le mode).
    pub fn reachable(&self) -> Vec<&'static str> { self.tools.iter().map(|t| t.0).collect() }
    /// Outils réellement utilisables maintenant, dans l'ordre de présentation.
    pub fn usable(&self) -> Vec<&'static str> { self.tools.iter().filter(|t| t.2 == Status::Available).map(|t| t.0).collect() }
    /// Outils affichés comme actifs : uniquement des outils réels, mis en avant.
    pub fn prominent_usable(&self) -> Vec<&'static str> {
        self.tools.iter().filter(|t| t.2 == Status::Available && t.1 == Visibility::Prominent).map(|t| t.0).collect()
    }
    pub fn visibility_of(&self, id: &str) -> Option<Visibility> { self.tools.iter().find(|t| t.0 == id).map(|t| t.1) }
}

#[derive(Clone, Copy, Debug)]
pub struct WorkspaceDef { pub id: &'static str, pub fr: &'static str, pub en: &'static str }

/// Nom des espaces de travail spécialisés.
pub fn workspace_for(s: Specialty) -> WorkspaceDef {
    use Specialty::*;
    let (id, fr, en) = match s {
        Portrait => ("portrait", "Portrait", "Portrait"), Wedding => ("wedding", "Mariage", "Wedding"),
        Family => ("family", "Famille", "Family"), Newborn => ("newborn", "Nouveau-né", "Newborn"),
        Landscape => ("landscape", "Paysage", "Landscape"), Wildlife => ("wildlife", "Animalier", "Wildlife"),
        Sports => ("sports", "Sport", "Sports"), Events => ("events", "Événementiel", "Events"),
        Product => ("product", "Produit", "Product"), RealEstate => ("realestate", "Immobilier", "Real estate"),
        Street => ("street", "Rue", "Street"), Studio => ("studio", "Studio", "Studio"),
        School => ("school", "Scolaire", "School"), Other => ("general", "Général", "General"),
    };
    WorkspaceDef { id, fr, en }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn all_modes() -> [Mode; 3] { [Mode::Beginner, Mode::Intermediate, Mode::Expert] }

    #[test]
    fn no_mode_or_specialty_ever_removes_a_tool() {
        let reference: BTreeSet<_> = TOOLS.iter().map(|t| t.id).collect();
        for m in all_modes() {
            for s in Specialty::ALL {
                for aids in [true, false] {
                    let l = Layout::build(m, &[s], aids);
                    let got: BTreeSet<_> = l.reachable().into_iter().collect();
                    assert_eq!(got, reference, "outils perdus en mode {m:?} / {s:?}");
                    assert_eq!(l.tools.len(), TOOLS.len(), "doublons ou pertes");
                }
            }
            let l = Layout::build(m, &[], true);
            assert_eq!(l.reachable().len(), TOOLS.len());
        }
    }

    #[test]
    fn same_engine_tools_usable_in_every_mode() {
        let a: BTreeSet<_> = Layout::build(Mode::Beginner, &[], true).usable().into_iter().collect();
        let b: BTreeSet<_> = Layout::build(Mode::Expert, &[], true).usable().into_iter().collect();
        assert_eq!(a, b);
        for op in norelias_core::Op::NAMES { assert!(a.contains(op)); }
    }

    #[test]
    fn complexity_grows_with_level() {
        let n = |m| Layout::build(m, &[], true).prominent_usable().len();
        assert!(n(Mode::Beginner) < n(Mode::Intermediate) && n(Mode::Intermediate) <= n(Mode::Expert));
        let b = Layout::build(Mode::Beginner, &[], true);
        assert_eq!(b.visibility_of("denoise"), Some(Visibility::Collapsed));
        assert_eq!(b.visibility_of("tone_curve"), Some(Visibility::OnDemand), "outil expert : accessible, jamais supprimé");
        assert_eq!(b.settings_detail, SettingsDetail::Essential);
        assert_eq!(Layout::build(Mode::Expert, &[], true).settings_detail, SettingsDetail::Full);
    }

    #[test]
    fn specialty_promotes_tools() {
        let plain = Layout::build(Mode::Beginner, &[], true);
        let wild = Layout::build(Mode::Beginner, &[Specialty::Wildlife], true);
        assert_eq!(plain.visibility_of("sharpen"), Some(Visibility::Collapsed));
        assert_eq!(wild.visibility_of("sharpen"), Some(Visibility::Prominent));
        assert_eq!(wild.specialty, Some(Specialty::Wildlife));
    }

    #[test]
    fn pins_only_reference_existing_tools() {
        for s in Specialty::ALL { for id in pins(s) { assert!(crate::tools::get(id).is_some(), "{s:?} épingle un outil inconnu : {id}"); } }
    }

    #[test]
    fn disabling_aids_removes_guidance_but_not_tools() {
        let l = Layout::build(Mode::Beginner, &[], false);
        assert_eq!(l.explanations, Explanations::None);
        assert!(!l.tutorial_prompts && !l.shortcut_hints);
        assert_eq!(l.recommendations, Recommendations::Badge);
        assert_eq!(l.reachable().len(), TOOLS.len());
    }

    #[test]
    fn every_specialty_has_a_named_workspace_in_two_languages() {
        let ids: BTreeSet<_> = Specialty::ALL.iter().map(|s| workspace_for(*s).id).collect();
        assert_eq!(ids.len(), Specialty::ALL.len());
        for s in Specialty::ALL { let w = workspace_for(s); assert!(!w.fr.is_empty() && !w.en.is_empty()); }
    }
}
