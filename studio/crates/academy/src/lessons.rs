use norelias_core::Lang;
use norelias_onboarding::profile::{Mode, Profile, Specialty};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Track { Photography, Retouch, Workflow, Specialty(Specialty) }

pub struct Lesson {
    pub id: &'static str,
    pub track: Track,
    pub level: Mode,
    /// Outils du registre mis en pratique.
    pub tools: &'static [&'static str],
    pub exercise: Option<&'static str>,
    pub title: (&'static str, &'static str),
    pub body: (&'static str, &'static str),
    pub keywords: (&'static str, &'static str),
}
impl Lesson {
    pub fn title(&self, l: Lang) -> &'static str { if l == Lang::Fr { self.title.0 } else { self.title.1 } }
    pub fn body(&self, l: Lang) -> &'static str { if l == Lang::Fr { self.body.0 } else { self.body.1 } }
    pub fn keywords(&self, l: Lang) -> &'static str { if l == Lang::Fr { self.keywords.0 } else { self.keywords.1 } }
}

use Mode::{Beginner as B, Intermediate as I};
use Track::*;

pub static LESSONS: &[Lesson] = &[
    Lesson { id: "exposure", track: Photography, level: B, tools: &["exposure", "advisor"], exercise: Some("ex_exposure"),
        title: ("Comprendre l'exposition", "Understanding exposure"),
        body: ("L'exposition règle la quantité de lumière captée. Un IL (stop) double ou divise par deux cette quantité. Une image sous-exposée est sombre et bruyante dans les ombres ; une image surexposée perd le détail des hautes lumières. Le conseiller compare la luminosité médiane au gris moyen (18 %), mais une scène volontairement sombre reste légitime : vous décidez.",
               "Exposure sets how much light is captured. One stop (EV) doubles or halves that amount. An underexposed image is dark and noisy in the shadows; an overexposed one loses highlight detail. The advisor compares median brightness to mid-grey (18%), but an intentionally dark scene is legitimate: you decide."),
        keywords: ("luminosité sombre clair IL stop gris moyen", "brightness dark bright EV stop mid-grey") },
    Lesson { id: "highlights", track: Retouch, level: B, tools: &["highlights", "exposure"], exercise: Some("ex_highlights"),
        title: ("Hautes lumières écrêtées", "Clipped highlights"),
        body: ("Quand un canal atteint son maximum, l'information est perdue : c'est l'écrêtage. En RAW, la marge de la capture permet souvent de récupérer du détail ; en JPEG déjà écrêté, on ne peut qu'adoucir la transition. Baissez l'exposition puis comprimez les hautes lumières.",
               "When a channel reaches its maximum the information is lost: that is clipping. In RAW the capture headroom often lets you recover detail; in an already clipped JPEG you can only soften the transition. Lower the exposure, then compress the highlights."),
        keywords: ("écrêtage surexposé cramé ciel blanc histogramme", "clipping blown highlights white sky histogram") },
    Lesson { id: "white_balance", track: Retouch, level: B, tools: &["white_balance", "advisor"], exercise: Some("ex_white_balance"),
        title: ("Balance des blancs", "White balance"),
        body: ("La balance des blancs rend neutres les surfaces qui doivent l'être. Une dominante orange indique une lumière chaude (tungstène, coucher de soleil), une dominante bleue une lumière froide (ombre, ciel couvert). Corrigez en visant un gris, un blanc ou une carte de référence.",
               "White balance makes surfaces that should be neutral look neutral. An orange cast means warm light (tungsten, sunset), a blue cast cold light (shade, overcast). Correct by aiming at a grey, a white or a reference card."),
        keywords: ("température couleur dominante orange bleu neutre gris", "temperature colour cast orange blue neutral grey") },
    Lesson { id: "tint", track: Retouch, level: I, tools: &["white_balance"], exercise: Some("ex_tint"),
        title: ("Teinte vert–magenta", "Green–magenta tint"),
        body: ("La température se règle sur l'axe orange–bleu ; la teinte sur l'axe vert–magenta. Les éclairages fluorescents et LED tirent souvent vers le vert. Une peau verdâtre est un bon indice d'un problème de teinte.",
               "Temperature works on the orange–blue axis; tint on the green–magenta axis. Fluorescent and LED lighting often lean green. Greenish skin is a good clue to a tint problem."),
        keywords: ("teinte vert magenta fluorescent led peau", "tint green magenta fluorescent led skin") },
    Lesson { id: "noise", track: Photography, level: I, tools: &["denoise"], exercise: Some("ex_noise"),
        title: ("Bruit et ISO", "Noise and ISO"),
        body: ("Le bruit vient d'un signal faible amplifié : sensibilité ISO élevée, ombres relevées. Il se voit surtout dans les zones plates. Une réduction trop forte efface les textures et donne un aspect plastique : augmentez l'intensité progressivement et comparez à 100 %.",
               "Noise comes from a weak signal being amplified: high ISO, lifted shadows. It shows mostly in flat areas. Too much reduction erases texture and looks plastic: increase strength gradually and compare at 100%."),
        keywords: ("grain iso luminance réduction du bruit ombres", "grain iso luminance noise reduction shadows") },
    Lesson { id: "sharpness", track: Retouch, level: B, tools: &["sharpen"], exercise: None,
        title: ("Netteté et mise au point", "Sharpness and focus"),
        body: ("Le renforcement augmente le contraste local des contours ; il améliore la perception du détail, mais ne rétablit pas une mise au point manquée. Sur une image bruyante, il amplifie le bruit : réduisez-le d'abord.",
               "Sharpening raises local edge contrast; it improves perceived detail but cannot restore missed focus. On a noisy image it amplifies the noise: reduce it first."),
        keywords: ("flou mise au point contours masque flou", "blur focus edges unsharp mask") },
    Lesson { id: "motion_blur", track: Photography, level: B, tools: &["advisor"], exercise: None,
        title: ("Flou de mouvement", "Motion blur"),
        body: ("Le bougé étire les contours dans une seule direction. Aucun réglage honnête ne le corrige après coup : agissez à la prise de vue. Règle pratique : vitesse ≥ 1/focale (en équivalent 24×36), stabilisation, trépied, ou ISO plus élevé pour accélérer.",
               "Motion blur smears edges in one direction. No honest edit fixes it afterwards: act at capture time. Rule of thumb: shutter speed ≥ 1/focal length (35 mm equivalent), stabilisation, tripod, or a higher ISO to go faster."),
        keywords: ("bougé vitesse d'obturation trépied stabilisation", "camera shake shutter speed tripod stabilisation") },
    Lesson { id: "horizon", track: Retouch, level: B, tools: &["rotate"], exercise: Some("ex_horizon"),
        title: ("Redresser l'horizon", "Straightening the horizon"),
        body: ("Un horizon incliné distrait, même de 1°. La rotation recadre légèrement l'image pour éviter les bords vides : visez des horizons droits sur l'eau, des verticales droites en architecture.",
               "A tilted horizon distracts, even at 1°. Rotation crops the image slightly to avoid empty borders: aim for straight horizons over water, straight verticals in architecture."),
        keywords: ("horizon incliné rotation redressement niveau", "tilted horizon rotation straighten level") },
    Lesson { id: "thirds", track: Photography, level: B, tools: &["crop"], exercise: Some("ex_thirds"),
        title: ("Composition : la règle des tiers", "Composition: the rule of thirds"),
        body: ("Divisez l'image en tiers horizontaux et verticaux ; placer le sujet sur une intersection donne souvent plus de dynamisme qu'au centre. Ce n'est pas une loi : un sujet symétrique peut très bien rester centré.",
               "Divide the frame into horizontal and vertical thirds; placing the subject on an intersection often feels more dynamic than the centre. It is not a law: a symmetrical subject may well stay centred."),
        keywords: ("recadrage tiers sujet centré cadrage", "crop thirds centred subject framing") },
    Lesson { id: "contrast_saturation", track: Retouch, level: B, tools: &["contrast", "saturation"], exercise: None,
        title: ("Contraste et saturation", "Contrast and saturation"),
        body: ("Le contraste écarte les clairs et les sombres autour du gris moyen ; la saturation intensifie les couleurs. Avec parcimonie : des couleurs trop saturées perdent leur nuance, particulièrement sur les peaux.",
               "Contrast spreads lights and darks around mid-grey; saturation intensifies colours. Use sparingly: over-saturated colours lose nuance, especially on skin."),
        keywords: ("contraste saturation couleurs éclatantes", "contrast saturation vivid colours") },
    Lesson { id: "shadows_highlights", track: Retouch, level: I, tools: &["shadows", "highlights"], exercise: None,
        title: ("Équilibrer ombres et hautes lumières", "Balancing shadows and highlights"),
        body: ("Pour une scène à fort contraste (contre-jour), relevez les ombres et comprimez les hautes lumières plutôt que de modifier l'exposition globale. Relever beaucoup les ombres fait ressortir le bruit : surveillez-le.",
               "For a high-contrast scene (backlight), lift shadows and compress highlights rather than changing global exposure. Lifting shadows a lot brings out noise: watch it."),
        keywords: ("contre-jour ombres bouchées plage dynamique", "backlight blocked shadows dynamic range") },
    Lesson { id: "non_destructive", track: Workflow, level: B, tools: &[], exercise: None,
        title: ("Retouche non destructive", "Non-destructive editing"),
        body: ("Norélias ne modifie jamais votre fichier original : les réglages sont des opérations enregistrées à part et appliquées à la volée. Vous pouvez les modifier, les retirer ou les copier à tout moment.",
               "Norélias never changes your original file: adjustments are recorded operations applied on the fly. You can change, remove or copy them at any time."),
        keywords: ("original sauvegarde réversible historique", "original backup reversible history") },
    Lesson { id: "privacy_ai", track: Workflow, level: B, tools: &["advisor"], exercise: None,
        title: ("Confidentialité et IA", "Privacy and AI"),
        body: ("Le conseiller photographique fonctionne entièrement sur votre ordinateur. Aucune photographie n'est envoyée à un service externe sans votre consentement explicite, et vous êtes prévenu avant tout service payant. Les consentements se gèrent et se retirent dans les préférences.",
               "The photo advisor runs entirely on your computer. No photograph is sent to an external service without your explicit consent, and you are warned before any paid service. Consents are managed and withdrawn in the preferences."),
        keywords: ("confidentialité consentement cloud télémétrie données", "privacy consent cloud telemetry data") },
    Lesson { id: "migrate", track: Workflow, level: B, tools: &[], exercise: None,
        title: ("Venir d'un autre logiciel", "Coming from another application"),
        body: ("Choisissez une disposition de raccourcis familière dans les préférences. Les presets XMP se convertissent en réglages réels pour les paramètres équivalents ; les autres sont listés dans un rapport avec la raison. Les rendus ne sont pas identiques pixel à pixel : vérifiez toujours le résultat.",
               "Pick a familiar shortcut layout in the preferences. XMP presets convert to real adjustments for equivalent parameters; the others are listed in a report with the reason. Renderings are not pixel-identical: always check the result."),
        keywords: ("photoshop lightroom capture one migration preset xmp raccourcis", "photoshop lightroom capture one migration preset xmp shortcuts") },
    Lesson { id: "portrait_light", track: Specialty(Specialty::Portrait), level: B, tools: &["white_balance", "exposure", "advisor"], exercise: None,
        title: ("Portrait : lumière et teint", "Portrait: light and skin tone"),
        body: ("En portrait, la peau est le meilleur repère : une peau verdâtre ou orangée trahit une balance des blancs ou une teinte à corriger. Exposez pour la peau avant le reste. Les outils de lissage de peau et de Dodge & Burn sont prévus mais pas encore intégrés.",
               "In portraits skin is your best reference: greenish or orange skin reveals a white balance or tint problem. Expose for the skin first. Skin smoothing and Dodge & Burn tools are planned but not integrated yet."),
        keywords: ("portrait peau teint visage lumière", "portrait skin tone face light") },
    Lesson { id: "wedding_batch", track: Specialty(Specialty::Wedding), level: I, tools: &["white_balance", "highlights", "shadows"], exercise: None,
        title: ("Mariage : robe blanche et lumière mixte", "Wedding: white dress and mixed light"),
        body: ("La robe blanche expose le risque d'écrêtage : surveillez les hautes lumières. La lumière mixte (église, flash) demande une balance des blancs par séquence. Le traitement par lots est prévu mais pas encore intégré.",
               "A white dress invites clipping: watch the highlights. Mixed light (church, flash) calls for white balance per sequence. Batch processing is planned but not integrated yet."),
        keywords: ("mariage robe blanche lumière mixte séquence", "wedding white dress mixed light sequence") },
    Lesson { id: "landscape_horizon", track: Specialty(Specialty::Landscape), level: B, tools: &["rotate", "highlights", "shadows"], exercise: None,
        title: ("Paysage : horizon et ciel", "Landscape: horizon and sky"),
        body: ("Redressez l'horizon d'abord, puis comprimez les hautes lumières du ciel et relevez les ombres du premier plan. Pour les scènes très contrastées, photographiez en RAW.",
               "Straighten the horizon first, then compress sky highlights and lift foreground shadows. For very contrasty scenes, shoot RAW."),
        keywords: ("paysage ciel premier plan horizon", "landscape sky foreground horizon") },
    Lesson { id: "wildlife_iso", track: Specialty(Specialty::Wildlife), level: I, tools: &["denoise", "sharpen", "crop"], exercise: None,
        title: ("Animalier : ISO élevé et recadrage", "Wildlife: high ISO and cropping"),
        body: ("La faune impose des vitesses rapides donc des ISO élevés. Réduisez le bruit avant de renforcer, puis recadrez : un recadrage serré révèle le bruit et le manque de netteté.",
               "Wildlife demands fast shutter speeds hence high ISO. Reduce noise before sharpening, then crop: a tight crop reveals noise and softness."),
        keywords: ("animalier oiseaux iso élevé recadrage bruit", "wildlife birds high iso crop noise") },
    Lesson { id: "product_wb", track: Specialty(Specialty::Product), level: B, tools: &["white_balance", "rotate", "crop"], exercise: None,
        title: ("Produit : couleurs fidèles et lignes droites", "Product: faithful colour and straight lines"),
        body: ("Pour un produit, la couleur doit correspondre à la réalité : photographiez une charte grise, réglez la balance des blancs dessus, et gardez les lignes droites.",
               "For a product, colour must match reality: shoot a grey card, set white balance on it, and keep lines straight."),
        keywords: ("produit couleur fidèle charte grise e-commerce", "product accurate colour grey card e-commerce") },
    Lesson { id: "school_workflow", track: Specialty(Specialty::School), level: B, tools: &["white_balance", "crop"], exercise: None,
        title: ("Scolaire : cohérence d'une série", "School: consistency across a series"),
        body: ("En photographie scolaire, la cohérence prime : même cadrage, même balance des blancs sur toute la série. Les forfaits, l'impression et la livraison en galerie sont prévus mais pas encore intégrés.",
               "In school photography consistency matters most: same framing, same white balance across the series. Packages, printing and gallery delivery are planned but not integrated yet."),
        keywords: ("scolaire série cohérence forfait", "school series consistency package") },
];

pub fn get(id: &str) -> Option<&'static Lesson> { LESSONS.iter().find(|l| l.id == id) }

/// Parcours progressif : leçons générales jusqu'au niveau demandé, puis celles des spécialités choisies.
/// Ordonné du plus simple au plus avancé. Toutes les leçons restent consultables via la recherche.
pub fn path_for(profile: &Profile) -> Vec<&'static Lesson> {
    let mode = profile.mode();
    let mut v: Vec<&Lesson> = LESSONS.iter().filter(|l| match l.track {
        Track::Specialty(s) => profile.specialties.contains(&s),
        _ => l.level <= mode,
    }).collect();
    v.sort_by_key(|l| (matches!(l.track, Track::Specialty(_)), l.level, l.id));
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ids_unique_and_tools_exist() {
        let mut ids: Vec<_> = LESSONS.iter().map(|l| l.id).collect();
        ids.sort();
        let n = ids.len();
        ids.dedup();
        assert_eq!(n, ids.len());
        for l in LESSONS { for t in l.tools { assert!(norelias_onboarding::tools::get(t).is_some(), "{} référence un outil inconnu {t}", l.id); } }
    }
    #[test]
    fn all_text_present_in_both_languages() {
        for l in LESSONS {
            for lang in [Lang::Fr, Lang::En] {
                assert!(l.title(lang).len() > 5 && l.body(lang).len() > 80 && l.keywords(lang).len() > 8, "{} incomplet ({lang:?})", l.id);
            }
            assert_ne!(l.body.0, l.body.1);
        }
    }
    #[test]
    fn lessons_never_present_planned_tools_as_available() {
        for l in LESSONS { for lang in [Lang::Fr, Lang::En] {
            let b = l.body(lang).to_lowercase();
            if b.contains("planned") || b.contains("prévu") { assert!(b.contains("not integrated") || b.contains("pas encore intégré"), "{}", l.id); }
        } }
    }
    #[test]
    fn path_is_progressive_and_specialised() {
        let mut p = Profile::default();
        p.photo_level = norelias_onboarding::Skill::Beginner;
        p.retouch_level = norelias_onboarding::Skill::Beginner;
        p.specialties = vec![Specialty::Wildlife];
        let path = path_for(&p);
        assert!(path.iter().all(|l| l.level == Mode::Beginner || matches!(l.track, Track::Specialty(_))));
        assert_eq!(path.last().unwrap().id, "wildlife_iso");
        assert!(path.iter().all(|l| l.id != "portrait_light"));
    }
}
