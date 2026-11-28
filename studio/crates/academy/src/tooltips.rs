//! Explications d'outils au survol. Un outil non intégré affiche honnêtement « prévu ».

use norelias_core::Lang;
use norelias_onboarding::{profile::Profile, tools};

pub fn text(tool: &str, lang: Lang) -> Option<&'static str> {
    let (fr, en) = match tool {
        "exposure" => ("Exposition : éclaircit ou assombrit toute l'image, en IL (chaque IL double ou divise la lumière).", "Exposure: brightens or darkens the whole image, in EV (each EV doubles or halves the light)."),
        "contrast" => ("Contraste : écarte les clairs et les sombres autour du gris moyen.", "Contrast: spreads lights and darks around mid-grey."),
        "white_balance" => ("Balance des blancs : multiplie les canaux rouge, vert et bleu pour neutraliser une dominante.", "White balance: scales the red, green and blue channels to neutralise a cast."),
        "saturation" => ("Saturation : intensifie ou atténue toutes les couleurs ; 0 donne une image neutre.", "Saturation: intensifies or mutes all colours; 0 gives a neutral image."),
        "highlights" => ("Hautes lumières : comprime les clairs pour retrouver du détail et adoucir l'écrêtage.", "Highlights: compresses bright areas to regain detail and soften clipping."),
        "shadows" => ("Ombres : éclaircit les zones sombres sans toucher aux noirs profonds. Peut faire ressortir le bruit.", "Shadows: lifts dark areas without touching deep blacks. May bring out noise."),
        "denoise" => ("Réduction du bruit : lisse les zones plates en préservant les contours. Trop fort, il adoucit les textures.", "Noise reduction: smooths flat areas while preserving edges. Too strong, it softens texture."),
        "sharpen" => ("Netteté : renforce le contraste des contours. Ne remplace pas une bonne mise au point.", "Sharpening: boosts edge contrast. It does not replace good focus."),
        "rotate" => ("Rotation : redresse l'image et recadre automatiquement pour éviter les bords vides.", "Rotate: straightens the image and crops automatically to avoid empty borders."),
        "crop" => ("Recadrage : conserve une partie de l'image. L'original n'est jamais modifié.", "Crop: keeps part of the image. The original is never modified."),
        "advisor" => ("Conseiller : analyse l'image localement et propose des corrections prévisualisables. Vous décidez de tout.", "Advisor: analyses the image locally and suggests previewable fixes. You decide everything."),
        "academy" => ("Académie : leçons, exercices et recherche dans les guides. Toutes les aides peuvent être désactivées.", "Academy: lessons, exercises and guide search. All aids can be switched off."),
        _ => return None,
    };
    Some(if lang == Lang::Fr { fr } else { en })
}

pub fn planned_notice(lang: Lang) -> &'static str {
    if lang == Lang::Fr { "Prévu — cet outil n'est pas encore disponible." } else { "Planned — this tool is not available yet." }
}

/// Texte d'infobulle respectant les préférences : `None` si les infobulles sont désactivées.
pub fn tooltip(profile: &Profile, tool: &str) -> Option<&'static str> {
    if !profile.tooltips { return None; }
    let def = tools::get(tool)?;
    let lang = profile.lang();
    match def.status {
        tools::Status::Available => text(tool, lang),
        tools::Status::Planned => Some(planned_notice(lang)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use norelias_onboarding::profile::Language;

    #[test]
    fn every_available_tool_is_documented_in_both_languages() {
        for t in tools::available() {
            for l in [Lang::Fr, Lang::En] { assert!(text(t.id, l).map_or(0, str::len) > 30, "{} sans infobulle {l:?}", t.id); }
        }
    }
    #[test]
    fn planned_tools_never_get_a_functional_description() {
        for t in tools::TOOLS.iter().filter(|t| t.status == tools::Status::Planned) {
            assert!(text(t.id, Lang::Fr).is_none(), "{} est prévu mais décrit comme fonctionnel", t.id);
            let mut p = Profile::default();
            p.language = Language::En;
            assert!(tooltip(&p, t.id).unwrap().starts_with("Planned"));
        }
    }
    #[test]
    fn tooltips_can_be_disabled() {
        let mut p = Profile::default();
        p.language = Language::Fr;
        assert!(tooltip(&p, "exposure").unwrap().contains("Exposition"));
        p.tooltips = false;
        assert!(tooltip(&p, "exposure").is_none());
    }
}
