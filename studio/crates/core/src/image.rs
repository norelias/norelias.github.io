use crate::color::{linear_to_srgb, luma, srgb_to_linear};

#[derive(Debug, PartialEq, Eq)]
pub enum ImageError {
    BadLength { expected: usize, got: usize },
    EmptyImage,
}
impl std::fmt::Display for ImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadLength { expected, got } => write!(f, "taille de tampon invalide : {got} octets au lieu de {expected}"),
            Self::EmptyImage => write!(f, "image vide"),
        }
    }
}
impl std::error::Error for ImageError {}

/// Image RVB linéaire, flottante, entrelacée. Les valeurs peuvent dépasser 1.0 (marge de manœuvre).
#[derive(Clone, Debug, PartialEq)]
pub struct Image {
    pub width: usize,
    pub height: usize,
    pub data: Vec<f32>,
}

impl Image {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height, data: vec![0.0; width * height * 3] }
    }
    pub fn from_fn(width: usize, height: usize, mut f: impl FnMut(usize, usize) -> [f32; 3]) -> Self {
        let mut img = Self::new(width, height);
        for y in 0..height {
            for x in 0..width {
                img.set(x, y, f(x, y));
            }
        }
        img
    }
    /// Construit depuis du RVB sRGB 8 bits entrelacé.
    pub fn from_srgb8(width: usize, height: usize, rgb: &[u8]) -> Result<Self, ImageError> {
        if width == 0 || height == 0 {
            return Err(ImageError::EmptyImage);
        }
        let expected = width * height * 3;
        if rgb.len() != expected {
            return Err(ImageError::BadLength { expected, got: rgb.len() });
        }
        let lut: Vec<f32> = (0..256).map(|i| srgb_to_linear(i as f32 / 255.0)).collect();
        Ok(Self { width, height, data: rgb.iter().map(|&b| lut[b as usize]).collect() })
    }
    pub fn to_srgb8(&self) -> Vec<u8> {
        self.data.iter().map(|&v| (linear_to_srgb(v) * 255.0 + 0.5) as u8).collect()
    }
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> [f32; 3] {
        let i = (y * self.width + x) * 3;
        [self.data[i], self.data[i + 1], self.data[i + 2]]
    }
    #[inline]
    pub fn set(&mut self, x: usize, y: usize, p: [f32; 3]) {
        let i = (y * self.width + x) * 3;
        self.data[i..i + 3].copy_from_slice(&p);
    }
    /// Luminance linéaire par pixel.
    pub fn luma_plane(&self) -> Vec<f32> {
        self.data.chunks_exact(3).map(|p| luma(p[0], p[1], p[2])).collect()
    }
    /// Réduction par moyenne de blocs pour que le plus grand côté soit ≤ `max_side` (copie sinon).
    pub fn downscale_to(&self, max_side: usize) -> Image {
        let m = self.width.max(self.height);
        if m <= max_side || max_side == 0 {
            return self.clone();
        }
        let f = m.div_ceil(max_side);
        let (nw, nh) = ((self.width / f).max(1), (self.height / f).max(1));
        Image::from_fn(nw, nh, |ox, oy| {
            let mut acc = [0.0f32; 3];
            let mut n = 0.0;
            for dy in 0..f {
                for dx in 0..f {
                    let (x, y) = (ox * f + dx, oy * f + dy);
                    if x < self.width && y < self.height {
                        let p = self.get(x, y);
                        acc[0] += p[0];
                        acc[1] += p[1];
                        acc[2] += p[2];
                        n += 1.0;
                    }
                }
            }
            [acc[0] / n, acc[1] / n, acc[2] / n]
        })
    }
    /// Rééchantillonnage à dimensions exactes par moyenne d'aire (réduction de qualité ; agrandissement bilinéaire).
    pub fn resize(&self, nw: usize, nh: usize) -> Image {
        let (nw, nh) = (nw.max(1), nh.max(1));
        if (nw, nh) == (self.width, self.height) { return self.clone(); }
        let (sx, sy) = (self.width as f32 / nw as f32, self.height as f32 / nh as f32);
        Image::from_fn(nw, nh, |ox, oy| {
            if sx <= 1.0 && sy <= 1.0 { return self.sample((ox as f32 + 0.5) * sx - 0.5, (oy as f32 + 0.5) * sy - 0.5); }
            let (x0, x1, y0, y1) = (ox as f32 * sx, (ox as f32 + 1.0) * sx, oy as f32 * sy, (oy as f32 + 1.0) * sy);
            let (mut acc, mut wsum) = ([0.0f32; 3], 0.0f32);
            for y in y0.floor() as usize..(y1.ceil() as usize).min(self.height) {
                let wy = (y1.min(y as f32 + 1.0) - y0.max(y as f32)).max(0.0);
                for x in x0.floor() as usize..(x1.ceil() as usize).min(self.width) {
                    let w = wy * (x1.min(x as f32 + 1.0) - x0.max(x as f32)).max(0.0);
                    let p = self.get(x, y);
                    for k in 0..3 { acc[k] += p[k] * w; }
                    wsum += w;
                }
            }
            [acc[0] / wsum, acc[1] / wsum, acc[2] / wsum]
        })
    }
    /// Réduction proportionnelle pour que le plus grand côté vaille exactement `max_side` (jamais d'agrandissement).
    pub fn fit_within(&self, max_side: usize) -> Image {
        let m = self.width.max(self.height);
        if max_side == 0 || m <= max_side { return self.clone(); }
        let f = max_side as f32 / m as f32;
        self.resize((self.width as f32 * f).round() as usize, (self.height as f32 * f).round() as usize)
    }
    /// Rognage en pixels (borné à l'image).
    pub fn crop_px(&self, x: usize, y: usize, w: usize, h: usize) -> Image {
        let x = x.min(self.width.saturating_sub(1));
        let y = y.min(self.height.saturating_sub(1));
        let w = w.min(self.width - x).max(1);
        let h = h.min(self.height - y).max(1);
        Image::from_fn(w, h, |i, j| self.get(x + i, y + j))
    }
    /// Échantillonnage bilinéaire avec bords répliqués.
    pub fn sample(&self, x: f32, y: f32) -> [f32; 3] {
        let x = x.clamp(0.0, (self.width - 1) as f32);
        let y = y.clamp(0.0, (self.height - 1) as f32);
        let (x0, y0) = (x.floor() as usize, y.floor() as usize);
        let (x1, y1) = ((x0 + 1).min(self.width - 1), (y0 + 1).min(self.height - 1));
        let (fx, fy) = (x - x0 as f32, y - y0 as f32);
        let (a, b, c, d) = (self.get(x0, y0), self.get(x1, y0), self.get(x0, y1), self.get(x1, y1));
        let mut o = [0.0; 3];
        for k in 0..3 {
            o[k] = (a[k] * (1.0 - fx) + b[k] * fx) * (1.0 - fy) + (c[k] * (1.0 - fx) + d[k] * fx) * fy;
        }
        o
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn srgb8_roundtrip_is_lossless() {
        let bytes: Vec<u8> = (0..4 * 4 * 3).map(|i| (i * 5 % 256) as u8).collect();
        let img = Image::from_srgb8(4, 4, &bytes).unwrap();
        assert_eq!(img.to_srgb8(), bytes);
    }
    #[test]
    fn rejects_bad_buffers() {
        assert!(matches!(Image::from_srgb8(2, 2, &[0; 5]), Err(ImageError::BadLength { .. })));
        assert_eq!(Image::from_srgb8(0, 2, &[]), Err(ImageError::EmptyImage));
    }
    #[test]
    fn resize_exact_dimensions_and_mean() {
        let img = Image::from_fn(640, 420, |x, y| [x as f32 / 640.0, y as f32 / 420.0, 0.3]);
        let r = img.fit_within(200);
        assert_eq!((r.width, r.height), (200, 131));
        let m = |i: &Image| i.data.iter().sum::<f32>() / i.data.len() as f32;
        assert!((m(&img) - m(&r)).abs() < 2e-3);
        assert_eq!(img.fit_within(1000), img, "pas d'agrandissement");
        let up = Image::from_fn(4, 4, |x, _| [x as f32, 0.0, 0.0]).resize(8, 8);
        assert_eq!((up.width, up.height), (8, 8));
    }
    #[test]
    fn downscale_preserves_mean() {
        let img = Image::from_fn(64, 32, |x, _| [x as f32 / 64.0, 0.5, 0.1]);
        let s = img.downscale_to(16);
        assert!(s.width <= 16 && s.height <= 16);
        let m = |i: &Image| i.data.iter().sum::<f32>() / i.data.len() as f32;
        assert!((m(&img) - m(&s)).abs() < 1e-3);
    }
}
