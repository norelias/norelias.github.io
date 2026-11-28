//! Images d'entraînement et de test **générées par le code** (aucun droit d'auteur tiers :
//! elles sont produites ici et peuvent être redistribuées librement avec le logiciel).
//! Servent à l'Académie (exercices) et à la validation du conseiller.

use crate::color::{linear_to_srgb, srgb_to_linear};
use crate::image::Image;

struct Rect { x0: f32, y0: f32, x1: f32, y1: f32, c: [f32; 3] }

const RECTS: [Rect; 8] = [
    Rect { x0: 0.08, y0: 0.58, x1: 0.20, y1: 0.80, c: [0.35, 0.05, 0.04] },
    Rect { x0: 0.24, y0: 0.40, x1: 0.34, y1: 0.62, c: [0.05, 0.25, 0.06] },
    Rect { x0: 0.38, y0: 0.62, x1: 0.50, y1: 0.86, c: [0.04, 0.08, 0.40] },
    Rect { x0: 0.54, y0: 0.30, x1: 0.64, y1: 0.55, c: [0.50, 0.40, 0.03] },
    Rect { x0: 0.68, y0: 0.56, x1: 0.80, y1: 0.80, c: [0.80, 0.80, 0.80] },
    Rect { x0: 0.84, y0: 0.38, x1: 0.94, y1: 0.60, c: [0.02, 0.02, 0.02] },
    Rect { x0: 0.12, y0: 0.14, x1: 0.24, y1: 0.30, c: [0.42, 0.10, 0.30] },
    Rect { x0: 0.70, y0: 0.10, x1: 0.82, y1: 0.28, c: [0.05, 0.30, 0.30] },
];

fn scene_at(u: f32, v: f32) -> [f32; 3] {
    for r in &RECTS {
        if u >= r.x0 && u < r.x1 && v >= r.y0 && v < r.y1 {
            return r.c;
        }
    }
    let g = if v < 0.6 { 0.34 - 0.13 * (v / 0.6) } else { 0.06 - 0.03 * ((v - 0.6) / 0.4) };
    [g, g, g]
}

/// Scène neutre et bien exposée, avec horizon et objets colorés. `tilt_deg` incline
/// le contenu dans le sens horaire (comme `Op::Rotate`).
pub fn scene(width: usize, height: usize, tilt_deg: f32) -> Image {
    let t = tilt_deg.to_radians();
    let (s, c) = t.sin_cos();
    let (cx, cy) = (width as f32 / 2.0, height as f32 / 2.0);
    Image::from_fn(width, height, |x, y| {
        let mut acc = [0.0f32; 3];
        for (ox, oy) in [(0.25, 0.25), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75)] {
            let (px, py) = (x as f32 + ox - cx, y as f32 + oy - cy);
            // valeur = scène(R(−t)·p)
            let (sx, sy) = (px * c + py * s, -px * s + py * c);
            let p = scene_at((sx + cx) / width as f32, (sy + cy) / height as f32);
            for k in 0..3 { acc[k] += p[k] / 4.0; }
        }
        acc
    })
}

/// Sujet unique centré sur fond uni (composition à améliorer).
pub fn centered_subject(width: usize, height: usize) -> Image {
    Image::from_fn(width, height, |x, y| {
        let (u, v) = (x as f32 / width as f32, y as f32 / height as f32);
        if (0.44..0.56).contains(&u) && (0.38..0.62).contains(&v) { [0.7, 0.6, 0.5] } else { [0.18, 0.18, 0.18] }
    })
}

/// Photographie « JPEG » : valeurs écrêtées à 1.0.
pub fn clip(img: &Image) -> Image {
    let mut o = img.clone();
    o.data.iter_mut().for_each(|v| *v = v.clamp(0.0, 1.0));
    o
}

/// Multiplie les canaux (dominante de couleur).
pub fn tint(img: &Image, gains: [f32; 3]) -> Image {
    let mut o = img.clone();
    for p in o.data.chunks_exact_mut(3) { for k in 0..3 { p[k] *= gains[k]; } }
    o
}

pub fn exposure(img: &Image, ev: f32) -> Image {
    let mut o = img.clone();
    o.data.iter_mut().for_each(|v| *v *= 2f32.powf(ev));
    o
}

/// Bruit gaussien additif en domaine sRGB (sigma sur l'échelle 0..1), graine déterministe.
pub fn add_noise(img: &Image, sigma: f32, seed: u32) -> Image {
    let mut state = seed.wrapping_mul(2654435761).wrapping_add(1);
    let mut next = move || { state = state.wrapping_mul(1664525).wrapping_add(1013904223); ((state >> 8) as f32 + 1.0) / 16777217.0 };
    let mut o = img.clone();
    for v in o.data.iter_mut() {
        let g = (-2.0 * next().ln()).sqrt() * (2.0 * std::f32::consts::PI * next()).cos();
        *v = srgb_to_linear((linear_to_srgb(*v) + g * sigma).clamp(0.0, 1.0));
    }
    o
}

/// Flou de mise au point (moyenne 3×3 répétée).
pub fn defocus(img: &Image, passes: usize) -> Image {
    let mut cur = img.clone();
    for _ in 0..passes {
        let prev = cur.clone();
        for y in 0..prev.height {
            for x in 0..prev.width {
                let mut a = [0.0f32; 3];
                for dy in -1i32..=1 { for dx in -1i32..=1 {
                    let p = prev.get((x as i32 + dx).clamp(0, prev.width as i32 - 1) as usize,
                                     (y as i32 + dy).clamp(0, prev.height as i32 - 1) as usize);
                    for k in 0..3 { a[k] += p[k] / 9.0; }
                }}
                cur.set(x, y, a);
            }
        }
    }
    cur
}

/// Flou de bougé horizontal de `len` pixels.
pub fn motion_blur_x(img: &Image, len: usize) -> Image {
    Image::from_fn(img.width, img.height, |x, y| {
        let mut a = [0.0f32; 3];
        for i in 0..len {
            let xx = (x + i).saturating_sub(len / 2).min(img.width - 1);
            let p = img.get(xx, y);
            for k in 0..3 { a[k] += p[k] / len as f32; }
        }
        a
    })
}
