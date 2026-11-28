use crate::{Finding, Kind};
use norelias_core::Lang;

pub fn title(f: &Finding, lang: Lang) -> String {
    let s = match (f.kind, lang) {
        (Kind::Exposure, Lang::Fr) => if f.value > 0.0 { "Image sous-exposée" } else { "Image surexposée" },
        (Kind::Exposure, Lang::En) => if f.value > 0.0 { "Underexposed image" } else { "Overexposed image" },
        (Kind::WhiteBalance, Lang::Fr) => "Balance des blancs à corriger",
        (Kind::WhiteBalance, Lang::En) => "White balance needs correction",
        (Kind::ColorCast, Lang::Fr) => "Dominante colorée (vert/magenta)",
        (Kind::ColorCast, Lang::En) => "Colour cast (green/magenta)",
        (Kind::HighlightClipping, Lang::Fr) => "Hautes lumières écrêtées",
        (Kind::HighlightClipping, Lang::En) => "Clipped highlights",
        (Kind::Noise, Lang::Fr) => "Bruit numérique visible",
        (Kind::Noise, Lang::En) => "Visible digital noise",
        (Kind::Sharpness, Lang::Fr) => "Image peu nette",
        (Kind::Sharpness, Lang::En) => "Soft image",
        (Kind::MotionBlur, Lang::Fr) => "Flou de mouvement probable",
        (Kind::MotionBlur, Lang::En) => "Probable motion blur",
        (Kind::HorizonTilt, Lang::Fr) => "Horizon incliné",
        (Kind::HorizonTilt, Lang::En) => "Tilted horizon",
        (Kind::Thirds, Lang::Fr) => "Sujet centré : essayez la règle des tiers",
        (Kind::Thirds, Lang::En) => "Centred subject: try the rule of thirds",
    };
    s.to_string()
}

pub fn explain(f: &Finding, lang: Lang) -> String {
    let v = f.value;
    match (f.kind, lang) {
        (Kind::Exposure, Lang::Fr) => format!(
            "La luminosité médiane est éloignée du gris moyen de {v:+.1} IL. Correction proposée : exposition {:+.1} IL. \
             Si l'image est volontairement sombre ou claire (basse/haute clé), ignorez cette suggestion.",
            ev_of(f)),
        (Kind::Exposure, Lang::En) => format!(
            "Median brightness is {v:+.1} EV away from mid-grey. Suggested fix: exposure {:+.1} EV. \
             If the image is intentionally dark or bright (low/high key), ignore this suggestion.", ev_of(f)),
        (Kind::WhiteBalance, Lang::Fr) => format!(
            "Les zones quasi neutres penchent de {v:+.2} IL entre rouge et bleu (positif = trop chaud). \
             La correction rééquilibre les canaux ; une lumière chaude voulue (coucher de soleil) peut être légitime."),
        (Kind::WhiteBalance, Lang::En) => format!(
            "Near-neutral areas lean {v:+.2} stops between red and blue (positive = too warm). \
             The fix rebalances the channels; intentionally warm light (sunset) can be legitimate."),
        (Kind::ColorCast, Lang::Fr) => format!(
            "Les zones neutres ont une dominante vert/magenta de {v:+.2} IL (positif = vert). \
             Éclairage fluorescent ou LED fréquent ; la correction compense le canal vert."),
        (Kind::ColorCast, Lang::En) => format!(
            "Neutral areas show a green/magenta cast of {v:+.2} stops (positive = green). \
             Common under fluorescent or LED light; the fix compensates the green channel."),
        (Kind::HighlightClipping, Lang::Fr) => format!(
            "{:.1} % des pixels ont au moins un canal saturé. Sur un RAW, une partie du détail peut être récupérée ; \
             sur un JPEG déjà écrêté, la correction adoucit seulement la transition, le détail perdu ne revient pas.", v * 100.0),
        (Kind::HighlightClipping, Lang::En) => format!(
            "{:.1}% of pixels have at least one saturated channel. In a RAW file some detail can be recovered; \
             in an already clipped JPEG the fix only softens the transition, lost detail does not come back.", v * 100.0),
        (Kind::Noise, Lang::Fr) => format!(
            "Bruit estimé σ = {v:.3} (échelle 0–1) dans les zones plates. La réduction préserve les contours mais peut adoucir les textures fines."),
        (Kind::Noise, Lang::En) => format!(
            "Estimated noise σ = {v:.3} (0–1 scale) in flat areas. Noise reduction preserves edges but may soften fine texture."),
        (Kind::Sharpness, Lang::Fr) => format!(
            "Les contours sont étalés (indice de netteté {v:.2} ; une image nette dépasse environ 1,3). Un renforcement améliore la perception, mais ne rétablit pas une mise au point manquée. \
             Une scène sans contours (ciel, fond uni) peut être signalée à tort."),
        (Kind::Sharpness, Lang::En) => format!(
            "Edges are spread out (sharpness index {v:.2}; a sharp image is above roughly 1.3). Sharpening improves perceived detail but cannot restore missed focus. \
             A scene without edges (sky, plain backdrop) may be flagged wrongly."),
        (Kind::MotionBlur, Lang::Fr) =>
            "Les contours sont nettement moins marqués dans une direction que dans l'autre, signe de bougé. Aucune correction automatique fiable n'est disponible : \
             recommandation de prise de vue — vitesse plus rapide, stabilisation, trépied.".to_string(),
        (Kind::MotionBlur, Lang::En) =>
            "Edges are much weaker in one direction than the other, a sign of motion blur. No reliable automatic fix is available: \
             shooting advice — faster shutter, stabilisation, tripod.".to_string(),
        (Kind::HorizonTilt, Lang::Fr) => format!(
            "Les lignes horizontales dominantes sont inclinées de {v:+.1}°. La correction pivote l'image et recadre légèrement pour éviter les bords vides."),
        (Kind::HorizonTilt, Lang::En) => format!(
            "Dominant horizontal lines are tilted by {v:+.1}°. The fix rotates the image and crops slightly to avoid empty borders."),
        (Kind::Thirds, Lang::Fr) =>
            "L'essentiel du détail est groupé au centre. Un recadrage plaçant le sujet sur un point fort des tiers donne souvent plus de dynamisme. C'est un choix créatif : jamais appliqué automatiquement.".to_string(),
        (Kind::Thirds, Lang::En) =>
            "Most of the detail is grouped at the centre. A crop placing the subject on a thirds intersection often feels more dynamic. This is a creative choice: never applied automatically.".to_string(),
    }
}

fn ev_of(f: &Finding) -> f32 {
    match f.ops.first() { Some(norelias_core::Op::Exposure { ev }) => *ev, _ => 0.0 }
}
