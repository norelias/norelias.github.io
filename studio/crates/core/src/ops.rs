//! Opérations de développement non destructives : une `Op` est une donnée sérialisable
//! appliquée à une image source qui n'est jamais modifiée.

use crate::color::{linear_to_srgb, luma, srgb_to_linear};
use crate::image::Image;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Op {
    /// Exposition en IL (EV), appliquée en linéaire.
    Exposure { ev: f32 },
    /// Contraste, −1..1, pivot sur le gris moyen 18 %.
    Contrast { amount: f32 },
    /// Gains par canal (linéaire).
    WhiteBalance { r: f32, g: f32, b: f32 },
    /// Compression douce des hautes lumières, 0..1.
    Highlights { amount: f32 },
    /// Éclaircissement des ombres, 0..1.
    Shadows { amount: f32 },
    /// Facteur de saturation (1 = inchangé).
    Saturation { factor: f32 },
    /// Réduction du bruit préservant les contours, 0..1.
    Denoise { strength: f32 },
    /// Masque flou, 0..2.
    Sharpen { amount: f32 },
    /// Rotation en degrés (sens horaire), recadrage automatique sans bords vides.
    Rotate { degrees: f32 },
    /// Recadrage, coordonnées normalisées 0..1.
    Crop { x: f32, y: f32, w: f32, h: f32 },
}

impl Op {
    /// Identifiant stable, partagé avec le registre d'outils de l'interface.
    pub fn name(&self) -> &'static str {
        match self {
            Op::Exposure { .. } => "exposure",
            Op::Contrast { .. } => "contrast",
            Op::WhiteBalance { .. } => "white_balance",
            Op::Highlights { .. } => "highlights",
            Op::Shadows { .. } => "shadows",
            Op::Saturation { .. } => "saturation",
            Op::Denoise { .. } => "denoise",
            Op::Sharpen { .. } => "sharpen",
            Op::Rotate { .. } => "rotate",
            Op::Crop { .. } => "crop",
        }
    }
    /// Paramètres (curseurs) de chaque opération, dans l'ordre de `NAMES`. Un test vérifie qu'ils
    /// correspondent exactement aux champs sérialisés : l'aide contextuelle s'appuie dessus.
    pub const PARAMS: [(&'static str, &'static [&'static str]); 10] = [
        ("exposure", &["ev"]), ("contrast", &["amount"]), ("white_balance", &["r", "g", "b"]), ("highlights", &["amount"]),
        ("shadows", &["amount"]), ("saturation", &["factor"]), ("denoise", &["strength"]), ("sharpen", &["amount"]),
        ("rotate", &["degrees"]), ("crop", &["x", "y", "w", "h"]),
    ];
    /// Tous les identifiants d'opérations du moteur.
    pub const NAMES: [&'static str; 10] = [
        "exposure", "contrast", "white_balance", "highlights", "shadows",
        "saturation", "denoise", "sharpen", "rotate", "crop",
    ];

    /// Applique l'opération ; `src` n'est jamais modifiée.
    pub fn apply(&self, src: &Image) -> Image {
        match *self {
            Op::Exposure { ev } => map(src, |p| p.map(|v| v * 2f32.powf(ev))),
            Op::Contrast { amount } => {
                let k = (1.0 + amount).max(0.05);
                map(src, |p| p.map(|v| 0.18 * (v.max(1e-6) / 0.18).powf(k)))
            }
            Op::WhiteBalance { r, g, b } => map(src, |p| [p[0] * r, p[1] * g, p[2] * b]),
            Op::Highlights { amount } => {
                let a = amount.clamp(0.0, 1.0);
                let knee = 0.7f32;
                map(src, |p| {
                    p.map(|v| {
                        if v <= knee { v } else {
                            let c = knee + (1.0 - knee) * ((v - knee) / (1.0 - knee)).tanh();
                            v + (c - v) * a
                        }
                    })
                })
            }
            Op::Shadows { amount } => {
                let a = amount.clamp(0.0, 1.0);
                map(src, |p| {
                    p.map(|v| {
                        let w = (1.0 - (v / 0.2).clamp(0.0, 1.0)).powi(2);
                        v * (1.0 + a * 3.0 * w)
                    })
                })
            }
            Op::Saturation { factor } => map(src, |p| {
                let y = luma(p[0], p[1], p[2]);
                p.map(|v| y + (v - y) * factor)
            }),
            Op::Denoise { strength } => denoise(src, strength.clamp(0.0, 1.0)),
            Op::Sharpen { amount } => sharpen(src, amount.clamp(0.0, 2.0)),
            Op::Rotate { degrees } => rotate(src, degrees),
            Op::Crop { x, y, w, h } => {
                let (fw, fh) = (src.width as f32, src.height as f32);
                src.crop_px((x.clamp(0.0, 1.0) * fw) as usize, (y.clamp(0.0, 1.0) * fh) as usize,
                            (w.clamp(0.0, 1.0) * fw).round() as usize, (h.clamp(0.0, 1.0) * fh).round() as usize)
            }
        }
    }
}

/// Applique une pile d'opérations dans l'ordre. La source est conservée.
pub fn apply_stack(src: &Image, ops: &[Op]) -> Image {
    ops.iter().fold(src.clone(), |img, op| op.apply(&img))
}

fn map(src: &Image, f: impl Fn([f32; 3]) -> [f32; 3]) -> Image {
    let mut out = src.clone();
    for p in out.data.chunks_exact_mut(3) {
        let q = f([p[0], p[1], p[2]]);
        p.copy_from_slice(&q);
    }
    out
}

fn to_enc(src: &Image) -> Image {
    let mut o = src.clone();
    o.data.iter_mut().for_each(|v| *v = linear_to_srgb(*v));
    o
}
fn from_enc(mut o: Image) -> Image {
    o.data.iter_mut().for_each(|v| *v = srgb_to_linear(*v));
    o
}

/// Filtre bilatéral 3×3 itéré, en domaine encodé (le bruit y est à peu près homogène).
fn denoise(src: &Image, strength: f32) -> Image {
    if strength <= 0.0 {
        return src.clone();
    }
    let passes = 1 + (strength * 3.0) as usize;
    let sigma_r = 0.02 + strength * 0.12;
    let mut cur = to_enc(src);
    for _ in 0..passes {
        let prev = cur.clone();
        for y in 0..prev.height {
            for x in 0..prev.width {
                let c = prev.get(x, y);
                let yc = luma(c[0], c[1], c[2]);
                let (mut acc, mut wsum) = ([0.0f32; 3], 0.0f32);
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        let (nx, ny) = ((x as i32 + dx).clamp(0, prev.width as i32 - 1) as usize,
                                        (y as i32 + dy).clamp(0, prev.height as i32 - 1) as usize);
                        let n = prev.get(nx, ny);
                        let dl = luma(n[0], n[1], n[2]) - yc;
                        let ws = match dx.abs() + dy.abs() { 0 => 1.0, 1 => 0.6, _ => 0.36 };
                        let w = ws * (-(dl * dl) / (2.0 * sigma_r * sigma_r)).exp();
                        for k in 0..3 { acc[k] += n[k] * w; }
                        wsum += w;
                    }
                }
                cur.set(x, y, [acc[0] / wsum, acc[1] / wsum, acc[2] / wsum]);
            }
        }
    }
    from_enc(cur)
}

fn sharpen(src: &Image, amount: f32) -> Image {
    if amount <= 0.0 {
        return src.clone();
    }
    let e = to_enc(src);
    let mut out = e.clone();
    const K: [[f32; 3]; 3] = [[1.0, 2.0, 1.0], [2.0, 4.0, 2.0], [1.0, 2.0, 1.0]];
    for y in 0..e.height {
        for x in 0..e.width {
            let mut blur = [0.0f32; 3];
            for (j, row) in K.iter().enumerate() {
                for (i, w) in row.iter().enumerate() {
                    let nx = (x as i32 + i as i32 - 1).clamp(0, e.width as i32 - 1) as usize;
                    let ny = (y as i32 + j as i32 - 1).clamp(0, e.height as i32 - 1) as usize;
                    let n = e.get(nx, ny);
                    for k in 0..3 { blur[k] += n[k] * w / 16.0; }
                }
            }
            let c = e.get(x, y);
            out.set(x, y, [0, 1, 2].map(|k| c[k] + amount * (c[k] - blur[k])));
        }
    }
    from_enc(out)
}

/// Rotation horaire de `degrees` avec zoom minimal supprimant les bords vides.
fn rotate(src: &Image, degrees: f32) -> Image {
    let t = degrees.to_radians();
    let (s, c) = t.sin_cos();
    let (w, h) = (src.width as f32, src.height as f32);
    let ratio = w.max(h) / w.min(h);
    let zoom = 1.0 / (t.cos().abs() + t.sin().abs() * ratio);
    let (cx, cy) = ((w - 1.0) / 2.0, (h - 1.0) / 2.0);
    Image::from_fn(src.width, src.height, |x, y| {
        let (px, py) = ((x as f32 - cx) * zoom, (y as f32 - cy) * zoom);
        // source = R(−θ)·p
        let (sx, sy) = (px * c + py * s, -px * s + py * c);
        src.sample(sx + cx, sy + cy)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn grey(v: f32) -> Image { Image::from_fn(8, 8, |_, _| [v, v, v]) }

    #[test]
    fn exposure_plus_one_ev_doubles_linear_value() {
        let o = Op::Exposure { ev: 1.0 }.apply(&grey(0.1));
        assert!((o.get(0, 0)[0] - 0.2).abs() < 1e-6);
    }
    #[test]
    fn source_is_never_modified() {
        let src = grey(0.3);
        let copy = src.clone();
        let _ = apply_stack(&src, &[Op::Exposure { ev: 2.0 }, Op::Denoise { strength: 1.0 }, Op::Sharpen { amount: 1.0 }]);
        assert_eq!(src, copy);
    }
    #[test]
    fn identity_parameters_change_nothing() {
        let src = Image::from_fn(9, 7, |x, y| [x as f32 / 9.0, y as f32 / 7.0, 0.3]);
        for op in [Op::Exposure { ev: 0.0 }, Op::Saturation { factor: 1.0 }, Op::Contrast { amount: 0.0 },
                   Op::Highlights { amount: 0.0 }, Op::Shadows { amount: 0.0 }, Op::Sharpen { amount: 0.0 },
                   Op::Denoise { strength: 0.0 }, Op::WhiteBalance { r: 1.0, g: 1.0, b: 1.0 }] {
            let o = op.apply(&src);
            let d = o.data.iter().zip(&src.data).map(|(a, b)| (a - b).abs()).fold(0.0, f32::max);
            assert!(d < 1e-4, "{op:?} a modifié l'image de {d}");
        }
    }
    #[test]
    fn contrast_keeps_mid_grey_fixed() {
        let o = Op::Contrast { amount: 0.5 }.apply(&grey(0.18));
        assert!((o.get(0, 0)[0] - 0.18).abs() < 1e-5);
    }
    #[test]
    fn zero_saturation_gives_neutral() {
        let src = Image::from_fn(2, 2, |_, _| [0.5, 0.2, 0.1]);
        let p = Op::Saturation { factor: 0.0 }.apply(&src).get(0, 0);
        assert!((p[0] - p[1]).abs() < 1e-6 && (p[1] - p[2]).abs() < 1e-6);
    }
    #[test]
    fn highlights_compress_only_above_knee() {
        let lo = Op::Highlights { amount: 1.0 }.apply(&grey(0.5)).get(0, 0)[0];
        let hi = Op::Highlights { amount: 1.0 }.apply(&grey(1.5)).get(0, 0)[0];
        assert!((lo - 0.5).abs() < 1e-6);
        assert!(hi < 1.0 && hi > 0.7);
    }
    #[test]
    fn denoise_reduces_variance_of_noise_but_keeps_edge() {
        let mut seed = 7u32;
        let mut rnd = || { seed = seed.wrapping_mul(1664525).wrapping_add(1013904223); (seed >> 8) as f32 / 16777216.0 - 0.5 };
        let src = Image::from_fn(32, 32, |x, _| { let b = if x < 16 { 0.1 } else { 0.6 }; let n = rnd() * 0.03; [b + n, b + n, b + n] });
        let out = Op::Denoise { strength: 0.8 }.apply(&src);
        let var = |i: &Image, x0: usize, x1: usize| {
            let v: Vec<f32> = (0..32).flat_map(|y| (x0..x1).map(move |x| (x, y))).map(|(x, y)| i.get(x, y)[0]).collect();
            let m = v.iter().sum::<f32>() / v.len() as f32;
            v.iter().map(|a| (a - m).powi(2)).sum::<f32>() / v.len() as f32
        };
        assert!(var(&out, 2, 12) < var(&src, 2, 12) * 0.5);
        assert!(out.get(20, 5)[0] - out.get(10, 5)[0] > 0.35, "le contour doit rester net");
    }
    #[test]
    fn sharpen_increases_edge_contrast() {
        let src = Image::from_fn(16, 4, |x, _| { let v = if x < 8 { 0.2 } else { 0.4 }; [v, v, v] });
        let out = Op::Sharpen { amount: 1.0 }.apply(&src);
        assert!(out.get(8, 1)[0] > src.get(8, 1)[0]);
    }
    #[test]
    fn rotate_zero_is_identity_and_keeps_size() {
        let src = Image::from_fn(20, 10, |x, y| [x as f32 / 20.0, y as f32 / 10.0, 0.0]);
        let o = Op::Rotate { degrees: 0.0 }.apply(&src);
        assert_eq!((o.width, o.height), (20, 10));
        let d = o.data.iter().zip(&src.data).map(|(a, b)| (a - b).abs()).fold(0.0, f32::max);
        assert!(d < 1e-5);
    }
    #[test]
    fn crop_dimensions() {
        let o = Op::Crop { x: 0.25, y: 0.0, w: 0.5, h: 1.0 }.apply(&Image::new(40, 20));
        assert_eq!((o.width, o.height), (20, 20));
    }
    #[test]
    fn declared_params_match_serialised_fields() {
        let ops = [Op::Exposure { ev: 1.0 }, Op::Contrast { amount: 0.1 }, Op::WhiteBalance { r: 1.0, g: 1.0, b: 1.0 }, Op::Highlights { amount: 0.1 },
            Op::Shadows { amount: 0.1 }, Op::Saturation { factor: 1.0 }, Op::Denoise { strength: 0.1 }, Op::Sharpen { amount: 0.1 }, Op::Rotate { degrees: 1.0 },
            Op::Crop { x: 0.0, y: 0.0, w: 1.0, h: 1.0 }];
        for (op, (name, params)) in ops.iter().zip(Op::PARAMS) {
            assert_eq!(op.name(), name);
            let v = serde_json::to_value(op).unwrap();
            let mut keys: Vec<&str> = v.as_object().unwrap().keys().map(String::as_str).filter(|k| *k != "op").collect();
            let mut want = params.to_vec();
            keys.sort(); want.sort();
            assert_eq!(keys, want, "{name}");
        }
    }
    #[test]
    fn names_cover_all_variants_and_serde_roundtrips() {
        let ops = [Op::Exposure { ev: 1.0 }, Op::Contrast { amount: 0.1 }, Op::WhiteBalance { r: 1.0, g: 1.0, b: 1.0 },
            Op::Highlights { amount: 0.1 }, Op::Shadows { amount: 0.1 }, Op::Saturation { factor: 1.0 },
            Op::Denoise { strength: 0.1 }, Op::Sharpen { amount: 0.1 }, Op::Rotate { degrees: 1.0 },
            Op::Crop { x: 0.0, y: 0.0, w: 1.0, h: 1.0 }];
        let names: Vec<_> = ops.iter().map(|o| o.name()).collect();
        assert_eq!(names, Op::NAMES);
        for o in ops {
            let j = serde_json::to_string(&o).unwrap();
            assert_eq!(serde_json::from_str::<Op>(&j).unwrap(), o);
        }
    }
}
