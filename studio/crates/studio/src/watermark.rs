//! Moteur de filigrane : rendu réel de texte (Inter / Liberation Serif Italic, SIL OFL) et de logos PNG,
//! avec opacité, rotation, ombre, contour, répétition, marges, positions prédéfinies ou libres,
//! adaptation portrait/paysage. Composition en lumière linéaire. La source n'est jamais modifiée.

use fontdue::{Font, FontSettings};
use norelias_core::color::srgb_to_linear;
use norelias_core::Image;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

static INTER: &[u8] = include_bytes!("../assets/Inter-Regular.otf");
static SERIF_ITALIC: &[u8] = include_bytes!("../assets/LiberationSerif-Italic.ttf");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FontStyle { Sans, ElegantSerifItalic }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TextSource { StudioName, PhotographerName, Copyright, Custom(String) }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Anchor { TopLeft, Top, TopRight, Left, Center, Right, BottomLeft, Bottom, BottomRight }

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Position { Preset(Anchor), /// Centre du filigrane en fractions de l'image (0–1).
    Free { x: f32, y: f32 } }

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Shadow { /// Décalage en % du côté court.
    pub dx_pct: f32, pub dy_pct: f32, pub blur_pct: f32, pub color: [u8; 3], pub opacity: f32 }
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Outline { pub width_pct: f32, pub color: [u8; 3] }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Content { Text { source: TextSource, font: FontStyle, color: [u8; 3] }, /// Le logo lui-même est fourni dans `Brand`.
    Logo }

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Overrides { pub position: Option<Position>, pub size_pct: Option<f32>, pub margin_pct: Option<f32> }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Watermark {
    pub content: Content,
    /// Taille : % du côté court de l'image (hauteur du texte, largeur du logo).
    pub size_pct: f32,
    pub opacity: f32,
    pub rotation_deg: f32,
    pub position: Position,
    pub margin_pct: f32,
    pub shadow: Option<Shadow>,
    pub outline: Option<Outline>,
    /// Répétition : espacement en % du côté court.
    pub tile_gap_pct: Option<f32>,
    pub landscape: Overrides,
    pub portrait: Overrides,
}

impl Default for Watermark {
    fn default() -> Self {
        Self { content: Content::Text { source: TextSource::StudioName, font: FontStyle::ElegantSerifItalic, color: [255, 255, 255] },
            size_pct: 4.0, opacity: 0.8, rotation_deg: 0.0, position: Position::Preset(Anchor::BottomRight), margin_pct: 3.0, shadow: None, outline: None,
            tile_gap_pct: None, landscape: Overrides::default(), portrait: Overrides::default() }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LogoImage { pub width: usize, pub height: usize, /// RGBA 8 bits, non prémultiplié.
    pub rgba: Vec<u8> }

impl LogoImage {
    /// Décode un PNG (transparence conservée). Le SVG n'est pas encore pris en charge.
    pub fn from_png(bytes: &[u8]) -> Result<Self, WmError> {
        if bytes.len() < 8 || &bytes[..8] != b"\x89PNG\r\n\x1a\n" { return Err(WmError::LogoUnsupported); }
        let img = image::load_from_memory_with_format(bytes, image::ImageFormat::Png).map_err(|_| WmError::LogoUnsupported)?.to_rgba8();
        Ok(Self { width: img.width() as usize, height: img.height() as usize, rgba: img.into_raw() })
    }
}

/// Informations d'identité utilisées pour composer le texte.
#[derive(Clone, Debug, Default)]
pub struct Brand { pub studio_name: String, pub photographer_name: String, pub holder: String, pub year: i32, pub logo: Option<LogoImage> }

#[derive(Debug, PartialEq, Eq)]
pub enum WmError { EmptyText, BadParameter(&'static str), LogoMissing, LogoUnsupported }
impl std::fmt::Display for WmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self { Self::EmptyText => write!(f, "le texte du filigrane est vide"), Self::BadParameter(p) => write!(f, "paramètre hors limites : {p}"),
            Self::LogoMissing => write!(f, "aucun logo importé"), Self::LogoUnsupported => write!(f, "logo non pris en charge (PNG requis ; SVG prévu)") }
    }
}
impl std::error::Error for WmError {}

pub fn resolve_text(src: &TextSource, b: &Brand) -> String {
    let studio = if b.studio_name.trim().is_empty() { &b.photographer_name } else { &b.studio_name };
    let holder = if b.holder.trim().is_empty() { studio } else { &b.holder };
    let raw = match src { TextSource::StudioName => studio.clone(), TextSource::PhotographerName => b.photographer_name.clone(),
        TextSource::Copyright => "© {year} {holder}".to_string(), TextSource::Custom(s) => s.clone() };
    raw.replace("{year}", &b.year.to_string()).replace("{studio}", studio).replace("{photographer}", &b.photographer_name).replace("{holder}", holder)
}

fn font(style: FontStyle) -> &'static Font {
    static S: OnceLock<Font> = OnceLock::new();
    static E: OnceLock<Font> = OnceLock::new();
    match style {
        FontStyle::Sans => S.get_or_init(|| Font::from_bytes(INTER, FontSettings::default()).expect("Inter")),
        FontStyle::ElegantSerifItalic => E.get_or_init(|| Font::from_bytes(SERIF_ITALIC, FontSettings::default()).expect("Liberation Serif Italic")),
    }
}

/// Couche RGBA prémultipliée, lumière linéaire.
struct Layer { w: usize, h: usize, px: Vec<[f32; 4]> }
impl Layer {
    fn new(w: usize, h: usize) -> Self { Self { w, h, px: vec![[0.0; 4]; w * h] } }
    fn over(&mut self, i: usize, c: [f32; 3], a: f32) { let d = &mut self.px[i]; for k in 0..3 { d[k] = c[k] * a + d[k] * (1.0 - a); } d[3] = a + d[3] * (1.0 - a); }
}
fn lin(c: [u8; 3]) -> [f32; 3] { c.map(|v| srgb_to_linear(v as f32 / 255.0)) }

fn dilate(a: &[f32], w: usize, h: usize, r: usize) -> Vec<f32> {
    if r == 0 { return a.to_vec(); }
    let mut o = vec![0.0f32; a.len()];
    let r2 = (r * r) as i32 + 1;
    for y in 0..h { for x in 0..w {
        let mut m = 0.0f32;
        for dy in -(r as i32)..=r as i32 { for dx in -(r as i32)..=r as i32 {
            if dx * dx + dy * dy > r2 { continue; }
            let (nx, ny) = (x as i32 + dx, y as i32 + dy);
            if nx >= 0 && ny >= 0 && (nx as usize) < w && (ny as usize) < h { m = m.max(a[ny as usize * w + nx as usize]); }
        }}
        o[y * w + x] = m;
    }}
    o
}
fn blur(a: &[f32], w: usize, h: usize, r: usize) -> Vec<f32> {
    if r == 0 { return a.to_vec(); }
    let mut cur = a.to_vec();
    for _ in 0..2 {
        for horizontal in [true, false] {
            let src = cur.clone();
            for y in 0..h { for x in 0..w {
                let (mut s, mut n) = (0.0f32, 0.0f32);
                for d in -(r as i32)..=r as i32 {
                    let (nx, ny) = if horizontal { (x as i32 + d, y as i32) } else { (x as i32, y as i32 + d) };
                    if nx >= 0 && ny >= 0 && (nx as usize) < w && (ny as usize) < h { s += src[ny as usize * w + nx as usize]; }
                    n += 1.0;
                }
                cur[y * w + x] = s / n;
            }}
        }
    }
    cur
}
fn shift(a: &[f32], w: usize, h: usize, dx: i32, dy: i32) -> Vec<f32> {
    let mut o = vec![0.0; a.len()];
    for y in 0..h as i32 { for x in 0..w as i32 {
        let (sx, sy) = (x - dx, y - dy);
        if sx >= 0 && sy >= 0 && (sx as usize) < w && (sy as usize) < h { o[y as usize * w + x as usize] = a[sy as usize * w + sx as usize]; }
    }}
    o
}

fn text_layer(text: &str, style: FontStyle, color: [u8; 3], px: f32, shadow: Option<Shadow>, outline: Option<Outline>, short: f32) -> Layer {
    let f = font(style);
    let lm = f.horizontal_line_metrics(px).expect("métriques de ligne");
    let (ascent, descent) = (lm.ascent, lm.descent);
    let pad = (px * 0.35).ceil() as usize + shadow.map_or(0, |s| ((s.dx_pct.abs().max(s.dy_pct.abs()) + s.blur_pct) / 100.0 * short).ceil() as usize)
        + outline.map_or(0, |o| (o.width_pct / 100.0 * short).ceil() as usize);
    let glyphs: Vec<_> = text.chars().filter(|c| !c.is_control()).map(|c| f.rasterize(c, px)).collect();
    let adv: f32 = glyphs.iter().map(|(m, _)| m.advance_width).sum();
    let (w, h) = (adv.ceil() as usize + 2 * pad + 2, (ascent - descent).ceil() as usize + 2 * pad + 2);
    let mut a = vec![0.0f32; w * h];
    let mut pen = pad as f32;
    for (m, bmp) in &glyphs {
        let gx = (pen + m.xmin as f32).round() as i32;
        let gy = (pad as f32 + ascent - m.ymin as f32 - m.height as f32).round() as i32;
        for row in 0..m.height { for col in 0..m.width {
            let (x, y) = (gx + col as i32, gy + row as i32);
            if x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h {
                let v = bmp[row * m.width + col] as f32 / 255.0;
                let i = y as usize * w + x as usize;
                a[i] = a[i].max(v);
            }
        }}
        pen += m.advance_width;
    }
    let mut layer = Layer::new(w, h);
    if let Some(s) = shadow {
        let (dx, dy) = ((s.dx_pct / 100.0 * short).round() as i32, (s.dy_pct / 100.0 * short).round() as i32);
        let sa = blur(&shift(&a, w, h, dx, dy), w, h, (s.blur_pct / 100.0 * short).round() as usize);
        let c = lin(s.color);
        for (i, v) in sa.iter().enumerate() { if *v > 0.0 { layer.over(i, c, v * s.opacity.clamp(0.0, 1.0)); } }
    }
    if let Some(o) = outline {
        let oa = dilate(&a, w, h, (o.width_pct / 100.0 * short).round().max(1.0) as usize);
        let c = lin(o.color);
        for (i, v) in oa.iter().enumerate() { if *v > 0.0 { layer.over(i, c, *v); } }
    }
    let c = lin(color);
    for (i, v) in a.iter().enumerate() { if *v > 0.0 { layer.over(i, c, *v); } }
    layer
}

fn logo_layer(logo: &LogoImage, target_w: f32) -> Layer {
    let scale = (target_w / logo.width as f32).max(0.01);
    let (w, h) = (((logo.width as f32 * scale).round() as usize).max(1), ((logo.height as f32 * scale).round() as usize).max(1));
    let src = |x: usize, y: usize| -> [f32; 4] { let i = (y * logo.width + x) * 4; let a = logo.rgba[i + 3] as f32 / 255.0;
        let c = [0, 1, 2].map(|k| srgb_to_linear(logo.rgba[i + k] as f32 / 255.0)); [c[0] * a, c[1] * a, c[2] * a, a] };
    let mut l = Layer::new(w, h);
    for y in 0..h { for x in 0..w {
        // moyenne de la zone source couverte (réduction) ou échantillon le plus proche (agrandissement)
        let (x0, x1) = ((x as f32 / scale) as usize, (((x + 1) as f32 / scale).ceil() as usize).clamp(1, logo.width));
        let (y0, y1) = ((y as f32 / scale) as usize, (((y + 1) as f32 / scale).ceil() as usize).clamp(1, logo.height));
        let (mut acc, mut n) = ([0.0f32; 4], 0.0f32);
        for sy in y0.min(logo.height - 1)..y1.max(y0 + 1).min(logo.height) { for sx in x0.min(logo.width - 1)..x1.max(x0 + 1).min(logo.width) { let p = src(sx, sy); for k in 0..4 { acc[k] += p[k]; } n += 1.0; } }
        l.px[y * w + x] = acc.map(|v| v / n.max(1.0));
    }}
    l
}

fn rotate_layer(l: &Layer, deg: f32) -> Layer {
    if deg.rem_euclid(360.0) == 0.0 { return Layer { w: l.w, h: l.h, px: l.px.clone() }; }
    let t = deg.to_radians();
    let (s, c) = t.sin_cos();
    let (nw, nh) = (((l.w as f32 * c.abs() + l.h as f32 * s.abs()).ceil() as usize) + 1, ((l.w as f32 * s.abs() + l.h as f32 * c.abs()).ceil() as usize) + 1);
    let mut o = Layer::new(nw, nh);
    let (cx, cy, ocx, ocy) = ((l.w as f32 - 1.0) / 2.0, (l.h as f32 - 1.0) / 2.0, (nw as f32 - 1.0) / 2.0, (nh as f32 - 1.0) / 2.0);
    for y in 0..nh { for x in 0..nw {
        let (px, py) = (x as f32 - ocx, y as f32 - ocy);
        let (sx, sy) = (px * c + py * s + cx, -px * s + py * c + cy);
        if sx < -0.5 || sy < -0.5 || sx > l.w as f32 - 0.5 || sy > l.h as f32 - 0.5 { continue; }
        let (fx, fy) = (sx.clamp(0.0, l.w as f32 - 1.0), sy.clamp(0.0, l.h as f32 - 1.0));
        let (x0, y0) = (fx.floor() as usize, fy.floor() as usize);
        let (x1, y1) = ((x0 + 1).min(l.w - 1), (y0 + 1).min(l.h - 1));
        let (ax, ay) = (fx - x0 as f32, fy - y0 as f32);
        let g = |xx: usize, yy: usize| l.px[yy * l.w + xx];
        let mut v = [0.0f32; 4];
        for k in 0..4 { v[k] = (g(x0, y0)[k] * (1.0 - ax) + g(x1, y0)[k] * ax) * (1.0 - ay) + (g(x0, y1)[k] * (1.0 - ax) + g(x1, y1)[k] * ax) * ay; }
        o.px[y * nw + x] = v;
    }}
    o
}

fn paste(img: &mut Image, l: &Layer, ox: i32, oy: i32, opacity: f32) {
    for y in 0..l.h as i32 { for x in 0..l.w as i32 {
        let (ix, iy) = (ox + x, oy + y);
        if ix < 0 || iy < 0 || ix as usize >= img.width || iy as usize >= img.height { continue; }
        let p = l.px[y as usize * l.w + x as usize];
        if p[3] <= 0.0 { continue; }
        let mut d = img.get(ix as usize, iy as usize);
        for k in 0..3 { d[k] = d[k] * (1.0 - p[3] * opacity) + p[k] * opacity; }
        img.set(ix as usize, iy as usize, d);
    }}
}

fn validate(wm: &Watermark) -> Result<(), WmError> {
    if !(0.3..=80.0).contains(&wm.size_pct) { return Err(WmError::BadParameter("taille")); }
    if !(0.0..=1.0).contains(&wm.opacity) { return Err(WmError::BadParameter("opacité")); }
    if !(0.0..=40.0).contains(&wm.margin_pct) { return Err(WmError::BadParameter("marge")); }
    if !wm.rotation_deg.is_finite() || wm.rotation_deg.abs() > 360.0 { return Err(WmError::BadParameter("rotation")); }
    if let Some(g) = wm.tile_gap_pct { if !(0.0..=100.0).contains(&g) { return Err(WmError::BadParameter("espacement")); } }
    Ok(())
}

/// Applique le filigrane et retourne une **nouvelle** image.
pub fn apply(img: &Image, wm: &Watermark, brand: &Brand) -> Result<Image, WmError> {
    validate(wm)?;
    let short = img.width.min(img.height) as f32;
    let portrait = img.height > img.width;
    let ov = if portrait { wm.portrait } else { wm.landscape };
    let size_pct = ov.size_pct.unwrap_or(wm.size_pct);
    let margin = ov.margin_pct.unwrap_or(wm.margin_pct) / 100.0 * short;
    let position = ov.position.unwrap_or(wm.position);
    let base = match &wm.content {
        Content::Text { source, font, color } => {
            let text = resolve_text(source, brand);
            if text.trim().is_empty() { return Err(WmError::EmptyText); }
            text_layer(&text, *font, *color, (size_pct / 100.0 * short).max(4.0), wm.shadow, wm.outline, short)
        }
        Content::Logo => {
            let logo = brand.logo.as_ref().ok_or(WmError::LogoMissing)?;
            logo_layer(logo, (size_pct / 100.0 * short).max(4.0))
        }
    };
    let layer = rotate_layer(&base, wm.rotation_deg);
    let mut out = img.clone();
    if let Some(gap) = wm.tile_gap_pct {
        let g = gap / 100.0 * short;
        let (sx, sy) = ((layer.w as f32 + g).max(1.0), (layer.h as f32 + g).max(1.0));
        let rows = (img.height as f32 / sy).ceil() as i32 + 1;
        let cols = (img.width as f32 / sx).ceil() as i32 + 1;
        for r in 0..rows { for c in -1..cols {
            let off = if r % 2 == 1 { sx / 2.0 } else { 0.0 };
            paste(&mut out, &layer, (c as f32 * sx + off) as i32, (r as f32 * sy) as i32, wm.opacity);
        }}
        return Ok(out);
    }
    let (lw, lh) = (layer.w as f32, layer.h as f32);
    let (w, h) = (img.width as f32, img.height as f32);
    let (x, y) = match position {
        Position::Free { x, y } => (x.clamp(0.0, 1.0) * w - lw / 2.0, y.clamp(0.0, 1.0) * h - lh / 2.0),
        Position::Preset(a) => {
            let xs = [margin, (w - lw) / 2.0, w - lw - margin];
            let ys = [margin, (h - lh) / 2.0, h - lh - margin];
            let (i, j) = match a { Anchor::TopLeft => (0, 0), Anchor::Top => (1, 0), Anchor::TopRight => (2, 0), Anchor::Left => (0, 1), Anchor::Center => (1, 1),
                Anchor::Right => (2, 1), Anchor::BottomLeft => (0, 2), Anchor::Bottom => (1, 2), Anchor::BottomRight => (2, 2) };
            (xs[i], ys[j])
        }
    };
    paste(&mut out, &layer, x.round() as i32, y.round() as i32, wm.opacity);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use norelias_core::synth;

    fn brand() -> Brand { Brand { studio_name: "Norélias Photography".into(), photographer_name: "Élodie Côté".into(), holder: "".into(), year: 2026, logo: None } }
    fn flat(w: usize, h: usize) -> Image { Image::from_fn(w, h, |_, _| [0.2, 0.2, 0.2]) }
    /// Boîte englobante des pixels modifiés.
    fn diff_bbox(a: &Image, b: &Image) -> Option<(usize, usize, usize, usize)> {
        let (mut x0, mut y0, mut x1, mut y1) = (usize::MAX, usize::MAX, 0, 0);
        let mut any = false;
        for y in 0..a.height { for x in 0..a.width { if a.get(x, y) != b.get(x, y) { any = true; x0 = x0.min(x); y0 = y0.min(y); x1 = x1.max(x); y1 = y1.max(y); } } }
        any.then_some((x0, y0, x1, y1))
    }
    fn changed(a: &Image, b: &Image) -> usize { a.data.chunks(3).zip(b.data.chunks(3)).filter(|(p, q)| p != q).count() }

    #[test]
    fn text_is_resolved_from_studio_with_placeholders() {
        let b = brand();
        assert_eq!(resolve_text(&TextSource::StudioName, &b), "Norélias Photography");
        assert_eq!(resolve_text(&TextSource::Copyright, &b), "© 2026 Norélias Photography");
        assert_eq!(resolve_text(&TextSource::Custom("{studio} — {year}".into()), &b), "Norélias Photography — 2026");
        let indep = Brand { studio_name: "".into(), ..brand() };
        assert_eq!(resolve_text(&TextSource::StudioName, &indep), "Élodie Côté", "photographe indépendant");
    }
    #[test]
    fn renders_real_pixels_in_the_chosen_corner_and_leaves_source_untouched() {
        let img = flat(800, 600);
        let snap = img.clone();
        let out = apply(&img, &Watermark::default(), &brand()).unwrap();
        assert_eq!(img, snap);
        let (x0, y0, x1, y1) = diff_bbox(&img, &out).expect("le filigrane doit modifier des pixels");
        assert!(x0 > 400 && y0 > 300 && x1 < 800 && y1 < 600, "bas-droite : {x0},{y0},{x1},{y1}");
        assert!(out.data.iter().cloned().fold(0.0, f32::max) > 0.5, "texte blanc visible sur fond sombre");
    }
    #[test]
    fn every_preset_lands_where_it_says() {
        let img = flat(900, 600);
        for (a, left, top) in [(Anchor::TopLeft, true, true), (Anchor::BottomLeft, true, false), (Anchor::TopRight, false, true), (Anchor::BottomRight, false, false)] {
            let wm = Watermark { position: Position::Preset(a), ..Default::default() };
            let (x0, y0, x1, y1) = diff_bbox(&img, &apply(&img, &wm, &brand()).unwrap()).unwrap();
            assert_eq!((x1 < 450, y1 < 300), (left, top), "{a:?}");
            assert!(x0 < 900 && y0 < 600);
        }
        let c = Watermark { position: Position::Preset(Anchor::Center), ..Default::default() };
        let (x0, _, x1, _) = diff_bbox(&img, &apply(&img, &c, &brand()).unwrap()).unwrap();
        assert!(((x0 + x1) as f32 / 2.0 - 450.0).abs() < 8.0);
        let f = Watermark { position: Position::Free { x: 0.25, y: 0.75 }, ..Default::default() };
        let (x0, y0, x1, y1) = diff_bbox(&img, &apply(&img, &f, &brand()).unwrap()).unwrap();
        assert!(((x0 + x1) as f32 / 2.0 - 225.0).abs() < 12.0 && ((y0 + y1) as f32 / 2.0 - 450.0).abs() < 12.0);
    }
    #[test]
    fn margin_moves_the_mark_inward() {
        let img = flat(800, 600);
        let at = |m| { let wm = Watermark { margin_pct: m, ..Default::default() }; diff_bbox(&img, &apply(&img, &wm, &brand()).unwrap()).unwrap() };
        assert!(at(10.0).2 < at(1.0).2 && at(10.0).3 < at(1.0).3);
    }
    #[test]
    fn opacity_scales_the_effect_and_zero_is_invisible() {
        let img = flat(600, 400);
        let mean = |o: f32| { let w = Watermark { opacity: o, ..Default::default() }; let out = apply(&img, &w, &brand()).unwrap(); out.data.iter().sum::<f32>() };
        let (m0, m5, m10) = (mean(0.0), mean(0.5), mean(1.0));
        assert_eq!(m0, img.data.iter().sum::<f32>(), "opacité 0 = aucun changement");
        assert!(m0 < m5 && m5 < m10);
    }
    #[test]
    fn size_changes_the_footprint() {
        let img = flat(1000, 700);
        let w = |s| { let wm = Watermark { size_pct: s, ..Default::default() }; let b = diff_bbox(&img, &apply(&img, &wm, &brand()).unwrap()).unwrap(); b.2 - b.0 };
        assert!(w(8.0) > w(4.0) * 3 / 2);
    }
    #[test]
    fn rotation_turns_a_wide_mark_into_a_tall_one() {
        let img = flat(1000, 1000);
        let wm = Watermark { rotation_deg: 90.0, position: Position::Preset(Anchor::Center), size_pct: 6.0, ..Default::default() };
        let (x0, y0, x1, y1) = diff_bbox(&img, &apply(&img, &wm, &brand()).unwrap()).unwrap();
        assert!(y1 - y0 > (x1 - x0) * 3, "{}x{}", x1 - x0, y1 - y0);
    }
    #[test]
    fn shadow_outline_and_tiling_add_pixels() {
        let img = flat(800, 600);
        let plain = changed(&img, &apply(&img, &Watermark::default(), &brand()).unwrap());
        let sh = Watermark { shadow: Some(Shadow { dx_pct: 0.4, dy_pct: 0.4, blur_pct: 0.4, color: [0, 0, 0], opacity: 0.7 }), ..Default::default() };
        let ol = Watermark { outline: Some(Outline { width_pct: 0.3, color: [0, 0, 0] }), ..Default::default() };
        let tl = Watermark { tile_gap_pct: Some(8.0), opacity: 0.3, ..Default::default() };
        assert!(changed(&img, &apply(&img, &sh, &brand()).unwrap()) > plain);
        assert!(changed(&img, &apply(&img, &ol, &brand()).unwrap()) > plain);
        assert!(changed(&img, &apply(&img, &tl, &brand()).unwrap()) > plain * 5, "répété sur toute l'image");
    }
    #[test]
    fn portrait_and_landscape_overrides() {
        let wm = Watermark { portrait: Overrides { position: Some(Position::Preset(Anchor::Bottom)), size_pct: Some(8.0), margin_pct: None }, ..Default::default() };
        let land = flat(900, 600); let port = flat(600, 900);
        let l = diff_bbox(&land, &apply(&land, &wm, &brand()).unwrap()).unwrap();
        let p = diff_bbox(&port, &apply(&port, &wm, &brand()).unwrap()).unwrap();
        assert!(l.0 > 450, "paysage : bas-droite par défaut");
        assert!(p.0 < 300 && p.2 > 300, "portrait : centré en bas : {p:?}");
    }
    #[test]
    fn unicode_text_renders_and_empty_is_rejected() {
        let img = flat(800, 400);
        let wm = Watermark { content: Content::Text { source: TextSource::Custom("Élodie Côté ✓ © 2026".into()), font: FontStyle::Sans, color: [255, 255, 255] }, ..Default::default() };
        assert!(changed(&img, &apply(&img, &wm, &brand()).unwrap()) > 200);
        let empty = Watermark { content: Content::Text { source: TextSource::Custom("   ".into()), font: FontStyle::Sans, color: [0; 3] }, ..Default::default() };
        assert_eq!(apply(&img, &empty, &brand()), Err(WmError::EmptyText));
    }
    #[test]
    fn parameter_validation() {
        let img = flat(100, 100);
        for wm in [Watermark { opacity: 1.5, ..Default::default() }, Watermark { size_pct: 0.0, ..Default::default() }, Watermark { margin_pct: -1.0, ..Default::default() },
                   Watermark { rotation_deg: f32::NAN, ..Default::default() }] { assert!(matches!(apply(&img, &wm, &brand()), Err(WmError::BadParameter(_)))); }
    }
    fn logo() -> LogoImage {
        // 40×20 : bords transparents, centre rouge opaque
        let mut rgba = vec![0u8; 40 * 20 * 4];
        for y in 5..15 { for x in 10..30 { let i = (y * 40 + x) * 4; rgba[i..i + 4].copy_from_slice(&[255, 0, 0, 255]); } }
        LogoImage { width: 40, height: 20, rgba }
    }
    #[test]
    fn transparent_logo_keeps_its_transparency() {
        let img = flat(800, 600);
        let wm = Watermark { content: Content::Logo, size_pct: 20.0, opacity: 1.0, position: Position::Preset(Anchor::Center), ..Default::default() };
        let mut b = brand(); b.logo = Some(logo());
        let out = apply(&img, &wm, &b).unwrap();
        let (x0, y0, x1, y1) = diff_bbox(&img, &out).unwrap();
        // logo de 120 px de large (20 % de 600) : la zone opaque (moitié centrale) est plus petite que le cadre
        assert!(x1 - x0 < 70 && y1 - y0 < 40, "{x0},{y0},{x1},{y1}");
        let p = out.get(400, 300);
        assert!(p[0] > 0.8 && p[1] < 0.1, "rouge opaque au centre : {p:?}");
        assert_eq!(apply(&img, &wm, &brand()), Err(WmError::LogoMissing));
    }
    #[test]
    fn png_logo_decoding() {
        assert_eq!(LogoImage::from_png(b"<svg></svg>"), Err(WmError::LogoUnsupported), "SVG non pris en charge : refus explicite");
        let mut buf = vec![];
        { let mut e = png::Encoder::new(&mut buf, 2, 2); e.set_color(png::ColorType::Rgba); e.set_depth(png::BitDepth::Eight);
          let mut w = e.write_header().unwrap(); w.write_image_data(&[255, 0, 0, 255, 0, 255, 0, 0, 0, 0, 255, 128, 9, 9, 9, 255]).unwrap(); }
        let l = LogoImage::from_png(&buf).unwrap();
        assert_eq!((l.width, l.height, l.rgba[7]), (2, 2, 0), "alpha conservé");
    }
    #[test]
    fn preview_matches_export_proportions() {
        let full = synth::scene(1600, 1000, 0.0);
        let wm = Watermark { position: Position::Preset(Anchor::BottomRight), ..Default::default() };
        let (a, b) = (apply(&full, &wm, &brand()).unwrap(), apply(&full.downscale_to(400), &wm, &brand()).unwrap());
        let (bf, bp) = (diff_bbox(&full, &a).unwrap(), diff_bbox(&full.downscale_to(400), &b).unwrap());
        let ratio = |v: usize, d: usize| v as f32 / d as f32;
        assert!((ratio(bf.0, 1600) - ratio(bp.0, 400)).abs() < 0.02 && (ratio(bf.3, 1000) - ratio(bp.3, 250)).abs() < 0.02, "aperçu fidèle à l'export");
    }
}
