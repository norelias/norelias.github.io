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
    assert_eq!(out.matches("refusé (par défaut)").count(), 5, "{out}");
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
