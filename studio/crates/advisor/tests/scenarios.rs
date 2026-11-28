//! Scénarios sur images synthétiques : chaque défaut est injecté puis le conseiller doit
//! le détecter, et la correction proposée doit réellement le résoudre (ré-analyse).

use norelias_advisor::*;
use norelias_core::{synth, Image};

const W: usize = 480;
const H: usize = 320;

fn base() -> Image { synth::scene(W, H, 0.0) }

fn kinds(r: &Report) -> Vec<Kind> { r.findings.iter().map(|f| f.kind).collect() }

#[test]
fn clean_reference_scene_has_no_findings() {
    let r = advise(&base());
    eprintln!("{:#?}", r.metrics);
    assert!(r.findings.is_empty(), "faux positifs : {:?}", kinds(&r));
}

#[test]
fn underexposure_detected_and_fixed() {
    let img = synth::exposure(&base(), -2.0);
    let r = advise(&img);
    let f = r.find(Kind::Exposure).expect("sous-exposition non détectée");
    assert!(f.value > 1.5 && f.severity == Severity::Strong);
    let fixed = apply_selected(&img, &[f]);
    assert!(!advise(&fixed).has(Kind::Exposure), "toujours sous-exposée après correction");
}

#[test]
fn overexposure_and_clipping_on_clipped_jpeg() {
    let img = synth::clip(&synth::exposure(&base(), 2.5));
    let r = advise(&img);
    eprintln!("{:#?}", r.metrics);
    assert!(r.has(Kind::Exposure));
    assert!(r.find(Kind::Exposure).unwrap().value < 0.0);
    assert!(r.has(Kind::HighlightClipping));
}

#[test]
fn warm_cast_detected_and_corrected() {
    let img = synth::tint(&base(), [1.3, 1.0, 0.75]);
    let r = advise(&img);
    let f = r.find(Kind::WhiteBalance).expect("dominante chaude non détectée");
    assert!(f.value > 0.5);
    let fixed = apply_selected(&img, &[f]);
    let after = advise(&fixed);
    assert!(after.metrics.wb_stops.abs() < 0.1, "wb résiduel {}", after.metrics.wb_stops);
    assert!(!after.has(Kind::WhiteBalance));
}

#[test]
fn green_tint_detected_and_corrected() {
    let img = synth::tint(&base(), [1.0, 1.35, 1.0]);
    let r = advise(&img);
    let f = r.find(Kind::ColorCast).expect("dominante verte non détectée");
    let fixed = apply_selected(&img, &[f]);
    assert!(!advise(&fixed).has(Kind::ColorCast));
}

#[test]
fn noise_detected_and_reduced() {
    let img = synth::add_noise(&base(), 0.04, 42);
    let r = advise(&img);
    let f = r.find(Kind::Noise).expect("bruit non détecté");
    let fixed = apply_selected(&img, &[f]);
    let (a, b) = (r.metrics.noise_sigma, advise(&fixed).metrics.noise_sigma);
    eprintln!("bruit {a:.4} -> {b:.4}");
    assert!(b < a * 0.6, "réduction insuffisante : {a} -> {b}");
}

#[test]
fn defocus_detected() {
    let img = synth::defocus(&base(), 6);
    let r = advise(&img);
    eprintln!("{:#?}", r.metrics);
    let f = r.find(Kind::Sharpness).expect("flou non détecté");
    assert!(!f.auto_safe, "le renforcement ne doit pas être automatique");
    assert!(!r.has(Kind::MotionBlur));
}

#[test]
fn motion_blur_detected_without_fake_fix() {
    let img = synth::motion_blur_x(&base(), 21);
    let r = advise(&img);
    eprintln!("{:#?}", r.metrics);
    let f = r.find(Kind::MotionBlur).expect("flou de bougé non détecté");
    assert!(f.ops.is_empty(), "aucune fausse correction ne doit être proposée");
}

#[test]
fn tilted_horizon_detected_and_straightened() {
    for tilt in [-4.0f32, 2.0, 5.0] {
        let img = synth::scene(W, H, tilt);
        let r = advise(&img);
        let f = r.find(Kind::HorizonTilt).unwrap_or_else(|| panic!("inclinaison {tilt}° non détectée : {:?}", r.metrics.tilt_deg));
        assert!((f.value - tilt).abs() < 0.7, "mesuré {} pour {tilt}", f.value);
        let fixed = apply_selected(&img, &[f]);
        let residual = advise(&fixed).metrics.tilt_deg.unwrap_or(0.0);
        assert!(residual.abs() < 0.7, "résiduel {residual}° après correction de {tilt}°");
    }
}

#[test]
fn centered_subject_suggests_crop_but_never_automatically() {
    let img = synth::centered_subject(W, H);
    let r = advise(&img);
    let f = r.find(Kind::Thirds).expect("composition centrée non signalée");
    assert!(!f.auto_safe);
    let (auto, applied) = auto_apply(&img, &r);
    assert!(!applied.contains(&Kind::Thirds));
    assert_eq!((auto.width, auto.height), (W, H), "aucun recadrage automatique");
}

#[test]
fn auto_apply_improves_combined_defects_and_preserves_source() {
    let img = synth::add_noise(&synth::tint(&synth::exposure(&base(), -1.8), [1.25, 1.0, 0.8]), 0.03, 7);
    let snapshot = img.clone();
    let before = advise(&img);
    let (fixed, applied) = auto_apply(&img, &before);
    assert_eq!(img, snapshot, "la source ne doit jamais être modifiée");
    assert!(applied.contains(&Kind::Exposure) && applied.contains(&Kind::WhiteBalance) && applied.contains(&Kind::Noise));
    let after = advise(&fixed);
    let score = |r: &Report| r.findings.iter().map(|f| f.severity as u32 + 1).sum::<u32>();
    eprintln!("avant {:?} après {:?}", kinds(&before), kinds(&after));
    assert!(score(&after) < score(&before) / 2);
}

#[test]
fn preview_is_small_and_leaves_source_untouched() {
    let img = synth::exposure(&base(), -2.0);
    let r = advise(&img);
    let p = r.find(Kind::Exposure).unwrap().preview(&img, 120);
    assert!(p.width <= 120 && p.height <= 120);
    assert_eq!((img.width, img.height), (W, H));
}

#[test]
fn explanations_exist_in_both_languages_for_every_finding() {
    let scenes = [
        synth::exposure(&base(), -2.0), synth::clip(&synth::exposure(&base(), 2.5)),
        synth::tint(&base(), [1.3, 1.0, 0.75]), synth::tint(&base(), [1.0, 1.35, 1.0]),
        synth::add_noise(&base(), 0.04, 1), synth::defocus(&base(), 6), synth::motion_blur_x(&base(), 21),
        synth::scene(W, H, 4.0), synth::centered_subject(W, H),
    ];
    let mut seen = std::collections::HashSet::new();
    for s in &scenes {
        for f in advise(s).findings {
            for l in [Lang::Fr, Lang::En] { assert!(f.title(l).len() > 5 && f.explain(l).len() > 40); }
            seen.insert(f.kind);
        }
    }
    assert_eq!(seen.len(), 9, "tous les types de constats doivent être exercés : {seen:?}");
}
