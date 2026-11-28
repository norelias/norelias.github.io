//! Questionnaire initial (logique indépendante de l'interface : l'interface graphique appellera
//! les mêmes fonctions que ce parcours texte). Toutes les questions sont facultatives :
//! Entrée conserve la valeur actuelle, `s` passe le reste du questionnaire.

use crate::privacy::{ConsentLedger, Purpose};
use crate::profile::*;
use norelias_core::Lang;
use std::io::{self, BufRead, Write};

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome { Completed, Skipped, /// Entrée fermée : aucun changement de statut.
    Aborted }

struct Q<'a, R: BufRead, W: Write> { input: &'a mut R, out: &'a mut W, lang: Lang, skipped: bool, eof: bool }

enum Answer<T> { Keep, Value(T) }

impl<R: BufRead, W: Write> Q<'_, R, W> {
    fn t<'s>(&self, fr: &'s str, en: &'s str) -> &'s str { if self.lang == Lang::Fr { fr } else { en } }

    fn line(&mut self) -> io::Result<Option<String>> {
        if self.skipped || self.eof { return Ok(None); }
        let mut s = String::new();
        if self.input.read_line(&mut s)? == 0 { self.eof = true; return Ok(None); }
        let s = s.trim().to_string();
        if s.eq_ignore_ascii_case("s") { self.skipped = true; return Ok(None); }
        Ok(Some(s))
    }

    fn header(&mut self, fr: &str, en: &str) -> io::Result<()> { writeln!(self.out, "\n── {}", self.t(fr, en)) }

    fn list<T: Clone>(&mut self, opts: &[(T, &str, &str)], multi: bool) -> io::Result<Answer<Vec<T>>> {
        for (i, (_, fr, en)) in opts.iter().enumerate() { writeln!(self.out, "  {:>2}. {}", i + 1, self.t(fr, en))?; }
        let hint = if multi { self.t("numéros séparés par des virgules, Entrée = conserver, s = passer", "numbers separated by commas, Enter = keep, s = skip") }
                   else { self.t("un numéro, Entrée = conserver, s = passer", "one number, Enter = keep, s = skip") };
        for _ in 0..3 {
            write!(self.out, "  > ({hint}) ")?;
            self.out.flush()?;
            let Some(l) = self.line()? else { return Ok(Answer::Keep) };
            if l.is_empty() { return Ok(Answer::Keep); }
            let parsed: Option<Vec<T>> = l.split(',').map(|p| p.trim().parse::<usize>().ok().and_then(|n| n.checked_sub(1)).and_then(|i| opts.get(i)).map(|o| o.0.clone())).collect();
            match parsed {
                Some(v) if multi || v.len() == 1 => return Ok(Answer::Value(v)),
                _ => writeln!(self.out, "  {}", self.t("Réponse non reconnue.", "Answer not recognised."))?,
            }
        }
        Ok(Answer::Keep)
    }
    fn one<T: Clone>(&mut self, opts: &[(T, &str, &str)]) -> io::Result<Option<T>> {
        Ok(match self.list(opts, false)? { Answer::Value(v) => v.into_iter().next(), Answer::Keep => None })
    }
    fn many<T: Clone>(&mut self, opts: &[(T, &str, &str)]) -> io::Result<Option<Vec<T>>> {
        Ok(match self.list(opts, true)? { Answer::Value(v) => Some(v), Answer::Keep => None })
    }
    fn text(&mut self, prompt_fr: &str, prompt_en: &str) -> io::Result<Option<String>> {
        write!(self.out, "  > {} ", self.t(prompt_fr, prompt_en))?;
        self.out.flush()?;
        Ok(self.line()?.filter(|s| !s.is_empty()))
    }
    fn yes_no(&mut self, fr: &str, en: &str, current: bool) -> io::Result<bool> {
        write!(self.out, "  > {} [{}] ", self.t(fr, en), if current { self.t("O/n", "Y/n") } else { self.t("o/N", "y/N") })?;
        self.out.flush()?;
        Ok(match self.line()?.map(|s| s.to_lowercase()) {
            Some(s) if ["o", "oui", "y", "yes"].contains(&s.as_str()) => true,
            Some(s) if ["n", "non", "no"].contains(&s.as_str()) => false,
            _ => current,
        })
    }
}

const SKILLS: [(Skill, &str, &str); 4] = [(Skill::Beginner, "Débutant", "Beginner"), (Skill::Intermediate, "Intermédiaire", "Intermediate"), (Skill::Advanced, "Avancé", "Advanced"), (Skill::Professional, "Professionnel", "Professional")];

/// Parcours complet. `now` sert à dater un éventuel consentement. Le profil et le registre sont modifiés en place.
pub fn run(profile: &mut Profile, ledger: &mut ConsentLedger, input: &mut impl BufRead, out: &mut impl Write, now: u64) -> io::Result<Outcome> {
    let mut q = Q { input, out, lang: profile.lang(), skipped: false, eof: false };

    writeln!(q.out, "{}", q.t("Bienvenue dans Norélias Studio Pro. Toutes les questions sont facultatives et modifiables plus tard.",
        "Welcome to Norélias Studio Pro. Every question is optional and can be changed later."))?;
    writeln!(q.out, "{}", q.t("Vos réponses restent sur cet ordinateur. Aucun compte requis.", "Your answers stay on this computer. No account required."))?;

    q.header("Langue de l'interface", "Interface language")?;
    if let Some(l) = q.one(&[(Language::Fr, "Français", "Français"), (Language::En, "English", "English"), (Language::System, "Langue du système", "System language")])? {
        profile.language = l;
        q.lang = profile.lang();
    }
    q.header("Apparence", "Appearance")?;
    if let Some(t) = q.one(&[(Theme::Dark, "Sombre", "Dark"), (Theme::Light, "Clair", "Light"), (Theme::System, "Système", "System")])? { profile.theme = t; }

    q.header("Votre niveau en photographie", "Your photography level")?;
    if let Some(s) = q.one(&SKILLS)? { profile.photo_level = s; }
    q.header("Votre niveau en retouche numérique", "Your digital editing level")?;
    if let Some(s) = q.one(&SKILLS)? { profile.retouch_level = s; }

    q.header("Vos spécialités", "Your specialties")?;
    let spec: Vec<(Specialty, &str, &str)> = Specialty::ALL.iter().map(|s| { let w = crate::workspace::workspace_for(*s); (*s, w.fr, w.en) }).collect();
    if let Some(v) = q.many(&spec)? { profile.specialties = v; }

    q.header("Logiciels déjà utilisés", "Software you have used")?;
    let sw = [(LegacySoftware::Photoshop, "Photoshop", "Photoshop"), (LegacySoftware::Lightroom, "Lightroom", "Lightroom"), (LegacySoftware::CaptureOne, "Capture One", "Capture One"),
        (LegacySoftware::Evoto, "Evoto", "Evoto"), (LegacySoftware::LuminarNeo, "Luminar Neo", "Luminar Neo"), (LegacySoftware::TopazPhotoAi, "Topaz Photo AI", "Topaz Photo AI"),
        (LegacySoftware::PhotoMechanic, "Photo Mechanic", "Photo Mechanic"), (LegacySoftware::None, "Aucun", "None")];
    if let Some(v) = q.many(&sw)? { profile.previous_software = v; }

    q.header("Votre appareil photo", "Your camera")?;
    let make = q.text("Marque (Entrée = ignorer) :", "Make (Enter = skip):")?;
    if let Some(make) = make {
        let model = q.text("Modèle :", "Model:")?.unwrap_or_default();
        if profile.set_camera(&make, &model).is_err() { writeln!(q.out, "  {}", q.t("Texte invalide ignoré.", "Invalid text ignored."))?; }
    }

    q.header("Vos objectifs", "Your goals")?;
    if let Some(v) = q.many(&[(Goal::Hobby, "Loisir", "Hobby"), (Goal::Portfolio, "Portfolio", "Portfolio"), (Goal::PaidWork, "Travail rémunéré", "Paid work"),
        (Goal::Learn, "Apprendre", "Learn"), (Goal::SaveTime, "Gagner du temps", "Save time"), (Goal::Archive, "Archiver", "Archive")])? { profile.goals = v; }

    q.header("Besoins en intelligence artificielle", "Artificial intelligence needs")?;
    if let Some(v) = q.many(&[(AiNeed::Selection, "Sélection / détourage", "Selection / cut-out"), (AiNeed::Denoise, "Réduction du bruit", "Noise reduction"),
        (AiNeed::Upscale, "Agrandissement", "Upscaling"), (AiNeed::ObjectRemoval, "Suppression d'objets", "Object removal"), (AiNeed::PortraitRetouch, "Retouche portrait", "Portrait retouching"),
        (AiNeed::NaturalLanguage, "Assistant en langage naturel", "Natural-language assistant"), (AiNeed::None, "Aucun", "None")])? { profile.ai_needs = v; }
    writeln!(q.out, "  {}", q.t("(Les outils IA locaux ne sont pas encore intégrés ; aucune photo ne quittera cet ordinateur sans votre accord explicite.)",
        "(Local AI tools are not integrated yet; no photo will leave this computer without your explicit consent.)"))?;

    q.header("Impression et livraison", "Printing and delivery")?;
    if let Some(v) = q.many(&[(PrintNeed::Prints, "Tirages", "Prints"), (PrintNeed::Albums, "Albums", "Albums"), (PrintNeed::ClientGalleries, "Galeries clients", "Client galleries"),
        (PrintNeed::WebExport, "Export Web", "Web export"), (PrintNeed::SocialMedia, "Réseaux sociaux", "Social media"), (PrintNeed::Products, "Produits dérivés", "Merchandise")])? { profile.print_needs = v; }

    q.header("Accompagnement", "Guidance")?;
    if let Some(c) = q.one(&[(Coaching::Guided, "Guidé : suggestions et leçons proposées", "Guided: suggestions and lessons offered"),
        (Coaching::OnDemand, "À la demande : seulement si je clique sur l'aide", "On demand: only when I click help"), (Coaching::Off, "Aucun accompagnement", "No guidance")])? { profile.coaching = c; }
    profile.tooltips = q.yes_no("Infobulles explicatives au survol ?", "Explanatory tooltips on hover?", profile.tooltips)?;
    profile.track_progress = q.yes_no("Suivre ma progression dans l'Académie (stockée localement) ?", "Track my progress in the Academy (stored locally)?", profile.track_progress)?;

    q.header("Mode de l'espace de travail", "Workspace mode")?;
    let suggested = profile.suggested_mode();
    writeln!(q.out, "  {} {:?}", q.t("Proposé :", "Suggested:"), suggested)?;
    if let Some(m) = q.one(&[(Mode::Beginner, "Débutant", "Beginner"), (Mode::Intermediate, "Intermédiaire", "Intermediate"), (Mode::Expert, "Expert", "Expert")])? { profile.mode_override = Some(m); }

    q.header("Raccourcis clavier", "Keyboard shortcuts")?;
    profile.shortcut_profile = profile.suggested_shortcut_profile();
    writeln!(q.out, "  {} {:?}", q.t("Proposé selon vos logiciels :", "Suggested from your software:"), profile.shortcut_profile)?;
    if let Some(s) = q.one(&[(ShortcutProfileId::Norelias, "Norélias", "Norélias"), (ShortcutProfileId::PhotoshopLike, "Type Photoshop", "Photoshop-like"),
        (ShortcutProfileId::LightroomLike, "Type Lightroom", "Lightroom-like"), (ShortcutProfileId::CaptureOneLike, "Type Capture One", "Capture One-like")])? { profile.shortcut_profile = s; }

    q.header("Stockage et confidentialité", "Storage and privacy")?;
    if let Some(d) = q.text("Dossier du catalogue (Entrée = emplacement par défaut) :", "Catalog folder (Enter = default location):")? {
        if d.chars().any(|c| c.is_control()) { writeln!(q.out, "  {}", q.t("Chemin invalide ignoré.", "Invalid path ignored."))?; } else { profile.storage.catalog_dir = Some(d); }
    }
    profile.storage.auto_backup = q.yes_no("Sauvegarde automatique du catalogue ?", "Automatic catalog backup?", profile.storage.auto_backup)?;
    let telemetry = q.yes_no("Partager des statistiques d'usage anonymes ? (non par défaut)", "Share anonymous usage statistics? (off by default)", ledger.is_granted(Purpose::Telemetry))?;
    if q.skipped {
        // Interrompu avant la question de télémétrie : on ne change rien au consentement.
    } else if telemetry && !ledger.is_granted(Purpose::Telemetry) {
        ledger.grant(Purpose::Telemetry, "Norélias (statistiques anonymes)", false, false, now).expect("fournisseur nommé");
    } else if !telemetry {
        ledger.revoke(Purpose::Telemetry);
    }
    writeln!(q.out, "  {}", q.t("Les services IA cloud et l'hébergement de galeries demanderont votre accord au moment de l'utilisation, avec le coût s'il y en a un.",
        "Cloud AI services and gallery hosting will ask for your consent at the time of use, with the cost if any."))?;

    if q.skipped { profile.skip(); return Ok(Outcome::Skipped); }
    if q.eof { return Ok(Outcome::Aborted); }
    profile.complete();
    Ok(Outcome::Completed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn go(profile: &mut Profile, ledger: &mut ConsentLedger, script: &str) -> (Outcome, String) {
        let mut out = Vec::new();
        let o = run(profile, ledger, &mut Cursor::new(script.as_bytes().to_vec()), &mut out, 1000).unwrap();
        (o, String::from_utf8(out).unwrap())
    }

    // 1 langue, 2 thème, 3 niveau photo, 4 niveau retouche, 5 spécialités, 6 logiciels, 7 marque, 8 modèle,
    // 9 objectifs, 10 IA, 11 impression, 12 accompagnement, 13 infobulles, 14 suivi, 15 mode, 16 raccourcis,
    // 17 dossier, 18 sauvegarde, 19 télémétrie
    const FULL: &str = "1\n1\n4\n2\n5,1\n2,3\nNikon\nD5200\n3,4\n1,4\n1,3\n1\nn\no\n3\n3\n/photos/catalogue\nn\no\n";

    #[test]
    fn full_answers_populate_every_field() {
        let (mut p, mut l) = (Profile::default(), ConsentLedger::default());
        let (o, _) = go(&mut p, &mut l, FULL);
        assert_eq!(o, Outcome::Completed);
        assert_eq!(p.language, Language::Fr);
        assert_eq!(p.theme, Theme::Dark);
        assert_eq!(p.photo_level, Skill::Professional);
        assert_eq!(p.retouch_level, Skill::Intermediate);
        assert_eq!(p.specialties, vec![Specialty::Landscape, Specialty::Portrait]);
        assert_eq!(p.previous_software, vec![LegacySoftware::Lightroom, LegacySoftware::CaptureOne]);
        assert_eq!(p.camera.as_ref().unwrap().model, "D5200");
        assert_eq!(p.goals, vec![Goal::PaidWork, Goal::Learn]);
        assert_eq!(p.ai_needs, vec![AiNeed::Selection, AiNeed::ObjectRemoval]);
        assert_eq!(p.print_needs, vec![PrintNeed::Prints, PrintNeed::ClientGalleries]);
        assert_eq!(p.coaching, Coaching::Guided);
        assert!(!p.tooltips && p.track_progress);
        assert_eq!(p.mode_override, Some(Mode::Expert));
        assert_eq!(p.shortcut_profile, ShortcutProfileId::LightroomLike);
        assert_eq!(p.storage.catalog_dir.as_deref(), Some("/photos/catalogue"));
        assert!(!p.storage.auto_backup);
        assert!(l.is_granted(Purpose::Telemetry), "télémétrie accordée explicitement");
        assert!(!p.needs_onboarding());
    }

    #[test]
    fn skip_at_any_point_keeps_defaults_and_grants_nothing() {
        let (mut p, mut l) = (Profile::default(), ConsentLedger::default());
        let (o, _) = go(&mut p, &mut l, "s\n");
        assert_eq!(o, Outcome::Skipped);
        assert!(!p.needs_onboarding());
        assert_eq!(p.photo_level, Profile::default().photo_level);
        for pu in [Purpose::Telemetry, Purpose::CloudAi, Purpose::CrashReports] { assert!(!l.is_granted(pu)); }
    }

    #[test]
    fn partial_then_skip_keeps_answers_given_so_far() {
        let (mut p, mut l) = (Profile::default(), ConsentLedger::default());
        let (o, _) = go(&mut p, &mut l, "2\n2\n3\ns\n");
        assert_eq!((o, p.language, p.theme, p.photo_level), (Outcome::Skipped, Language::En, Theme::Light, Skill::Advanced));
    }

    #[test]
    fn all_blank_answers_complete_with_defaults_and_no_consent() {
        let (mut p, mut l) = (Profile::default(), ConsentLedger::default());
        let (o, _) = go(&mut p, &mut l, &"\n".repeat(30));
        assert_eq!(o, Outcome::Completed);
        assert!(!l.is_granted(Purpose::Telemetry));
        assert!(!p.track_progress, "le suivi reste désactivé sans réponse explicite");
    }

    #[test]
    fn invalid_input_is_rejected_then_defaults_kept() {
        let (mut p, mut l) = (Profile::default(), ConsentLedger::default());
        let (_, out) = go(&mut p, &mut l, "99\nabc\n1,2\n\n\n\n");
        assert!(out.contains("non reconnue") || out.contains("not recognised"));
        assert_eq!(p.language, Profile::default().language);
    }

    #[test]
    fn closed_input_aborts_without_changing_status() {
        let (mut p, mut l) = (Profile::default(), ConsentLedger::default());
        let (o, _) = go(&mut p, &mut l, "2\n");
        assert_eq!(o, Outcome::Aborted);
        assert!(p.needs_onboarding());
    }

    #[test]
    fn rerunning_can_revoke_telemetry() {
        let (mut p, mut l) = (Profile::default(), ConsentLedger::default());
        go(&mut p, &mut l, FULL);
        assert!(l.is_granted(Purpose::Telemetry));
        // second passage : 17 réponses vides (tout conserver), puis « non » à la télémétrie
        let script = format!("{}n\n", "\n".repeat(17));
        let (o, _) = go(&mut p, &mut l, &script);
        assert_eq!(o, Outcome::Completed);
        assert!(!l.is_granted(Purpose::Telemetry));
    }

    #[test]
    fn interface_switches_language_mid_questionnaire() {
        let (mut p, mut l) = (Profile::default(), ConsentLedger::default());
        p.language = Language::Fr;
        let (_, out) = go(&mut p, &mut l, "2\n\ns\n");
        assert!(out.contains("Welcome") || out.contains("Bienvenue"));
        assert!(out.contains("Appearance"), "après avoir choisi l'anglais, les questions suivantes sont en anglais");
    }
}
