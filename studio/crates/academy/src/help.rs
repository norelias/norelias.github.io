//! Registre centralisé de l'aide contextuelle. Chaque outil **disponible** doit avoir une fiche complète et chaque
//! curseur du moteur doit être expliqué : des tests échouent sinon. Aucune fiche pour un outil non intégré.
//! Aucune vidéo n'existe à ce jour : `video` est `None` partout, et un test exige que tout chemin déclaré existe.

use norelias_core::{ops::apply_stack, synth, Image, Lang, Op};
use norelias_onboarding::privacy::{ConsentError, ConsentLedger, Purpose};
use norelias_onboarding::profile::{Mode, Profile};
use norelias_onboarding::shortcuts::Command;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Copy)]
pub struct Tx(pub &'static str, pub &'static str);
impl Tx { pub fn get(&self, l: Lang) -> &'static str { if l == Lang::Fr { self.0 } else { self.1 } } }

pub struct Param { pub id: &'static str, pub name: Tx, pub effect: Tx }

pub struct Entry {
    pub tool: &'static str,
    pub name: Tx,
    pub definition: Tx,
    pub utility: Tx,
    pub situations: Tx,
    pub how: Tx,
    pub params: &'static [Param],
    pub mistakes: Tx,
    pub tips: Tx,
    /// Exemple avant/après généré depuis une même photographie (voir `example`).
    pub has_example: bool,
    pub exercise: Option<&'static str>,
    pub shortcut: Option<Command>,
    /// Chemin d'une vidéo réellement présente dans le dépôt ; `None` tant qu'aucune n'existe.
    pub video: Option<&'static str>,
}

const fn p(id: &'static str, fr_n: &'static str, en_n: &'static str, fr: &'static str, en: &'static str) -> Param { Param { id, name: Tx(fr_n, en_n), effect: Tx(fr, en) } }

pub static ENTRIES: &[Entry] = &[
    Entry { tool: "exposure", name: Tx("Exposition", "Exposure"),
        definition: Tx("Éclaircit ou assombrit toute l'image d'un seul geste.", "Brightens or darkens the whole image in one move."),
        utility: Tx("Corrige une photo prise trop sombre ou trop claire sans refaire la prise de vue.", "Fixes a photo shot too dark or too bright without reshooting."),
        situations: Tx("Photo sous-exposée en intérieur, scène claire (neige, plage), balance générale avant les autres réglages.", "Underexposed indoor shots, bright scenes (snow, beach), overall balance before other adjustments."),
        how: Tx("Déplacez le curseur vers la droite si la photo est trop sombre, vers la gauche si elle est trop claire. Réglez-le en premier.", "Move the slider right if the photo is too dark, left if too bright. Set it first."),
        params: &[p("ev", "Exposition (IL)", "Exposure (EV)", "Chaque +1 IL double la lumière ; −1 IL la divise par deux.", "Each +1 EV doubles the light; −1 EV halves it.")],
        mistakes: Tx("Éclaircir fortement une image déjà écrêtée : le détail perdu ne revient pas, et les ombres relevées montrent du bruit.", "Heavily brightening an already clipped image: lost detail does not return, and lifted shadows show noise."),
        tips: Tx("Surveillez les hautes lumières : mieux vaut un peu sombre que brûlé, car on récupère plus facilement les ombres.", "Watch the highlights: slightly dark beats blown out, since shadows are easier to recover."),
        has_example: true, exercise: Some("ex_exposure"), shortcut: None, video: None },
    Entry { tool: "contrast", name: Tx("Contraste", "Contrast"),
        definition: Tx("Écarte les zones claires et sombres autour du gris moyen.", "Pushes light and dark areas apart around mid-grey."),
        utility: Tx("Donne de la profondeur à une image terne ou adoucit une image trop dure.", "Adds depth to a flat image or softens a harsh one."),
        situations: Tx("Journée brumeuse, éclairage plat, ou au contraire lumière dure à adoucir.", "Hazy day, flat lighting, or on the contrary harsh light to soften."),
        how: Tx("Augmentez par petits pas ; comparez avec l'original. Le gris moyen reste inchangé.", "Increase in small steps; compare with the original. Mid-grey stays unchanged."),
        params: &[p("amount", "Quantité", "Amount", "Positif : plus de contraste. Négatif : moins. Zéro : aucun changement.", "Positive: more contrast. Negative: less. Zero: no change.")],
        mistakes: Tx("Trop de contraste écrase les ombres et brûle les hautes lumières.", "Too much contrast crushes shadows and blows highlights."),
        tips: Tx("Ajoutez du contraste après avoir réglé l'exposition, jamais avant.", "Add contrast after setting exposure, never before."),
        has_example: true, exercise: None, shortcut: None, video: None },
    Entry { tool: "white_balance", name: Tx("Balance des blancs", "White balance"),
        definition: Tx("Neutralise la dominante de couleur de l'éclairage.", "Neutralises the colour cast of the lighting."),
        utility: Tx("Rend le blanc blanc et la peau naturelle sous une lumière chaude, froide ou verdâtre.", "Makes white look white and skin natural under warm, cold or greenish light."),
        situations: Tx("Intérieur sous ampoules, ombre bleutée, éclairage fluorescent ou LED, mélange de lumières.", "Indoors under bulbs, bluish shade, fluorescent or LED light, mixed lighting."),
        how: Tx("Visez une surface censée être grise ou blanche, ou rééquilibrez rouge et bleu jusqu'à ce que les neutres le soient.", "Aim at a surface that should be grey or white, or rebalance red and blue until neutrals look neutral."),
        params: &[p("r", "Gain rouge", "Red gain", "Multiplie le rouge : plus chaud (orangé) au-dessus de 1, plus froid en dessous.", "Scales red: warmer (orange) above 1, cooler below."),
                  p("g", "Gain vert", "Green gain", "Multiplie le vert : corrige la dominante vert–magenta (teinte).", "Scales green: fixes the green–magenta cast (tint)."),
                  p("b", "Gain bleu", "Blue gain", "Multiplie le bleu : plus froid (bleuté) au-dessus de 1, plus chaud en dessous.", "Scales blue: cooler (bluish) above 1, warmer below.")],
        mistakes: Tx("Neutraliser une lumière chaude voulue (coucher de soleil) : l'ambiance disparaît.", "Neutralising intentionally warm light (sunset): the mood disappears."),
        tips: Tx("Réglez-la sur la peau en portrait ; photographiez une charte grise pour une fidélité exacte.", "Set it on skin in portraits; shoot a grey card for exact fidelity."),
        has_example: true, exercise: Some("ex_white_balance"), shortcut: None, video: None },
    Entry { tool: "highlights", name: Tx("Hautes lumières", "Highlights"),
        definition: Tx("Comprime les zones les plus claires pour retrouver du détail.", "Compresses the brightest areas to regain detail."),
        utility: Tx("Récupère un ciel ou une robe blanche trop lumineux.", "Recovers an over-bright sky or white dress."),
        situations: Tx("Ciel brûlé, nappe ou robe blanche, reflets, contre-jour.", "Blown sky, white tablecloth or dress, reflections, backlight."),
        how: Tx("Augmentez la quantité jusqu'à ce que le détail réapparaisse dans les zones claires.", "Raise the amount until detail reappears in the bright areas."),
        params: &[p("amount", "Quantité", "Amount", "Plus la valeur est haute, plus les clairs au-dessus du seuil sont comprimés.", "The higher the value, the more bright areas above the threshold are compressed.")],
        mistakes: Tx("Attendre un miracle sur un JPEG déjà écrêté : seul un RAW garde l'information.", "Expecting a miracle on an already clipped JPEG: only a RAW keeps the information."),
        tips: Tx("Combinez avec une légère baisse d'exposition pour un résultat plus naturel.", "Combine with a slight exposure decrease for a more natural result."),
        has_example: true, exercise: Some("ex_highlights"), shortcut: None, video: None },
    Entry { tool: "shadows", name: Tx("Ombres", "Shadows"),
        definition: Tx("Éclaircit les zones sombres sans toucher aux noirs profonds.", "Lifts dark areas without touching deep blacks."),
        utility: Tx("Révèle le détail d'un visage à contre-jour ou d'un premier plan sombre.", "Reveals detail in a backlit face or dark foreground."),
        situations: Tx("Contre-jour, sous-bois, intérieur contrasté.", "Backlight, forest floor, contrasty interior."),
        how: Tx("Relevez progressivement en gardant un œil sur le bruit.", "Raise gradually while watching the noise."),
        params: &[p("amount", "Quantité", "Amount", "Plus la valeur est haute, plus les ombres sont éclaircies.", "The higher the value, the more shadows are lifted.")],
        mistakes: Tx("Relever trop fort donne un rendu terne et fait ressortir le bruit.", "Lifting too much looks dull and brings out noise."),
        tips: Tx("Gardez de vrais noirs quelque part dans l'image pour préserver la profondeur.", "Keep true blacks somewhere in the image to preserve depth."),
        has_example: true, exercise: None, shortcut: None, video: None },
    Entry { tool: "saturation", name: Tx("Saturation", "Saturation"),
        definition: Tx("Intensifie ou atténue toutes les couleurs ensemble.", "Intensifies or mutes all colours together."),
        utility: Tx("Rend les couleurs plus vives, plus discrètes, ou passe en noir et blanc.", "Makes colours more vivid, more subtle, or goes to black and white."),
        situations: Tx("Image terne, couleurs trop criardes, conversion en noir et blanc.", "Dull image, garish colours, black-and-white conversion."),
        how: Tx("Modifiez par petits pas ; 0 donne une image sans couleur, 1 ne change rien.", "Change in small steps; 0 gives a colourless image, 1 changes nothing."),
        params: &[p("factor", "Facteur", "Factor", "1 = inchangé ; moins de 1 atténue ; plus de 1 intensifie ; 0 = noir et blanc.", "1 = unchanged; below 1 mutes; above 1 intensifies; 0 = black and white.")],
        mistakes: Tx("Sursaturer les peaux : elles deviennent orange ou rouges.", "Over-saturating skin: it turns orange or red."),
        tips: Tx("Si une seule couleur pose problème, un réglage ciblé (prévu) sera préférable.", "If only one colour is a problem, a targeted adjustment (planned) will be better."),
        has_example: true, exercise: None, shortcut: None, video: None },
    Entry { tool: "denoise", name: Tx("Réduction du bruit", "Noise reduction"),
        definition: Tx("Lisse le grain numérique des zones unies en préservant les contours.", "Smooths digital grain in flat areas while preserving edges."),
        utility: Tx("Nettoie les photos à ISO élevé ou aux ombres relevées.", "Cleans high-ISO photos or lifted shadows."),
        situations: Tx("Photo de soirée, sport en salle, animalier, ombres éclaircies.", "Evening shots, indoor sport, wildlife, brightened shadows."),
        how: Tx("Augmentez jusqu'à ce que le grain devienne discret, puis vérifiez à 100 %.", "Increase until the grain becomes discreet, then check at 100%."),
        params: &[p("strength", "Intensité", "Strength", "Plus l'intensité est haute, plus le lissage est fort ; au-delà, les textures fines s'effacent.", "The higher the strength, the stronger the smoothing; beyond that fine texture fades.")],
        mistakes: Tx("Réduire trop : les visages deviennent plastiques.", "Reducing too much: faces become plasticky."),
        tips: Tx("Réduisez le bruit avant d'ajouter de la netteté, jamais l'inverse.", "Reduce noise before adding sharpness, never the reverse."),
        has_example: true, exercise: Some("ex_noise"), shortcut: None, video: None },
    Entry { tool: "sharpen", name: Tx("Netteté", "Sharpening"),
        definition: Tx("Renforce le contraste des contours pour accentuer le détail perçu.", "Boosts edge contrast to enhance perceived detail."),
        utility: Tx("Redonne de la précision à une image légèrement douce.", "Restores precision to a slightly soft image."),
        situations: Tx("Image légèrement douce, finition avant l'exportation.", "Slightly soft image, finishing before export."),
        how: Tx("Augmentez doucement ; regardez les contours à 100 % et évitez les halos clairs.", "Increase gently; look at edges at 100% and avoid bright halos."),
        params: &[p("amount", "Quantité", "Amount", "Plus la valeur est haute, plus les contours sont renforcés ; trop fort crée des halos.", "The higher the value, the stronger the edges; too strong creates halos.")],
        mistakes: Tx("Espérer corriger une mise au point manquée ou un flou de bougé : c'est impossible.", "Expecting to fix missed focus or motion blur: it is impossible."),
        tips: Tx("Appliquez-la en dernier, adaptée à la taille de sortie.", "Apply it last, matched to the output size."),
        has_example: true, exercise: None, shortcut: None, video: None },
    Entry { tool: "rotate", name: Tx("Rotation et redressement", "Rotate and straighten"),
        definition: Tx("Fait pivoter l'image et la recadre pour éviter les bords vides.", "Rotates the image and crops it to avoid empty borders."),
        utility: Tx("Redresse un horizon ou des verticales inclinés.", "Straightens a tilted horizon or verticals."),
        situations: Tx("Paysage, bord de mer, architecture, photo prise à main levée.", "Landscape, seaside, architecture, handheld shots."),
        how: Tx("Alignez une ligne de référence (horizon) avec la grille.", "Align a reference line (horizon) with the grid."),
        params: &[p("degrees", "Angle (°)", "Angle (°)", "Positif : rotation horaire. L'image est légèrement agrandie pour supprimer les bords.", "Positive: clockwise rotation. The image is slightly enlarged to remove borders.")],
        mistakes: Tx("Redresser un horizon qui n'en est pas un (collines, vagues) : jugez à l'œil.", "Straightening something that is not a true horizon (hills, waves): judge by eye."),
        tips: Tx("Un angle de 1° se voit déjà sur l'eau.", "A 1° angle is already visible on water."),
        has_example: true, exercise: Some("ex_horizon"), shortcut: None, video: None },
    Entry { tool: "crop", name: Tx("Recadrage", "Crop"),
        definition: Tx("Conserve une partie de l'image et élimine le reste.", "Keeps part of the image and removes the rest."),
        utility: Tx("Améliore la composition et retire les éléments distrayants.", "Improves composition and removes distractions."),
        situations: Tx("Sujet trop centré, éléments gênants sur les bords, format imposé.", "Over-centred subject, distracting edge elements, required format."),
        how: Tx("Placez le sujet sur un point fort des tiers, puis validez. L'original n'est jamais modifié.", "Place the subject on a thirds intersection, then confirm. The original is never modified."),
        params: &[p("x", "Position X", "X position", "Bord gauche du cadre, de 0 (gauche) à 1 (droite).", "Left edge of the frame, from 0 (left) to 1 (right)."),
                  p("y", "Position Y", "Y position", "Bord haut du cadre, de 0 (haut) à 1 (bas).", "Top edge of the frame, from 0 (top) to 1 (bottom)."),
                  p("w", "Largeur", "Width", "Largeur conservée, en fraction de l'image.", "Width kept, as a fraction of the image."),
                  p("h", "Hauteur", "Height", "Hauteur conservée, en fraction de l'image.", "Height kept, as a fraction of the image.")],
        mistakes: Tx("Recadrer trop serré : on perd en résolution et le bruit devient visible.", "Cropping too tight: resolution drops and noise becomes visible."),
        tips: Tx("Recadrez à la fin de la retouche, en pensant au format de sortie.", "Crop at the end of editing, thinking of the output format."),
        has_example: true, exercise: Some("ex_thirds"), shortcut: Some(Command::Crop), video: None },
    Entry { tool: "advisor", name: Tx("Conseiller photographique", "Photo advisor"),
        definition: Tx("Analyse localement l'image et propose des corrections que vous pouvez prévisualiser.", "Analyses the image locally and suggests fixes you can preview."),
        utility: Tx("Repère exposition, couleur, bruit, netteté, horizon et composition, et explique pourquoi.", "Spots exposure, colour, noise, sharpness, horizon and composition, and explains why."),
        situations: Tx("Premier regard sur une photo, apprentissage, traitement rapide d'une série.", "First look at a photo, learning, quick processing of a series."),
        how: Tx("Ouvrez les constats, prévisualisez, puis appliquez un par un ou lancez les corrections sûres.", "Open the findings, preview, then apply one by one or run the safe corrections."),
        params: &[],
        mistakes: Tx("Appliquer sans réfléchir : une image volontairement sombre ou chaude peut être signalée.", "Applying blindly: an intentionally dark or warm image may be flagged."),
        tips: Tx("Vous gardez toujours le dernier mot ; tout est annulable.", "You always have the final say; everything is undoable."),
        has_example: false, exercise: None, shortcut: None, video: None },
    Entry { tool: "academy", name: Tx("Académie Norélias", "Norélias Academy"),
        definition: Tx("Leçons, exercices et recherche dans les guides, intégrés au logiciel.", "Lessons, exercises and guide search built into the software."),
        utility: Tx("Apprendre la photographie et la retouche à votre rythme, sur des photos libres de droits.", "Learn photography and editing at your own pace, on rights-free photos."),
        situations: Tx("Découverte d'un outil, progression par spécialité, révision.", "Discovering a tool, progress by specialty, revision."),
        how: Tx("Choisissez un parcours ou cherchez un mot-clé ; le suivi de progression est facultatif.", "Pick a path or search a keyword; progress tracking is optional."),
        params: &[],
        mistakes: Tx("Croire que tout est disponible : les leçons indiquent clairement les outils encore prévus.", "Assuming everything is available: lessons clearly say which tools are still planned."),
        tips: Tx("Toutes les aides se désactivent dans les préférences.", "All aids can be switched off in the preferences."),
        has_example: false, exercise: None, shortcut: Some(Command::OpenAcademy), video: None },
];

pub fn entry(tool: &str) -> Option<&'static Entry> { ENTRIES.iter().find(|e| e.tool == tool) }

// ───────────────────────── Exemple avant/après ─────────────────────────

/// Même photographie (scène procédurale) avant et après l'outil : retourne (avant, après, référence intacte).
pub fn example(tool: &str) -> Option<(Image, Image)> {
    let base = synth::scene(480, 320, 0.0);
    let (before, ops): (Image, Vec<Op>) = match tool {
        "exposure" => (synth::exposure(&base, -1.5), vec![Op::Exposure { ev: 1.4 }]),
        "contrast" => (apply_stack(&base, &[Op::Contrast { amount: -0.4 }]), vec![Op::Contrast { amount: 0.5 }]),
        "white_balance" => (synth::tint(&base, [1.3, 1.0, 0.75]), vec![Op::WhiteBalance { r: 0.77, g: 1.0, b: 1.33 }]),
        "highlights" => (synth::clip(&synth::exposure(&base, 2.0)), vec![Op::Exposure { ev: -1.0 }, Op::Highlights { amount: 1.0 }]),
        "shadows" => (synth::exposure(&base, -1.0), vec![Op::Shadows { amount: 0.8 }]),
        "saturation" => (base.clone(), vec![Op::Saturation { factor: 1.6 }]),
        "denoise" => (synth::add_noise(&base, 0.04, 3), vec![Op::Denoise { strength: 0.8 }]),
        "sharpen" => (synth::defocus(&base, 2), vec![Op::Sharpen { amount: 1.2 }]),
        "rotate" => (synth::scene(480, 320, 4.0), vec![Op::Rotate { degrees: -4.0 }]),
        "crop" => (base.clone(), vec![Op::Crop { x: 0.1, y: 0.1, w: 0.7, h: 0.7 }]),
        _ => return None,
    };
    let after = apply_stack(&before, &ops);
    Some((before, after))
}

// ───────────────────────── Fiches et état utilisateur ─────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Depth { Full, Compact }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action { TryNow, LearnMore, Close, NeverForTool, DisableAll, ResetTutorials }

#[derive(Debug, Clone, PartialEq)]
pub struct Card {
    pub tool: &'static str,
    pub depth: Depth,
    pub title: String,
    pub sections: Vec<(&'static str, String)>,
    pub params: Vec<(String, String)>,
    pub shortcut: Option<Command>,
    pub exercise: Option<&'static str>,
    pub video: Option<&'static str>,
    pub has_example: bool,
    pub doc_link: String,
    pub actions: Vec<Action>,
}

pub fn card(tool: &str, depth: Depth, lang: Lang) -> Option<Card> {
    let e = entry(tool)?;
    let mut sections = vec![("definition", e.definition.get(lang).to_string()), ("how", e.how.get(lang).to_string())];
    if depth == Depth::Full {
        sections.insert(1, ("utility", e.utility.get(lang).to_string()));
        sections.insert(2, ("situations", e.situations.get(lang).to_string()));
        sections.push(("mistakes", e.mistakes.get(lang).to_string()));
        sections.push(("tips", e.tips.get(lang).to_string()));
    }
    Some(Card { tool: e.tool, depth, title: e.name.get(lang).to_string(), sections,
        params: e.params.iter().map(|p| (p.name.get(lang).to_string(), p.effect.get(lang).to_string())).collect(),
        shortcut: e.shortcut, exercise: e.exercise, video: e.video, has_example: e.has_example, doc_link: format!("help/{}.md", e.tool),
        actions: vec![Action::TryNow, Action::LearnMore, Action::Close, Action::NeverForTool, Action::DisableAll, Action::ResetTutorials] })
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelpState { seen: BTreeSet<String>, muted: BTreeSet<String>, all_disabled: bool }

impl HelpState {
    /// Première utilisation d'un outil : retourne une fiche à afficher, ou `None`.
    /// Politique : débutant → fiche complète ; intermédiaire → fiche courte ; **expert → jamais d'interruption**.
    /// Rien si les aides sont désactivées, si l'outil est déjà vu ou masqué.
    pub fn first_use(&mut self, profile: &Profile, tool: &str) -> Option<Card> {
        if !profile.aids_enabled() || self.all_disabled || self.seen.contains(tool) || self.muted.contains(tool) { return None; }
        let depth = match profile.mode() { Mode::Expert => return None, Mode::Intermediate => Depth::Compact, Mode::Beginner => Depth::Full };
        let c = card(tool, depth, profile.lang())?;
        self.seen.insert(tool.to_string());
        Some(c)
    }
    /// Aide demandée explicitement (bouton « ? », infobulle) : toujours la fiche complète, sauf aides désactivées.
    pub fn on_request(&self, profile: &Profile, tool: &str) -> Option<Card> {
        if !profile.aids_enabled() { return None; }
        card(tool, Depth::Full, profile.lang())
    }
    pub fn never_for(&mut self, tool: &str) { self.muted.insert(tool.to_string()); }
    pub fn disable_all(&mut self) { self.all_disabled = true; }
    /// « Réinitialiser les tutoriels » : tout redevient « jamais vu » et les aides sont réactivées.
    pub fn reset(&mut self) { *self = Self::default(); }
    pub fn seen_count(&self) -> usize { self.seen.len() }

    pub fn save(&self, dir: &std::path::Path) -> std::io::Result<()> {
        std::fs::create_dir_all(dir)?;
        let tmp = dir.join("help_state.json.tmp");
        std::fs::write(&tmp, serde_json::to_vec_pretty(self)?)?;
        std::fs::rename(tmp, dir.join("help_state.json"))
    }
    pub fn load(dir: &std::path::Path) -> Self { std::fs::read(dir.join("help_state.json")).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default() }

    /// Charge utile de synchronisation : **uniquement** avec le consentement `ProgressSync`. Aucun transport n'est
    /// implémenté (pas de serveur) : cette fonction garantit seulement que rien ne partirait sans accord.
    pub fn sync_payload(&self, ledger: &ConsentLedger) -> Result<String, ConsentError> {
        ledger.require(Purpose::ProgressSync)?;
        Ok(serde_json::to_string(self).expect("sérialisable"))
    }
}

/// Fiche complète en Markdown (page « documentation complète »).
pub fn render_markdown(tool: &str, lang: Lang) -> Option<String> {
    let e = entry(tool)?;
    let fr = lang == Lang::Fr;
    let mut s = format!("# {}\n\n", e.name.get(lang));
    for (h_fr, h_en, t) in [("Définition", "Definition", e.definition), ("Utilité", "Purpose", e.utility), ("Quand l'utiliser", "When to use it", e.situations), ("Comment l'utiliser", "How to use it", e.how)] {
        s += &format!("## {}\n\n{}\n\n", if fr { h_fr } else { h_en }, t.get(lang));
    }
    if !e.params.is_empty() { s += &format!("## {}\n\n", if fr { "Effet des réglages" } else { "Effect of the settings" }); for p in e.params { s += &format!("* **{}** — {}\n", p.name.get(lang), p.effect.get(lang)); } s += "\n"; }
    s += &format!("## {}\n\n{}\n\n## {}\n\n{}\n\n", if fr { "Erreurs fréquentes" } else { "Common mistakes" }, e.mistakes.get(lang), if fr { "Conseils professionnels" } else { "Professional tips" }, e.tips.get(lang));
    if let Some(c) = e.shortcut { s += &format!("## {}\n\n`{:?}`\n\n", if fr { "Raccourci clavier" } else { "Keyboard shortcut" }, c); }
    if let Some(x) = e.exercise { s += &format!("## {}\n\n`{x}`\n\n", if fr { "Exercice" } else { "Exercise" }); }
    s += &format!("*{}*\n", if e.video.is_some() { "" } else if fr { "Aucun tutoriel vidéo disponible pour le moment." } else { "No video tutorial is available yet." });
    Some(s)
}

/// Écrit les pages d'aide complètes dans `dir/help/`.
pub fn export_docs(dir: &std::path::Path, lang: Lang) -> std::io::Result<usize> {
    let h = dir.join("help");
    std::fs::create_dir_all(&h)?;
    for e in ENTRIES { std::fs::write(h.join(format!("{}.md", e.tool)), render_markdown(e.tool, lang).unwrap())?; }
    Ok(ENTRIES.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use norelias_onboarding::profile::{Coaching, Skill};

    fn profile(mode: Mode) -> Profile { let mut p = Profile::default(); p.mode_override = Some(mode); p }
    fn mean_abs(a: &Image, b: &Image) -> f32 { a.data.iter().zip(&b.data).map(|(x, y)| (x - y).abs()).sum::<f32>() / a.data.len() as f32 }

    #[test]
    fn every_available_tool_has_a_complete_card_and_nothing_else_does() {
        for t in norelias_onboarding::tools::available() { assert!(entry(t.id).is_some(), "outil sans aide : {}", t.id); }
        for e in ENTRIES { assert_eq!(norelias_onboarding::tools::get(e.tool).map(|t| t.status), Some(norelias_onboarding::tools::Status::Available), "{} documenté mais non disponible", e.tool); }
        for e in ENTRIES { for l in [Lang::Fr, Lang::En] {
            assert!(e.name.get(l).len() >= 3, "{} nom vide", e.tool);
            for t in [e.definition, e.utility, e.situations, e.how, e.mistakes, e.tips] { assert!(t.get(l).len() >= 12, "{} champ trop court ({l:?})", e.tool); }
            assert_ne!(e.definition.0, e.definition.1);
        } }
    }
    #[test]
    fn every_slider_of_the_engine_is_documented() {
        for (op, params) in Op::PARAMS {
            let e = entry(op).unwrap_or_else(|| panic!("opération sans fiche : {op}"));
            let doc: Vec<&str> = e.params.iter().map(|p| p.id).collect();
            assert_eq!(doc, params.to_vec(), "{op} : curseurs documentés ≠ curseurs du moteur");
            for p in e.params { for l in [Lang::Fr, Lang::En] { assert!(p.effect.get(l).len() >= 12 && p.name.get(l).len() >= 2); } }
        }
    }
    #[test]
    fn no_fake_media_or_links() {
        for e in ENTRIES {
            if let Some(v) = e.video { assert!(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(v).exists(), "vidéo déclarée mais absente : {v}"); }
            if let Some(x) = e.exercise { assert!(crate::exercises::get(x).is_some(), "exercice inexistant : {x}"); }
            if let Some(c) = e.shortcut { let m = norelias_onboarding::shortcuts::ShortcutMap::for_profile(norelias_onboarding::profile::ShortcutProfileId::Norelias); assert!(m.get(c).is_some()); }
        }
        assert!(ENTRIES.iter().all(|e| e.video.is_none()), "aucune vidéo n'existe : la fiche ne doit jamais en annoncer une");
        assert!(card("crop", Depth::Full, Lang::Fr).unwrap().video.is_none());
    }
    #[test]
    fn examples_use_the_same_photograph_and_really_differ() {
        for e in ENTRIES.iter().filter(|e| e.has_example) {
            let (before, after) = example(e.tool).unwrap_or_else(|| panic!("exemple manquant : {}", e.tool));
            if e.tool != "crop" { assert_eq!((before.width, before.height), (after.width, after.height)); }
            let cmp = if e.tool == "crop" { after.resize(before.width, before.height) } else { after.clone() };
            let max = before.data.iter().zip(&cmp.data).map(|(x, y)| (x - y).abs()).fold(0.0, f32::max);
            assert!(mean_abs(&before, &cmp) > 0.004 || max > 0.05, "{} : avant ≈ après", e.tool);
        }
        assert!(example("advisor").is_none() && ENTRIES.iter().filter(|e| !e.has_example).count() == 2);
    }
    #[test]
    fn corrective_examples_move_toward_the_pristine_photo() {
        let pristine = synth::scene(480, 320, 0.0);
        for tool in ["exposure", "white_balance", "denoise"] {
            let (b, a) = example(tool).unwrap();
            assert!(mean_abs(&a, &pristine) < mean_abs(&b, &pristine) * 0.6, "{tool} : l'après n'est pas plus proche de la photo d'origine");
        }
    }
    #[test]
    fn rotate_example_really_straightens_the_horizon() {
        let (b, a) = example("rotate").unwrap();
        let (tb, ta) = (norelias_advisor::advise(&b).metrics.tilt_deg.unwrap_or(0.0), norelias_advisor::advise(&a).metrics.tilt_deg.unwrap_or(0.0));
        assert!(tb.abs() > 3.0 && ta.abs() < 0.7, "{tb}° → {ta}°");
    }
    #[test]
    fn first_use_policy_by_level() {
        let mut st = HelpState::default();
        let c = st.first_use(&profile(Mode::Beginner), "exposure").unwrap();
        assert_eq!(c.depth, Depth::Full);
        assert!(st.first_use(&profile(Mode::Beginner), "exposure").is_none(), "une seule fois");
        assert_eq!(st.first_use(&profile(Mode::Intermediate), "crop").unwrap().depth, Depth::Compact);
        assert!(st.first_use(&profile(Mode::Expert), "denoise").is_none(), "un expert n'est jamais interrompu");
        assert!(st.on_request(&profile(Mode::Expert), "denoise").is_some(), "mais il peut demander l'aide");
        assert!(st.first_use(&profile(Mode::Beginner), "outil_inexistant").is_none());
    }
    #[test]
    fn mute_disable_and_reset() {
        let (mut st, p) = (HelpState::default(), profile(Mode::Beginner));
        st.never_for("sharpen");
        assert!(st.first_use(&p, "sharpen").is_none() && st.first_use(&p, "denoise").is_some());
        st.disable_all();
        assert!(st.first_use(&p, "crop").is_none());
        st.reset();
        assert!(st.first_use(&p, "sharpen").is_some() && st.first_use(&p, "crop").is_some());
        let mut off = p.clone(); off.disable_all_aids();
        assert!(HelpState::default().first_use(&off, "crop").is_none() && HelpState::default().on_request(&off, "crop").is_none());
        let mut g = p.clone(); g.coaching = Coaching::Off; g.tooltips = true;
        assert!(HelpState::default().on_request(&g, "crop").is_some(), "infobulles encore actives");
        let _ = Skill::Beginner;
    }
    #[test]
    fn card_has_all_required_parts_and_actions() {
        let c = card("white_balance", Depth::Full, Lang::Fr).unwrap();
        let keys: Vec<_> = c.sections.iter().map(|s| s.0).collect();
        for k in ["definition", "utility", "situations", "how", "mistakes", "tips"] { assert!(keys.contains(&k), "{k}"); }
        assert_eq!(c.params.len(), 3);
        assert_eq!(c.doc_link, "help/white_balance.md");
        for a in [Action::TryNow, Action::LearnMore, Action::Close, Action::NeverForTool, Action::DisableAll, Action::ResetTutorials] { assert!(c.actions.contains(&a)); }
        assert!(card("crop", Depth::Compact, Lang::En).unwrap().sections.len() < c.sections.len());
    }
    #[test]
    fn state_persists_locally_and_sync_requires_consent() {
        let d = std::env::temp_dir().join(format!("norelias-help-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        let mut st = HelpState::default();
        st.first_use(&profile(Mode::Beginner), "crop");
        st.never_for("sharpen");
        st.save(&d).unwrap();
        assert_eq!(HelpState::load(&d), st);
        let mut ledger = ConsentLedger::default();
        assert!(st.sync_payload(&ledger).is_err(), "aucune synchronisation sans consentement");
        ledger.grant(Purpose::ProgressSync, "Norélias Cloud", false, false, 1).unwrap();
        assert!(st.sync_payload(&ledger).unwrap().contains("crop"));
        let _ = std::fs::remove_dir_all(&d);
    }
    #[test]
    fn full_documentation_pages_are_generated_and_links_resolve() {
        let d = std::env::temp_dir().join(format!("norelias-docs-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        assert_eq!(export_docs(&d, Lang::Fr).unwrap(), ENTRIES.len());
        for e in ENTRIES {
            let c = card(e.tool, Depth::Full, Lang::Fr).unwrap();
            let md = std::fs::read_to_string(d.join(&c.doc_link)).expect("le lien « documentation complète » doit pointer vers une page réelle");
            for h in ["## Définition", "## Utilité", "## Comment l'utiliser", "## Erreurs fréquentes", "## Conseils professionnels"] { assert!(md.contains(h), "{} sans {h}", e.tool); }
            assert!(md.contains("Aucun tutoriel vidéo disponible"));
        }
        let _ = std::fs::remove_dir_all(&d);
    }
}
