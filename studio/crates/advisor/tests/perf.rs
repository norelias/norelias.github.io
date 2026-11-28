//! Mesures de performance (lancer avec : cargo test --release -p norelias-advisor --test perf -- --ignored --nocapture).
use norelias_advisor::*;
use norelias_core::{synth, ops::apply_stack, Op};
use std::time::Instant;

#[test]
#[ignore]
fn timings_24_megapixels() {
    let t = Instant::now();
    let img = synth::exposure(&synth::scene(6000, 4000, 0.0), -1.5);
    eprintln!("génération 24 MP : {:?}", t.elapsed());
    let t = Instant::now();
    let r = advise(&img);
    eprintln!("analyse du conseiller (24 MP) : {:?} — {} constats", t.elapsed(), r.findings.len());
    let t = Instant::now();
    let p = r.findings[0].preview(&img, 1024);
    eprintln!("aperçu 1024 px : {:?} ({}×{})", t.elapsed(), p.width, p.height);
    let t = Instant::now();
    let out = apply_stack(&img, &[Op::Exposure { ev: 1.0 }, Op::WhiteBalance { r: 1.1, g: 1.0, b: 0.9 }, Op::Highlights { amount: 0.5 }]);
    eprintln!("exposition+BB+hautes lumières pleine résolution : {:?}", t.elapsed());
    let t = Instant::now();
    let _ = Op::Denoise { strength: 0.6 }.apply(&out);
    eprintln!("réduction du bruit pleine résolution : {:?}", t.elapsed());
    let t = Instant::now();
    let _ = Op::Sharpen { amount: 0.8 }.apply(&out);
    eprintln!("netteté pleine résolution : {:?}", t.elapsed());
    eprintln!("mémoire image f32 RVB : {} Mo", img.data.len() * 4 / 1_000_000);
}
