//! Interface en ligne de commande de Norélias Studio Pro : preuve de bout en bout des modules
//! intégrés (conseiller, profil, espaces de travail, Académie, confidentialité) sur de vrais fichiers.
//! L'interface graphique de bureau (Tauri) appellera les mêmes bibliothèques.

use norelias_academy::{exercises, lessons, search, tooltips};
use norelias_advisor::{advise, apply_selected, auto_apply, Finding, Kind, Lang};
use norelias_core::Image;
use norelias_onboarding::{
    privacy::Purpose, profile::Mode, profile::Specialty, questionnaire, store::Store, workspace::Layout,
};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => { eprintln!("erreur : {e}"); ExitCode::FAILURE }
    }
}

type R = Result<(), Box<dyn std::error::Error>>;

const USAGE: &str = "norelias <commande>
  analyze <image.png|jpg> [--lang fr|en] [--auto | --only id,id] [--out sortie.png]
  profile  show|init|skip|erase            (--dir <dossier> pour changer l'emplacement)
  workspace beginner|intermediate|expert [specialite]
  academy  search <mots> | path | exercise <id> | training <id> <sortie.png>
  privacy                                   (affiche les consentements accordés)
  tools                                     (outils réels et outils prévus)";

fn flag(args: &[String], name: &str) -> Option<String> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned()
}
fn has(args: &[String], name: &str) -> bool { args.iter().any(|a| a == name) }

fn data_dir(args: &[String]) -> PathBuf {
    if let Some(d) = flag(args, "--dir") { return d.into(); }
    if let Ok(d) = std::env::var("NORELIAS_DIR") { return d.into(); }
    if let Ok(d) = std::env::var("XDG_CONFIG_HOME") { return Path::new(&d).join("norelias"); }
    Path::new(&std::env::var("HOME").unwrap_or_else(|_| ".".into())).join(".config/norelias")
}

fn lang_of(args: &[String], store: &Store) -> Lang {
    match flag(args, "--lang").as_deref() {
        Some("en") => Lang::En,
        Some("fr") => Lang::Fr,
        _ => store.load().map(|l| l.profile.lang()).unwrap_or(Lang::Fr),
    }
}

fn run(args: &[String]) -> R {
    let store = Store::new(data_dir(args));
    match args.first().map(String::as_str) {
        Some("analyze") => analyze(args, &store),
        Some("profile") => profile(args, &store),
        Some("workspace") => workspace(args, &store),
        Some("academy") => academy(args, &store),
        Some("privacy") => privacy(&store),
        Some("tools") => {
            for t in norelias_onboarding::tools::TOOLS {
                println!("{:<18} {:<10} {:?}", t.id, t.panel, t.status);
            }
            Ok(())
        }
        _ => { println!("{USAGE}"); Ok(()) }
    }
}

fn load_image(path: &str) -> Result<Image, Box<dyn std::error::Error>> {
    let rgb = image::open(path)?.to_rgb8(); // profil ICC ignoré : sRGB supposé (limite documentée)
    Ok(Image::from_srgb8(rgb.width() as usize, rgb.height() as usize, rgb.as_raw())?)
}

fn save_image(img: &Image, path: &Path) -> R {
    image::save_buffer(path, &img.to_srgb8(), img.width as u32, img.height as u32, image::ColorType::Rgb8)?;
    Ok(())
}

/// Un original n'est jamais écrasé : la sortie ne doit ni être le fichier source ni déjà exister.
fn check_output(input: &str, out: &Path) -> R {
    if out.exists() {
        let same = std::fs::canonicalize(input).ok() == std::fs::canonicalize(out).ok();
        return Err(if same { "refus : la sortie est le fichier original, qui n'est jamais modifié".into() }
                   else { format!("refus : {} existe déjà (aucun fichier n'est écrasé)", out.display()) }.into());
    }
    Ok(())
}

fn analyze(args: &[String], store: &Store) -> R {
    let input = args.get(1).ok_or("fichier image manquant")?;
    let lang = lang_of(args, store);
    let out = flag(args, "--out").map(PathBuf::from);
    if let Some(o) = &out { check_output(input, o)?; }
    let img = load_image(input)?;
    let report = advise(&img);
    let m = &report.metrics;
    println!("{}  {}×{}", input, img.width, img.height);
    println!("médiane {:.3} ({:+.2} IL vs gris moyen) · écrêtage {:.2} % · bruit σ={:.4} · netteté {:.2} · inclinaison {}",
        m.median_linear, m.exposure_ev, m.clipped_high * 100.0, m.noise_sigma, m.sharp_ratio,
        m.tilt_deg.map_or("—".into(), |t| format!("{t:+.1}°")));
    if report.findings.is_empty() {
        println!("{}", if lang == Lang::Fr { "Aucun problème détecté." } else { "No problem detected." });
    }
    for f in &report.findings {
        println!("\n[{:?}] {}{}\n  {}", f.severity, f.title(lang), if f.auto_safe { "" } else { "  (jamais appliqué automatiquement)" }, f.explain(lang));
        println!("  ops: {}", if f.ops.is_empty() { "aucune".to_string() } else { format!("{:?}", f.ops) });
    }
    if let Some(o) = out {
        let corrected = if has(args, "--auto") {
            let (img2, applied) = auto_apply(&img, &report);
            println!("\nCorrections automatiques appliquées : {:?}", applied.iter().map(|k| k.id()).collect::<Vec<_>>());
            img2
        } else if let Some(list) = flag(args, "--only") {
            let kinds: Vec<Kind> = list.split(',').map(|s| Kind::from_id(s.trim()).ok_or(format!("type inconnu : {s}"))).collect::<Result<_, _>>()?;
            let chosen: Vec<&Finding> = report.findings.iter().filter(|f| kinds.contains(&f.kind)).collect();
            apply_selected(&img, &chosen)
        } else {
            return Err("--out demande --auto ou --only <ids> : vous gardez le contrôle des corrections".into());
        };
        save_image(&corrected, &o)?;
        println!("Écrit : {} (l'original est intact)", o.display());
    }
    Ok(())
}

fn profile(args: &[String], store: &Store) -> R {
    let mut loaded = store.load()?;
    match args.get(1).map(String::as_str) {
        Some("show") | None => {
            println!("{}", serde_json_pretty(&loaded.profile)?);
            if let Some(q) = loaded.quarantined { eprintln!("fichier corrompu mis de côté : {}", q.display()); }
        }
        Some("init") => {
            let mut ledger = store.load_ledger()?;
            let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs();
            let o = questionnaire::run(&mut loaded.profile, &mut ledger, &mut io::stdin().lock(), &mut io::stdout(), now)?;
            if o != questionnaire::Outcome::Aborted { store.save(&loaded.profile)?; store.save_ledger(&ledger)?; }
            println!("\n{o:?}");
        }
        Some("skip") => { loaded.profile.skip(); store.save(&loaded.profile)?; println!("Questionnaire passé. Il reste disponible via « profile init »."); }
        Some("erase") => { println!("{} fichier(s) supprimé(s)", store.erase_all()?); }
        Some(x) => return Err(format!("sous-commande inconnue : {x}").into()),
    }
    Ok(())
}

fn serde_json_pretty<T: serde::Serialize>(v: &T) -> Result<String, serde_json::Error> { serde_json::to_string_pretty(v) }

fn workspace(args: &[String], store: &Store) -> R {
    let mode = match args.get(1).map(String::as_str) {
        Some("beginner") => Mode::Beginner, Some("intermediate") => Mode::Intermediate, Some("expert") => Mode::Expert,
        _ => return Err("mode attendu : beginner|intermediate|expert".into()),
    };
    let specs: Vec<Specialty> = match args.get(2) {
        Some(s) => vec![Specialty::ALL.into_iter().find(|x| norelias_onboarding::workspace::workspace_for(*x).id == s).ok_or("spécialité inconnue")?],
        None => vec![],
    };
    let aids = store.load().map(|l| l.profile.aids_enabled()).unwrap_or(true);
    let l = Layout::build(mode, &specs, aids);
    println!("mode {:?} · détail {:?} · explications {:?} · recommandations {:?} · tutoriels {}", l.mode, l.settings_detail, l.explanations, l.recommendations, l.tutorial_prompts);
    for (id, v, st) in &l.tools { println!("  {:<18} {:?}{}", id, v, if *st == norelias_onboarding::tools::Status::Planned { "  [prévu]" } else { "" }); }
    println!("{} outils accessibles sur {} : aucun supprimé.", l.reachable().len(), norelias_onboarding::tools::TOOLS.len());
    Ok(())
}

fn academy(args: &[String], store: &Store) -> R {
    let lang = lang_of(args, store);
    match args.get(1).map(String::as_str) {
        Some("search") => {
            let q = args[2..].iter().take_while(|a| !a.starts_with("--")).cloned().collect::<Vec<_>>().join(" ");
            for h in search::search(&q, lang) {
                let title = match h.kind {
                    search::HitKind::Lesson => lessons::get(h.id).unwrap().title(lang).to_string(),
                    search::HitKind::Tool => tooltips::text(h.id, lang).unwrap_or("").to_string(),
                };
                println!("{:>3}  {:?}:{}  {}", h.score, h.kind, h.id, title);
            }
        }
        Some("path") => {
            let p = store.load()?.profile;
            for l in lessons::path_for(&p) { println!("[{:?}] {} — {}", l.level, l.id, l.title(lang)); }
        }
        Some("exercise") => {
            let ex = exercises::get(args.get(2).ok_or("identifiant manquant")?).ok_or("exercice inconnu")?;
            println!("{}", ex.instruction(lang));
            println!("Image d'entraînement : « academy training {} <sortie.png> »", ex.id);
        }
        Some("training") => {
            let ex = exercises::get(args.get(2).ok_or("identifiant manquant")?).ok_or("exercice inconnu")?;
            let out = PathBuf::from(args.get(3).ok_or("sortie manquante")?);
            if out.exists() { return Err(format!("refus : {} existe déjà", out.display()).into()); }
            save_image(&ex.image(), &out)?;
            println!("Photographie d'entraînement générée (libre de droits) : {}", out.display());
        }
        _ => return Err("academy search|path|exercise|training".into()),
    }
    io::stdout().flush()?;
    Ok(())
}

fn privacy(store: &Store) -> R {
    let l = store.load_ledger()?;
    for p in [Purpose::CloudAi, Purpose::ModelDownload, Purpose::GalleryHosting, Purpose::Telemetry, Purpose::CrashReports] {
        match l.require(p) {
            Ok(c) => println!("{p:?}: ACCORDÉ à « {} »{}", c.provider, if c.paid { " (payant)" } else { "" }),
            Err(_) => println!("{p:?}: refusé (par défaut)"),
        }
    }
    Ok(())
}
