//! Tests de bout en bout : le vrai binaire, de vrais fichiers PNG, un vrai dossier de données.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn tmpdir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("norelias-e2e-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}
fn bin(dir: &Path) -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_norelias"));
    c.env("NORELIAS_DIR", dir.join("data")).env_remove("XDG_CONFIG_HOME");
    c
}
fn run(c: &mut Command) -> (bool, String, String) {
    let o = c.output().unwrap();
    (o.status.success(), String::from_utf8_lossy(&o.stdout).into(), String::from_utf8_lossy(&o.stderr).into())
}
fn sha(p: &Path) -> Vec<u8> { std::fs::read(p).unwrap() }

#[test]
fn training_photo_is_analysed_corrected_and_original_preserved() {
    let d = tmpdir("analyze");
    let photo = d.join("sombre.png");
    let (ok, ..) = run(bin(&d).args(["academy", "training", "ex_exposure", photo.to_str().unwrap()]));
    assert!(ok);
    let original = sha(&photo);

    let (ok, out, _) = run(bin(&d).args(["analyze", photo.to_str().unwrap(), "--lang", "fr"]));
    assert!(ok);
    assert!(out.contains("sous-exposée"), "{out}");

    // sans choix explicite de corrections, aucune sortie n'est produite
    let corrected = d.join("corrigee.png");
    let (ok, _, err) = run(bin(&d).args(["analyze", photo.to_str().unwrap(), "--out", corrected.to_str().unwrap()]));
    assert!(!ok && err.contains("--auto") && !corrected.exists());

    let (ok, out, err) = run(bin(&d).args(["analyze", photo.to_str().unwrap(), "--auto", "--out", corrected.to_str().unwrap()]));
    assert!(ok, "{err}");
    assert!(out.contains("exposure"), "{out}");
    assert!(corrected.exists());
    assert_eq!(sha(&photo), original, "l'original ne doit jamais changer");

    // la photo corrigée n'a plus le défaut
    let (_, out2, _) = run(bin(&d).args(["analyze", corrected.to_str().unwrap(), "--lang", "en"]));
    assert!(!out2.contains("Underexposed"), "{out2}");
}

#[test]
fn never_overwrites_original_or_existing_file() {
    let d = tmpdir("overwrite");
    let photo = d.join("a.png");
    run(bin(&d).args(["academy", "training", "ex_noise", photo.to_str().unwrap()]));
    let original = sha(&photo);
    let (ok, _, err) = run(bin(&d).args(["analyze", photo.to_str().unwrap(), "--auto", "--out", photo.to_str().unwrap()]));
    assert!(!ok && err.contains("original"), "{err}");
    assert_eq!(sha(&photo), original);
    let other = d.join("b.png");
    std::fs::write(&other, b"precieux").unwrap();
    let (ok, ..) = run(bin(&d).args(["analyze", photo.to_str().unwrap(), "--auto", "--out", other.to_str().unwrap()]));
    assert!(!ok);
    assert_eq!(std::fs::read(&other).unwrap(), b"precieux");
}

#[test]
fn only_applies_the_chosen_corrections() {
    let d = tmpdir("only");
    let photo = d.join("chaude.png");
    run(bin(&d).args(["academy", "training", "ex_white_balance", photo.to_str().unwrap()]));
    let out = d.join("wb.png");
    let (ok, stdout, err) = run(bin(&d).args(["analyze", photo.to_str().unwrap(), "--only", "white_balance", "--out", out.to_str().unwrap()]));
    assert!(ok, "{err}");
    assert!(out.exists() && stdout.contains("intact"));
    let (ok, _, err) = run(bin(&d).args(["analyze", photo.to_str().unwrap(), "--only", "inconnu", "--out", d.join("x.png").to_str().unwrap()]));
    assert!(!ok && err.contains("inconnu"));
}

#[test]
fn first_run_is_local_private_and_skippable() {
    let d = tmpdir("profile");
    let (ok, out, _) = run(bin(&d).args(["profile", "show"]));
    assert!(ok && out.contains("\"status\": \"pending\"") && out.contains("\"track_progress\": false"), "{out}");
    let (_, out, _) = run(bin(&d).arg("privacy"));
    assert_eq!(out.matches("refusé (par défaut)").count(), 8, "{out}");
    run(bin(&d).args(["profile", "skip"]));
    let (_, out, _) = run(bin(&d).args(["profile", "show"]));
    assert!(out.contains("\"status\": \"skipped\""));
    // aucune sortie réseau : le dossier ne contient que du JSON local
    let names: Vec<_> = std::fs::read_dir(d.join("data")).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().into_owned()).collect();
    assert!(names.iter().all(|n| n.starts_with("profile.json")), "{names:?}");
    let (_, out, _) = run(bin(&d).args(["profile", "erase"]));
    assert!(out.contains("supprimé"));
}

#[test]
fn questionnaire_over_stdin_persists_answers_and_telemetry_consent() {
    let d = tmpdir("init");
    let mut c = bin(&d);
    let mut child = c.args(["profile", "init"]).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    child.stdin.take().unwrap().write_all(b"1\n1\n4\n4\n5\n2\nNikon\nD5200\n3\n\n\n2\nn\no\n\n\n\nn\no\n").unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    let (_, shown, _) = run(bin(&d).args(["profile", "show"]));
    assert!(shown.contains("D5200") && shown.contains("\"status\": \"completed\"") && shown.contains("\"track_progress\": true"), "{shown}");
    let (_, p, _) = run(bin(&d).arg("privacy"));
    assert!(p.contains("Telemetry: ACCORDÉ") && p.contains("CloudAi: refusé"), "{p}");
}

#[test]
fn workspace_never_drops_a_tool() {
    let d = tmpdir("ws");
    let total = run(bin(&d).arg("tools")).1.lines().count();
    for mode in ["beginner", "intermediate", "expert"] {
        let (ok, out, _) = run(bin(&d).args(["workspace", mode, "portrait"]));
        assert!(ok);
        assert!(out.contains(&format!("{total} outils accessibles sur {total}")), "{mode}: {out}");
    }
}

#[test]
fn academy_search_finds_lessons_without_accents() {
    let d = tmpdir("search");
    let (_, out, _) = run(bin(&d).args(["academy", "search", "ecretage", "--lang", "fr"]));
    assert!(out.contains("Lesson:highlights"), "{out}");
}

/// Garde-fou de confidentialité : aucune bibliothèque réseau ne doit entrer dans le graphe de dépendances
/// sans décision explicite (ajouter ici l'exception, documentée dans ARCHITECTURE.md et SECURITY.md).
#[test]
fn no_networking_crate_in_dependency_graph() {
    let lock = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../Cargo.lock")).unwrap();
    let banned = ["reqwest", "hyper", "ureq", "curl", "isahc", "surf", "attohttpc", "openssl", "native-tls", "rustls",
                  "tungstenite", "tokio", "mio", "socket2", "h2", "tiny_http"];
    for line in lock.lines().filter(|l| l.starts_with("name = ")) {
        let name = line.trim_start_matches("name = ").trim_matches('"');
        assert!(!banned.contains(&name), "dépendance réseau non autorisée : {name}");
    }
}

#[test]
fn export_watermark_checkbox_and_dual_versions() {
    let d = tmpdir("export");
    let photo = d.join("séance.png");
    run(bin(&d).args(["academy", "training", "ex_exposure", photo.to_str().unwrap()]));
    let original = sha(&photo);
    let out = d.join("sortie");
    // case décochée : aucun filigrane, fichier identique à la source ré-encodée
    let (ok, o, e) = run(bin(&d).args(["export", photo.to_str().unwrap(), out.to_str().unwrap(), "--brand", "Norélias Photography"]));
    assert!(ok, "{e}");
    assert!(o.contains("séance.png"));
    let clean = image::open(out.join("séance.png")).unwrap().to_rgb8();
    assert_eq!(clean.as_raw(), image::open(&photo).unwrap().to_rgb8().as_raw());
    // case cochée + version sans filigrane
    let out2 = d.join("sortie2");
    let (ok, _, e) = run(bin(&d).args(["export", photo.to_str().unwrap(), out2.to_str().unwrap(), "--wm", "--dual", "--brand", "Norélias Photography", "--copyright", "© 2026 Norélias"]));
    assert!(ok, "{e}");
    let marked = image::open(out2.join("séance_filigrane.png")).unwrap().to_rgb8();
    let unmarked = image::open(out2.join("séance_sans-filigrane.png")).unwrap().to_rgb8();
    assert_ne!(marked.as_raw(), unmarked.as_raw());
    assert_eq!(unmarked.as_raw(), clean.as_raw());
    assert_eq!(sha(&photo), original, "original intact");
    // refus d'écraser
    let (ok, _, e) = run(bin(&d).args(["export", photo.to_str().unwrap(), out2.to_str().unwrap(), "--wm", "--dual", "--brand", "X"]));
    assert!(!ok && e.contains("existe déjà"), "{e}");
}

#[test]
fn help_pages_and_honest_unknown_tools() {
    let d = tmpdir("help");
    let (ok, out, _) = run(bin(&d).args(["help", "show", "exposure", "--lang", "fr"]));
    assert!(ok && out.contains("## Erreurs fréquentes") && out.contains("Aucun tutoriel vidéo disponible"), "{out}");
    let (ok, _, err) = run(bin(&d).args(["help", "show", "tone_curve"]));
    assert!(!ok && err.contains("pas encore intégré"), "pas de fausse aide pour un outil prévu : {err}");
    let dir = d.join("docs");
    let (ok, out, _) = run(bin(&d).args(["help", "export", dir.to_str().unwrap(), "--lang", "en"]));
    assert!(ok && out.starts_with("12 "), "{out}");
    assert!(dir.join("help/crop.md").exists());
}

#[test]
fn assistant_needs_permission_and_never_applies() {
    let d = tmpdir("assistant");
    let photo = d.join("a.png");
    run(bin(&d).args(["academy", "training", "ex_exposure", photo.to_str().unwrap()]));
    let (_, out, _) = run(bin(&d).args(["assistant", "ma photo est trop sombre", photo.to_str().unwrap(), "--lang", "fr"]));
    assert!(out.contains("permission") && !out.contains("Proposition"), "{out}");
    let (_, out, _) = run(bin(&d).args(["assistant", "ma photo est trop sombre", photo.to_str().unwrap(), "--allow-photo", "--lang", "fr"]));
    assert!(out.contains("Proposition") && out.contains("non appliquée"), "{out}");
    let (_, out, _) = run(bin(&d).args(["assistant", "supprime l'objet avec l'IA", "--lang", "fr"]));
    assert!(out.contains("prévue mais pas encore disponible"), "{out}");
}
