//! Assistant photographique **à règles** (sans IA générative, sans réseau).
//!
//! Il comprend un vocabulaire limité (FR/EN), s'appuie sur le registre d'aide et sur le conseiller pour
//! expliquer, diagnostiquer et proposer des corrections. Garanties (testées) :
//! * il ne connaît que les outils réellement disponibles et dit « prévu » pour les autres ;
//! * il ne regarde la photographie **qu'avec permission** (`share_photo`) ;
//! * il ne modifie rien sans confirmation explicite, et n'annonce un changement qu'après la confirmation
//!   du moteur de traitement (`Confirmation`) ;
//! * tout changement est prévisualisable et réversible (annuler / rétablir).
//!
//! Un assistant en langage naturel libre (modèle de langage) est **prévu** : il devra produire les mêmes
//! `Proposal` structurées, validées par ce même moteur, avec le consentement requis s'il est distant.

use norelias_academy::{help, search::fold};
use norelias_advisor::{advise, Kind};
use norelias_core::{ops::apply_stack, Image, Lang, Op};
use norelias_onboarding::tools::{self, Status};

/// Document ouvert : image source immuable + pile d'opérations (non destructif).
pub struct Document { source: Image, stack: Vec<Op>, redo: Vec<Op> }

#[derive(Debug, Clone, PartialEq)]
pub struct Confirmation { pub applied: Vec<Op>, /// Fraction des valeurs de pixels réellement modifiées par le moteur.
    pub changed_fraction: f32 }

#[derive(Debug, PartialEq, Eq)]
pub enum AssistError { NotConfirmed, NoEffect, UnknownProposal, UnsupportedOp(String) }

impl Document {
    pub fn new(source: Image) -> Self { Self { source, stack: vec![], redo: vec![] } }
    pub fn current(&self) -> Image { apply_stack(&self.source, &self.stack) }
    pub fn source(&self) -> &Image { &self.source }
    pub fn stack(&self) -> &[Op] { &self.stack }
    fn apply_ops(&mut self, ops: &[Op]) -> Result<Confirmation, AssistError> {
        for o in ops { if !Op::NAMES.contains(&o.name()) || tools::get(o.name()).map(|t| t.status) != Some(Status::Available) { return Err(AssistError::UnsupportedOp(o.name().into())); } }
        let before = self.current();
        let mut next = self.stack.clone();
        next.extend_from_slice(ops);
        let after = apply_stack(&self.source, &next);
        let same_dims = before.data.len() == after.data.len();
        let changed = if same_dims { before.data.iter().zip(&after.data).filter(|(a, b)| (*a - *b).abs() > 1e-5).count() as f32 / before.data.len() as f32 } else { 1.0 };
        if changed == 0.0 { return Err(AssistError::NoEffect); }
        self.stack = next;
        self.redo.clear();
        Ok(Confirmation { applied: ops.to_vec(), changed_fraction: changed })
    }
    pub fn undo(&mut self) -> bool { self.stack.pop().map(|o| self.redo.push(o)).is_some() }
    pub fn redo(&mut self) -> bool { self.redo.pop().map(|o| self.stack.push(o)).is_some() }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Proposal { pub id: usize, pub title: String, pub reason: String, pub ops: Vec<Op>, pub kind: Option<Kind> }

#[derive(Debug, Clone, PartialEq)]
pub struct Reply { pub text: String, pub proposals: Vec<Proposal>, pub steps: Vec<String>, pub needs_photo_permission: bool, pub used_photo: bool }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intent { Capabilities, ExplainActive, Explain, Diagnose, FixExposureDark, FixExposureBright, FixColour, FixNoise, FixSharp, FixHorizon, FixCrop, Undo, Redo, PlannedFeature, Unknown }

pub struct Assistant { pub lang: Lang, pub active_tool: Option<String>, /// Permission de lire la photographie ouverte (désactivée par défaut).
    pub share_photo: bool }

const PLANNED_WORDS: &[&str] = &["masque", "mask", "calque", "layer", "peau", "skin", "ciel", "sky", "supprime", "remove object", "objet", "inpaint", "courbe", "curve", "hsl", "lot", "batch", "export", "impression", "print", "galerie", "gallery", "ia generative", "generative"];

fn has(t: &str, words: &[&str]) -> bool { words.iter().any(|w| t.contains(w)) }

pub fn parse(text: &str, has_active_tool: bool) -> Intent {
    let t = fold(text);
    if has(&t, &["annule", "undo", "defaire", "reviens en arriere"]) { return Intent::Undo; }
    if has(&t, &["retablis", "redo", "refais"]) { return Intent::Redo; }
    if has(&t, &["que peux-tu", "que sais-tu", "what can you", "aide-moi", "help me", "capabilit", "tu peux faire"]) { return Intent::Capabilities; }
    if has(&t, &["trop sombre", "sous-expose", "sous expose", "underexpos", "too dark", "plus lumineux", "eclaircir", "brighten", "manque de lumiere"]) { return Intent::FixExposureDark; }
    if has(&t, &["trop clair", "surexpos", "overexpos", "too bright", "bruler", "brule", "blown", "assombrir"]) { return Intent::FixExposureBright; }
    if has(&t, &["dominante", "trop chaud", "trop froid", "trop jaune", "trop bleu", "trop vert", "orange", "color cast", "too warm", "too cold", "too yellow", "too blue", "too green", "balance des blancs", "white balance"]) { return Intent::FixColour; }
    if has(&t, &["bruit", "grain", "noise", "noisy"]) { return Intent::FixNoise; }
    if has(&t, &["flou", "pas net", "nettete", "blurry", "blur", "sharp", "soft"]) { return Intent::FixSharp; }
    if has(&t, &["horizon", "penche", "incline", "tilt", "straighten", "redress", "crooked"]) { return Intent::FixHorizon; }
    if has(&t, &["recadr", "crop", "composition", "tiers", "thirds"]) { return Intent::FixCrop; }
    if has(&t, &["analys", "diagnos", "corrige tout", "tout corriger", "ameliore", "fix everything", "improve", "analyze", "what is wrong", "quel est le probleme"]) { return Intent::Diagnose; }
    if has(&t, PLANNED_WORDS) { return Intent::PlannedFeature; }
    if has_active_tool && has(&t, &["ceci", "ca", "cet outil", "ce curseur", "this", "this tool", "this slider", "c'est quoi", "a quoi sert", "what is", "what does"]) { return Intent::ExplainActive; }
    if has(&t, &["c'est quoi", "a quoi sert", "explique", "what is", "what does", "how do i", "comment utiliser", "how to use", "expliquer"]) { return Intent::Explain; }
    Intent::Unknown
}

/// Retrouve un outil réel d'après son nom ou son identifiant dans le texte.
fn find_tool(text: &str) -> Option<&'static str> {
    let t = fold(text);
    let extra: &[(&str, &str)] = &[("luminosite", "exposure"), ("brightness", "exposure"), ("temperature", "white_balance"), ("grain", "denoise"), ("bruit", "denoise"), ("netteté", "sharpen"), ("nettete", "sharpen"),
        ("redress", "rotate"), ("straighten", "rotate"), ("hautes lumieres", "highlights"), ("ombres", "shadows")];
    let mut best: Option<(&str, usize)> = None;
    for e in help::ENTRIES {
        for name in [fold(e.name.0), fold(e.name.1), e.tool.replace('_', " ")] {
            if name.len() >= 4 && t.contains(&name) { let l = name.len(); if best.is_none_or(|b| l > b.1) { best = Some((e.tool, l)); } }
        }
    }
    for (w, tool) in extra { if t.contains(w) && best.is_none() { best = Some((tool, w.len())); } }
    best.map(|b| b.0)
}

fn tx(lang: Lang, fr: &str, en: &str) -> String { if lang == Lang::Fr { fr.to_string() } else { en.to_string() } }

impl Assistant {
    pub fn new(lang: Lang) -> Self { Self { lang, active_tool: None, share_photo: false } }

    /// Outils disponibles et prévus, tels que le registre les déclare.
    pub fn capabilities(&self) -> String {
        let avail: Vec<_> = tools::available().filter_map(|t| help::entry(t.id).map(|e| e.name.get(self.lang))).collect();
        let planned: Vec<_> = tools::TOOLS.iter().filter(|t| t.status == Status::Planned).map(|t| t.id.replace('_', " ")).collect();
        tx(self.lang,
            &format!("Je suis un assistant à règles (pas une IA générative). Je peux expliquer ces outils : {}. Avec votre permission, j'analyse la photo, propose des corrections prévisualisables et les applique après votre confirmation ; tout est annulable. Pas encore disponible : {}.", avail.join(", "), planned.join(", ")),
            &format!("I am a rule-based assistant (not a generative AI). I can explain these tools: {}. With your permission I analyse the photo, suggest previewable fixes and apply them after your confirmation; everything is undoable. Not available yet: {}.", avail.join(", "), planned.join(", ")))
    }

    fn explain(&self, tool: &str) -> Reply {
        let c = help::card(tool, help::Depth::Full, self.lang).expect("fiche");
        let mut text = format!("{} — {}", c.title, c.sections.iter().find(|s| s.0 == "definition").map(|s| s.1.as_str()).unwrap_or(""));
        for (k, label_fr, label_en) in [("utility", "Utilité", "Purpose"), ("tips", "Conseil", "Tip")] {
            if let Some(s) = c.sections.iter().find(|s| s.0 == k) { text += &format!("\n{} : {}", tx(self.lang, label_fr, label_en), s.1); }
        }
        let steps: Vec<String> = c.sections.iter().filter(|s| s.0 == "how").map(|s| s.1.clone()).collect();
        Reply { text, proposals: vec![], steps, needs_photo_permission: false, used_photo: false }
    }

    /// Répond à un message. `photo` n'est consultée que si `share_photo` est vrai.
    pub fn ask(&self, text: &str, photo: Option<&Document>) -> Reply {
        let intent = parse(text, self.active_tool.is_some());
        let plain = |t: String| Reply { text: t, proposals: vec![], steps: vec![], needs_photo_permission: false, used_photo: false };
        match intent {
            Intent::Capabilities => plain(self.capabilities()),
            Intent::ExplainActive => self.explain(self.active_tool.as_deref().unwrap()),
            Intent::Explain => match find_tool(text) {
                Some(t) => self.explain(t),
                None => plain(tx(self.lang, "De quel outil parlez-vous ? ", "Which tool do you mean? ") + &self.capabilities()),
            },
            Intent::PlannedFeature => plain(tx(self.lang,
                "Cette fonction est prévue mais pas encore disponible : je ne peux ni l'expliquer comme un outil actif ni l'exécuter. Voici ce qui existe : ",
                "That feature is planned but not available yet: I can neither describe it as an active tool nor run it. Here is what exists: ") + &self.capabilities()),
            Intent::Undo | Intent::Redo => plain(tx(self.lang, "Utilisez Annuler / Rétablir : seul le moteur modifie l'historique, je ne le fais pas à votre place sans confirmation.", "Use Undo / Redo: only the engine changes the history; I do not do it for you without confirmation.")),
            Intent::Unknown => plain(tx(self.lang, "Je n'ai pas compris. ", "I did not understand. ") + &self.capabilities()),
            fix => self.fix(fix, photo),
        }
    }

    fn fix(&self, intent: Intent, photo: Option<&Document>) -> Reply {
        let tool = match intent { Intent::FixExposureDark | Intent::FixExposureBright => "exposure", Intent::FixColour => "white_balance", Intent::FixNoise => "denoise",
            Intent::FixSharp => "sharpen", Intent::FixHorizon => "rotate", Intent::FixCrop => "crop", _ => "advisor" };
        let mut reply = self.explain(tool);
        let guidance = reply.clone();
        if !self.share_photo || photo.is_none() {
            // Sans permission : conseil général uniquement, aucune mesure, aucune proposition chiffrée.
            reply.text = format!("{}\n{}", tx(self.lang, "Je n'ai pas la permission de voir votre photographie : autorisez-le pour recevoir des suggestions personnalisées. En attendant, voici l'outil concerné :", "I do not have permission to see your photograph: allow it to get personalised suggestions. Meanwhile, here is the relevant tool:"), guidance.text);
            reply.needs_photo_permission = true;
            return reply;
        }
        let doc = photo.unwrap();
        let report = advise(&doc.current());
        let wanted: Vec<Kind> = match intent {
            Intent::FixExposureDark | Intent::FixExposureBright => vec![Kind::Exposure, Kind::HighlightClipping],
            Intent::FixColour => vec![Kind::WhiteBalance, Kind::ColorCast], Intent::FixNoise => vec![Kind::Noise], Intent::FixSharp => vec![Kind::Sharpness, Kind::MotionBlur],
            Intent::FixHorizon => vec![Kind::HorizonTilt], Intent::FixCrop => vec![Kind::Thirds], _ => Kind::ALL.to_vec(),
        };
        let mut proposals = vec![];
        let mut lines = vec![];
        for f in report.findings.iter().filter(|f| wanted.contains(&f.kind)) {
            lines.push(format!("• {} — {}", f.title(self.lang), f.explain(self.lang)));
            if !f.ops.is_empty() { proposals.push(Proposal { id: proposals.len(), title: f.title(self.lang), reason: f.explain(self.lang), ops: f.ops.clone(), kind: Some(f.kind) }); }
        }
        // Une demande explicitement contradictoire avec la mesure est signalée, jamais exécutée à l'aveugle.
        let measured_ev = report.metrics.exposure_ev;
        let contradicts = (intent == Intent::FixExposureDark && measured_ev < -0.3) || (intent == Intent::FixExposureBright && measured_ev > 0.3);
        reply.used_photo = true;
        reply.text = if lines.is_empty() {
            tx(self.lang, "J'ai analysé la photo : aucun problème correspondant n'est détecté. Vous restez libre de régler à votre goût avec l'outil.", "I analysed the photo: no matching problem is detected. You are free to adjust to taste with the tool.")
        } else {
            let head = if contradicts { tx(self.lang, "Attention : la mesure contredit votre impression.\n", "Note: the measurement contradicts your impression.\n") } else { String::new() };
            format!("{head}{}\n{}", tx(self.lang, "J'ai analysé la photo (localement) :", "I analysed the photo (locally):"), lines.join("\n"))
        };
        reply.proposals = proposals;
        reply.steps = guidance.steps;
        reply
    }

    /// Aperçu réduit d'une proposition ; ne modifie jamais le document.
    pub fn preview(&self, doc: &Document, p: &Proposal, max_side: usize) -> Image { apply_stack(&doc.current().downscale_to(max_side), &p.ops) }

    /// Applique une proposition **après confirmation**. Le texte retourné vient du moteur : il n'affirme un
    /// changement que si `Document::apply_ops` l'a réellement produit.
    pub fn apply(&self, doc: &mut Document, p: &Proposal, confirmed: bool) -> Result<(Confirmation, String), (AssistError, String)> {
        if !confirmed { return Err((AssistError::NotConfirmed, tx(self.lang, "Rien n'a été modifié : confirmation requise.", "Nothing was changed: confirmation required."))); }
        match doc.apply_ops(&p.ops) {
            Ok(c) => { let t = tx(self.lang, &format!("Appliqué par le moteur de traitement ({:.0} % des valeurs de pixels modifiées). Vous pouvez annuler.", c.changed_fraction * 100.0),
                &format!("Applied by the processing engine ({:.0}% of pixel values changed). You can undo.", c.changed_fraction * 100.0)); Ok((c, t)) }
            Err(e) => { let t = tx(self.lang, "Rien n'a été modifié : le moteur n'a produit aucun changement ou a refusé l'opération.", "Nothing was changed: the engine produced no change or refused the operation."); Err((e, t)) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use norelias_core::synth;
    fn dark_doc() -> Document { Document::new(synth::exposure(&synth::scene(400, 280, 0.0), -2.0)) }
    fn allowed(lang: Lang) -> Assistant { let mut a = Assistant::new(lang); a.share_photo = true; a }

    #[test]
    fn intent_parsing_both_languages_with_accents() {
        for (t, i) in [("ma photo est trop sombre", Intent::FixExposureDark), ("my photo is too dark", Intent::FixExposureDark), ("Elle est SURÉXPOSÉE", Intent::FixExposureBright),
            ("il y a une dominante orange", Intent::FixColour), ("there is noise", Intent::FixNoise), ("l'horizon est penché", Intent::FixHorizon), ("annule", Intent::Undo),
            ("analyse ma photo", Intent::Diagnose), ("que peux-tu faire ?", Intent::Capabilities), ("blabla", Intent::Unknown), ("applique un masque sur le ciel", Intent::PlannedFeature)] {
            assert_eq!(parse(t, false), i, "{t}");
        }
        assert_eq!(parse("c'est quoi ça ?", true), Intent::ExplainActive);
    }
    #[test]
    fn explains_real_tools_in_both_languages_and_knows_the_active_panel() {
        let mut a = Assistant::new(Lang::Fr);
        assert!(a.ask("c'est quoi la balance des blancs ?", None).text.contains("Balance des blancs"));
        assert!(Assistant::new(Lang::En).ask("what is noise reduction?", None).text.contains("Noise reduction"));
        a.active_tool = Some("sharpen".into());
        assert!(a.ask("à quoi sert ce curseur ?", None).text.contains("Netteté"), "utilise le panneau actif");
        assert!(a.ask("explique l'exposition", None).text.contains("Exposition"));
    }
    #[test]
    fn capabilities_list_exactly_the_registry() {
        let c = Assistant::new(Lang::En).capabilities();
        for t in tools::available() { assert!(c.contains(help::entry(t.id).unwrap().name.1), "outil réel absent : {}", t.id); }
        assert!(c.contains("tone curve") && c.contains("Not available yet"));
        let r = Assistant::new(Lang::Fr).ask("retire l'objet avec l'IA générative", None);
        assert!(r.text.contains("prévue mais pas encore disponible") && r.proposals.is_empty());
    }
    #[test]
    fn never_looks_at_the_photo_without_permission() {
        let doc = dark_doc();
        let a = Assistant::new(Lang::Fr); // share_photo = false par défaut
        let r = a.ask("ma photo est trop sombre", Some(&doc));
        assert!(r.needs_photo_permission && !r.used_photo && r.proposals.is_empty(), "aucune mesure ni proposition sans permission");
        assert!(r.text.contains("permission"));
        let mut b = allowed(Lang::Fr);
        assert!(b.ask("ma photo est trop sombre", None).needs_photo_permission, "pas de document : rien à lire");
        b.share_photo = false;
        assert!(b.ask("analyse", Some(&doc)).proposals.is_empty());
    }
    #[test]
    fn proposal_preview_confirmation_apply_undo_redo() {
        let mut doc = dark_doc();
        let original = doc.current();
        let a = allowed(Lang::Fr);
        let r = a.ask("ma photo est trop sombre", Some(&doc));
        assert!(r.used_photo && !r.proposals.is_empty());
        let p = r.proposals.iter().find(|p| p.kind == Some(Kind::Exposure)).unwrap().clone();
        assert!(matches!(p.ops[0], Op::Exposure { ev } if ev > 1.0));
        // aperçu : réduit, ne touche pas au document
        let pv = a.preview(&doc, &p, 100);
        assert!(pv.width <= 100 && doc.current() == original && doc.stack().is_empty());
        // sans confirmation : rien ne change et le texte ne prétend rien
        let (e, t) = a.apply(&mut doc, &p, false).unwrap_err();
        assert_eq!(e, AssistError::NotConfirmed);
        assert!(t.contains("Rien n'a été modifié") && doc.current() == original);
        // avec confirmation : le moteur modifie réellement
        let (c, t) = a.apply(&mut doc, &p, true).unwrap();
        assert!(c.changed_fraction > 0.9 && t.contains("Appliqué par le moteur"));
        assert_ne!(doc.current(), original);
        assert!(!advise(&doc.current()).has(Kind::Exposure), "le problème est réellement corrigé");
        // réversible
        assert!(doc.undo() && doc.current() == original && !doc.undo());
        assert!(doc.redo() && doc.current() != original);
        assert_eq!(doc.source(), &synth::exposure(&synth::scene(400, 280, 0.0), -2.0), "la source n'a jamais bougé");
    }
    #[test]
    fn a_noop_is_reported_as_nothing_changed() {
        let mut doc = dark_doc();
        let a = allowed(Lang::En);
        let p = Proposal { id: 0, title: "x".into(), reason: "".into(), ops: vec![Op::Exposure { ev: 0.0 }], kind: None };
        let (e, t) = a.apply(&mut doc, &p, true).unwrap_err();
        assert_eq!(e, AssistError::NoEffect);
        assert!(t.contains("Nothing was changed") && doc.stack().is_empty());
    }
    #[test]
    fn contradicting_requests_are_flagged_not_blindly_executed() {
        let doc = Document::new(synth::scene(400, 280, 0.0)); // image correcte
        let r = allowed(Lang::Fr).ask("elle est trop sombre", Some(&doc));
        assert!(r.proposals.is_empty() && r.text.contains("aucun problème"), "{}", r.text);
        let bright = Document::new(synth::exposure(&synth::scene(400, 280, 0.0), -2.0));
        let r = allowed(Lang::Fr).ask("elle est trop claire", Some(&bright));
        assert!(r.text.contains("contredit"), "la mesure contredit l'impression : {}", r.text);
    }
    #[test]
    fn diagnose_lists_every_finding_with_proposals_and_respects_non_automatic_ones() {
        let img = synth::add_noise(&synth::tint(&synth::exposure(&synth::scene(480, 320, 0.0), -1.8), [1.25, 1.0, 0.8]), 0.03, 7);
        let mut doc = Document::new(img);
        let a = allowed(Lang::En);
        let r = a.ask("analyze my photo", Some(&doc));
        let kinds: Vec<_> = r.proposals.iter().filter_map(|p| p.kind).collect();
        for k in [Kind::Exposure, Kind::WhiteBalance, Kind::Noise] { assert!(kinds.contains(&k), "{k:?} manquant : {kinds:?}"); }
        let before = doc.current();
        for p in &r.proposals { a.apply(&mut doc, p, true).ok(); }
        assert_ne!(doc.current(), before);
        assert!(doc.stack().iter().all(|o| Op::NAMES.contains(&o.name())), "uniquement des opérations réellement prises en charge");
    }
    #[test]
    fn english_replies_are_english() {
        let doc = dark_doc();
        let r = allowed(Lang::En).ask("my photo is too dark", Some(&doc));
        assert!(r.text.contains("I analysed") && !r.text.contains("J'ai analysé"));
    }
}
