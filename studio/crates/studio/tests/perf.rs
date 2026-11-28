//! cargo test --release -p norelias-studio --test perf -- --ignored --nocapture
use norelias_studio::watermark::*;
use norelias_core::synth;
use std::time::Instant;

#[test]
#[ignore]
fn watermark_timings() {
    let brand = Brand { studio_name: "Norélias Photography".into(), photographer_name: "x".into(), holder: "".into(), year: 2026, logo: None };
    let full = synth::scene(6000, 4000, 0.0);
    let preview = full.fit_within(1000);
    let simple = Watermark::default();
    let rich = Watermark { shadow: Some(Shadow { dx_pct: 0.3, dy_pct: 0.3, blur_pct: 0.4, color: [0; 3], opacity: 0.6 }), outline: Some(Outline { width_pct: 0.2, color: [0; 3] }), rotation_deg: -12.0, ..Default::default() };
    let tiled = Watermark { tile_gap_pct: Some(8.0), opacity: 0.3, ..Default::default() };
    for (n, wm) in [("simple", &simple), ("ombre+contour+rotation", &rich), ("répété", &tiled)] {
        let t = Instant::now(); let _ = apply(&preview, wm, &brand).unwrap(); let p = t.elapsed();
        let t = Instant::now(); let _ = apply(&full, wm, &brand).unwrap(); let f = t.elapsed();
        eprintln!("{n}: aperçu 1000 px {p:?} · 24 Mpx {f:?}");
    }
    let t = Instant::now(); let _ = full.fit_within(2048); eprintln!("réduction 24 Mpx → 2048 px : {:?}", t.elapsed());
}
