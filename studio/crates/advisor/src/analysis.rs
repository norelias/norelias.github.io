use crate::{Finding, Kind, Severity};
use norelias_core::color::{linear_to_srgb, luma};
use norelias_core::{Image, Op};

#[derive(Clone, Debug)]
pub struct Thresholds {
    pub target_median: f32,
    pub exposure_warn_ev: f32,
    pub exposure_strong_ev: f32,
    pub clip_level: f32,
    pub clip_fraction_warn: f32,
    pub clip_fraction_strong: f32,
    pub wb_warn_stops: f32,
    pub wb_strong_stops: f32,
    pub tint_warn_stops: f32,
    pub noise_warn: f32,
    pub noise_strong: f32,
    pub soft_warn: f32,
    pub soft_strong: f32,
    pub motion_ratio: f32,
    pub tilt_min_deg: f32,
    /// Taille de travail pour exposition/netteté/composition.
    pub work_side: usize,
}
impl Default for Thresholds {
    fn default() -> Self {
        Self {
            target_median: 0.18, exposure_warn_ev: 0.6, exposure_strong_ev: 1.3,
            clip_level: 0.995, clip_fraction_warn: 0.01, clip_fraction_strong: 0.05,
            wb_warn_stops: 0.25, wb_strong_stops: 0.5, tint_warn_stops: 0.2,
            noise_warn: 0.015, noise_strong: 0.035,
            soft_warn: 1.22, soft_strong: 1.15, motion_ratio: 0.35,
            tilt_min_deg: 0.6, work_side: 1024,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Metrics {
    pub median_linear: f32,
    pub exposure_ev: f32,
    pub clipped_high: f32,
    pub clipped_low: f32,
    pub neutral_fraction: f32,
    /// log2(R/B) sur les pixels quasi neutres (0 = neutre).
    pub wb_stops: f32,
    /// log2(G / √(R·B)) sur les pixels quasi neutres.
    pub tint_stops: f32,
    pub noise_sigma: f32,
    /// 99,8e centile du gradient de luminance encodée (force des contours les plus marqués).
    pub edge_p99: f32,
    /// Rapport de force des contours entre l'échelle fine et l'échelle 2× : ≈1,35+ net, <1,2 flou.
    pub sharp_ratio: f32,
    /// min/max de l'énergie des gradients horizontaux et verticaux (1 = isotrope).
    pub motion_ratio: f32,
    pub tilt_deg: Option<f32>,
    pub centroid: (f32, f32),
    pub spread: (f32, f32),
}

fn enc_luma(img: &Image) -> Vec<f32> {
    img.luma_plane().into_iter().map(linear_to_srgb).collect()
}

fn box3(p: &[f32], w: usize, h: usize) -> Vec<f32> {
    let mut o = vec![0.0; p.len()];
    for y in 0..h { for x in 0..w {
        let mut a = 0.0;
        for dy in -1i32..=1 { for dx in -1i32..=1 {
            a += p[(y as i32 + dy).clamp(0, h as i32 - 1) as usize * w + (x as i32 + dx).clamp(0, w as i32 - 1) as usize];
        }}
        o[y * w + x] = a / 9.0;
    }}
    o
}

/// Gradients par différences centrées (valeurs sur l'échelle encodée 0..1 par pixel).
fn gradients(p: &[f32], w: usize, h: usize) -> (Vec<f32>, Vec<f32>) {
    let mut gx = vec![0.0; p.len()];
    let mut gy = vec![0.0; p.len()];
    for y in 1..h.saturating_sub(1) { for x in 1..w.saturating_sub(1) {
        gx[y * w + x] = (p[y * w + x + 1] - p[y * w + x - 1]) / 2.0;
        gy[y * w + x] = (p[(y + 1) * w + x] - p[(y - 1) * w + x]) / 2.0;
    }}
    (gx, gy)
}

fn percentile(v: &mut [f32], q: f32) -> f32 {
    if v.is_empty() { return 0.0; }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[((v.len() - 1) as f32 * q) as usize]
}

/// Estimateur de bruit d'Immerkær restreint aux zones plates (σ sur l'échelle encodée).
fn noise_sigma(p: &[f32], w: usize, h: usize) -> f32 {
    if w < 8 || h < 8 { return 0.0; }
    let (gx, gy) = gradients(&box3(p, w, h), w, h);
    let (mut sum, mut n) = (0.0f64, 0usize);
    for y in 1..h - 1 { for x in 1..w - 1 {
        let i = y * w + x;
        if gx[i].hypot(gy[i]) > 0.02 { continue; }
        let at = |dx: i32, dy: i32| p[(y as i32 + dy) as usize * w + (x as i32 + dx) as usize];
        let r = at(-1, -1) - 2.0 * at(0, -1) + at(1, -1) - 2.0 * at(-1, 0) + 4.0 * at(0, 0) - 2.0 * at(1, 0)
            + at(-1, 1) - 2.0 * at(0, 1) + at(1, 1);
        sum += r.abs() as f64;
        n += 1;
    }}
    if n < 64 { return 0.0; }
    ((std::f64::consts::PI / 2.0).sqrt() * sum / n as f64 / 6.0) as f32
}

pub fn analyze(img: &Image, th: &Thresholds) -> Metrics {
    let mut m = Metrics { motion_ratio: 1.0, ..Default::default() };
    let work = img.downscale_to(th.work_side);
    let (w, h) = (work.width, work.height);
    let n = (w * h) as f32;

    // Exposition + écrêtage
    let ys = work.luma_plane();
    let mut sorted = ys.clone();
    m.median_linear = percentile(&mut sorted, 0.5).max(1e-5);
    m.exposure_ev = (th.target_median / m.median_linear).log2();
    m.clipped_high = work.data.chunks_exact(3).filter(|p| p.iter().any(|&v| v >= th.clip_level)).count() as f32 / n;
    m.clipped_low = ys.iter().filter(|&&y| y <= 0.0005).count() as f32 / n;

    // Pixels quasi neutres → balance des blancs / teinte
    let (mut s, mut c) = ([0.0f64; 3], 0usize);
    for p in work.data.chunks_exact(3) {
        let y = luma(p[0], p[1], p[2]);
        if !(0.03..=0.7).contains(&y) || p.iter().any(|&v| v >= th.clip_level) { continue; }
        let (mx, mn) = (p.iter().cloned().fold(0.0, f32::max), p.iter().cloned().fold(f32::MAX, f32::min));
        if mn > 1e-4 && mx / mn < 2.2 {
            for k in 0..3 { s[k] += p[k] as f64; }
            c += 1;
        }
    }
    m.neutral_fraction = c as f32 / n;
    if m.neutral_fraction >= 0.02 {
        let (r, g, b) = (s[0] as f32, s[1] as f32, s[2] as f32);
        m.wb_stops = (r / b).log2();
        m.tint_stops = (g / (r * b).sqrt()).log2();
    }

    // Netteté et flou directionnel (sur l'image de travail, préfiltrée contre le bruit)
    let enc = enc_luma(&work);
    let (gx, gy) = gradients(&box3(&enc, w, h), w, h);
    let mut mags: Vec<f32> = gx.iter().zip(&gy).map(|(a, b)| a.hypot(*b)).collect();
    let (ex, ey): (f64, f64) = (gx.iter().map(|v| (*v as f64).powi(2)).sum(), gy.iter().map(|v| (*v as f64).powi(2)).sum());
    if ex.max(ey) > 1e-9 { m.motion_ratio = (ex.min(ey) / ex.max(ey)) as f32; }
    let total_e: f32 = mags.iter().sum();
    // Centre de gravité et dispersion de l'énergie des contours (sujet concentré ?)
    if total_e > 1e-6 {
        let (mut cx, mut cy) = (0.0f64, 0.0f64);
        for y in 0..h { for x in 0..w { let e = mags[y * w + x] as f64; cx += e * x as f64; cy += e * y as f64; } }
        let (cx, cy) = (cx / total_e as f64, cy / total_e as f64);
        let (mut vx, mut vy) = (0.0f64, 0.0f64);
        for y in 0..h { for x in 0..w { let e = mags[y * w + x] as f64; vx += e * (x as f64 - cx).powi(2); vy += e * (y as f64 - cy).powi(2); } }
        m.centroid = ((cx / w as f64) as f32, (cy / h as f64) as f32);
        m.spread = (((vx / total_e as f64).sqrt() / w as f64) as f32, ((vy / total_e as f64).sqrt() / h as f64) as f32);
    }
    // Horizon : orientation dominante des contours quasi horizontaux (±15°)
    m.tilt_deg = tilt(&gx, &gy, w, h);
    m.edge_p99 = percentile(&mut mags, 0.998);
    m.sharp_ratio = 99.0; // pas de contours : non évaluable
    if m.edge_p99 > 0.015 {
        let b2 = box3(&box3(&enc, w, h), w, h);
        let mut g2: Vec<f32> = (0..w * h).map(|i| { let (x, y) = (i % w, i / w);
            if x < 2 || y < 2 || x + 2 >= w || y + 2 >= h { 0.0 } else {
                ((b2[y * w + x + 2] - b2[y * w + x - 2]) / 4.0).hypot((b2[(y + 2) * w + x] - b2[(y - 2) * w + x]) / 4.0) } }).collect();
        m.sharp_ratio = m.edge_p99 / percentile(&mut g2, 0.998).max(1e-6);
    }

    // Bruit : sur un recadrage central en pleine résolution
    let side = 1024;
    let crop = img.crop_px(img.width.saturating_sub(side) / 2, img.height.saturating_sub(side) / 2, side, side);
    m.noise_sigma = noise_sigma(&enc_luma(&crop), crop.width, crop.height);
    m
}

fn box_sep(p: &[f32], w: usize, h: usize, r: usize) -> Vec<f32> {
    let mut t = vec![0.0f32; p.len()];
    let n = (2 * r + 1) as f32;
    for y in 0..h { for x in 0..w {
        let mut a = 0.0;
        for d in 0..=2 * r { a += p[y * w + (x + d).saturating_sub(r).min(w - 1)]; }
        t[y * w + x] = a / n;
    }}
    let mut o = vec![0.0f32; p.len()];
    for y in 0..h { for x in 0..w {
        let mut a = 0.0;
        for d in 0..=2 * r { a += t[((y + d).saturating_sub(r)).min(h - 1) * w + x]; }
        o[y * w + x] = a / n;
    }}
    o
}

/// Inclinaison de l'horizon : orientation des contours quasi horizontaux par tenseur de structure lissé
/// (précision sous-pixel, contrairement à l'angle local du gradient qui est biaisé vers 0° par la quantification).
fn tilt(gx: &[f32], gy: &[f32], w: usize, h: usize) -> Option<f32> {
    if w < 64 || h < 32 { return None; }
    let jxx = box_sep(&gx.iter().map(|v| v * v).collect::<Vec<_>>(), w, h, 4);
    let jyy = box_sep(&gy.iter().map(|v| v * v).collect::<Vec<_>>(), w, h, 4);
    let jxy = box_sep(&gx.iter().zip(gy).map(|(a, b)| a * b).collect::<Vec<_>>(), w, h, 4);
    const STRIPS: usize = 3;
    let (mut sxy, mut syy) = ([0.0f64; STRIPS], [0.0f64; STRIPS]);
    let (mut near_h, mut total) = (0.0f64, 0.0f64);
    for y in 0..h { for x in 0..w {
        let i = y * w + x;
        let e = jxx[i] + jyy[i];
        if e < 0.0009 { continue; }
        if (jxx[i] - jyy[i]).hypot(2.0 * jxy[i]) / e < 0.6 { continue; } // pas un contour net et orienté
        total += e as f64;
        let phi = 0.5 * (2.0 * jxy[i]).atan2(jxx[i] - jyy[i]).to_degrees();
        let t = if phi > 0.0 { phi - 90.0 } else { phi + 90.0 };
        if t.abs() <= 15.0 {
            near_h += e as f64;
            let s = (x * STRIPS / w).min(STRIPS - 1);
            sxy[s] += (gx[i] * gy[i]) as f64;
            syy[s] += (gy[i] * gy[i]) as f64;
        }
    }}
    if total <= 0.0 || near_h / total < 0.25 { return None; }
    // Pente moyenne par bande verticale : un horizon droit penché donne la même pente partout.
    let est: Vec<f64> = (0..STRIPS).filter(|&s| syy[s] > 1e-6).map(|s| -(sxy[s] / syy[s]).atan().to_degrees()).collect();
    if est.len() < 2 { return None; }
    let (mn, mx) = (est.iter().cloned().fold(f64::MAX, f64::min), est.iter().cloned().fold(f64::MIN, f64::max));
    if mx - mn > 1.0 { return None; }
    let (sxy_t, syy_t): (f64, f64) = (sxy.iter().sum(), syy.iter().sum());
    Some((-(sxy_t / syy_t).atan().to_degrees()) as f32)
}

pub(crate) fn findings(m: &Metrics, th: &Thresholds) -> Vec<Finding> {
    let mut out = Vec::new();
    let sev = |v: f32, warn: f32, strong: f32| if v >= strong { Severity::Strong } else if v >= warn { Severity::Warning } else { Severity::Info };

    if m.exposure_ev.abs() >= th.exposure_warn_ev {
        let ev = (m.exposure_ev * 0.9).clamp(-3.0, 3.0);
        out.push(Finding { kind: Kind::Exposure, severity: sev(m.exposure_ev.abs(), th.exposure_warn_ev, th.exposure_strong_ev),
            value: m.exposure_ev, ops: vec![Op::Exposure { ev }], auto_safe: true });
    }
    if m.clipped_high >= th.clip_fraction_warn {
        let amount = (0.5 + m.clipped_high * 5.0).clamp(0.5, 1.0);
        out.push(Finding { kind: Kind::HighlightClipping, severity: sev(m.clipped_high, th.clip_fraction_warn, th.clip_fraction_strong),
            value: m.clipped_high, ops: vec![Op::Highlights { amount }], auto_safe: true });
    }
    if m.neutral_fraction >= 0.02 {
        if m.wb_stops.abs() >= th.wb_warn_stops {
            let h = m.wb_stops / 2.0;
            out.push(Finding { kind: Kind::WhiteBalance, severity: sev(m.wb_stops.abs(), th.wb_warn_stops, th.wb_strong_stops),
                value: m.wb_stops, ops: vec![Op::WhiteBalance { r: 2f32.powf(-h), g: 1.0, b: 2f32.powf(h) }], auto_safe: true });
        }
        if m.tint_stops.abs() >= th.tint_warn_stops {
            out.push(Finding { kind: Kind::ColorCast, severity: sev(m.tint_stops.abs(), th.tint_warn_stops, th.tint_warn_stops * 2.0),
                value: m.tint_stops, ops: vec![Op::WhiteBalance { r: 1.0, g: 2f32.powf(-m.tint_stops), b: 1.0 }], auto_safe: true });
        }
    }
    if m.noise_sigma >= th.noise_warn {
        let strength = (m.noise_sigma / 0.06).clamp(0.2, 1.0);
        out.push(Finding { kind: Kind::Noise, severity: sev(m.noise_sigma, th.noise_warn, th.noise_strong),
            value: m.noise_sigma, ops: vec![Op::Denoise { strength }], auto_safe: true });
    }
    let motion = m.motion_ratio < th.motion_ratio && m.edge_p99 > 0.0;
    if m.sharp_ratio < th.soft_warn && !motion {
        let sev = if m.sharp_ratio < th.soft_strong { Severity::Strong } else { Severity::Warning };
        // On ne propose un renforcement que s'il n'amplifiera pas un bruit déjà marqué.
        let ops = if m.noise_sigma >= th.noise_strong { vec![] } else { vec![Op::Sharpen { amount: 0.8 }] };
        out.push(Finding { kind: Kind::Sharpness, severity: sev, value: m.sharp_ratio, ops, auto_safe: false });
    }
    if motion {
        // Aucune déconvolution n'est disponible : on n'invente pas de correction.
        out.push(Finding { kind: Kind::MotionBlur, severity: Severity::Warning, value: m.motion_ratio, ops: vec![], auto_safe: false });
    }
    if let Some(t) = m.tilt_deg {
        if t.abs() >= th.tilt_min_deg {
            out.push(Finding { kind: Kind::HorizonTilt, severity: if t.abs() > 2.0 { Severity::Strong } else { Severity::Warning },
                value: t, ops: vec![Op::Rotate { degrees: -t }], auto_safe: false });
        }
    }
    if m.spread.0 < 0.15 && m.spread.1 < 0.15 && (m.centroid.0 - 0.5).abs() < 0.06 && (m.centroid.1 - 0.5).abs() < 0.06 && m.edge_p99 > 0.0 {
        let k = 0.85f32;
        let (cx, cy) = m.centroid;
        let t = |c: f32| if c < 0.5 { 1.0 / 3.0 } else { 2.0 / 3.0 };
        let x0 = (cx - t(cx) * k).clamp(0.0, 1.0 - k);
        let y0 = (cy - t(cy) * k).clamp(0.0, 1.0 - k);
        out.push(Finding { kind: Kind::Thirds, severity: Severity::Info, value: cx,
            ops: vec![Op::Crop { x: x0, y: y0, w: k, h: k }], auto_safe: false });
    }
    out.sort_by(|a, b| b.severity.cmp(&a.severity).then(a.kind.cmp(&b.kind)));
    out
}
