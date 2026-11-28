//! Fonctions de transfert sRGB (IEC 61966-2-1) et luminance Rec. 709.

pub const LUMA: [f32; 3] = [0.2126, 0.7152, 0.0722];

pub fn srgb_to_linear(v: f32) -> f32 {
    if v <= 0.04045 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
}

/// Encode une valeur linéaire en sRGB, bornée à [0, 1].
pub fn linear_to_srgb(v: f32) -> f32 {
    let v = v.clamp(0.0, 1.0);
    if v <= 0.003_130_8 { v * 12.92 } else { 1.055 * v.powf(1.0 / 2.4) - 0.055 }
}

pub fn luma(r: f32, g: f32, b: f32) -> f32 {
    LUMA[0] * r + LUMA[1] * g + LUMA[2] * b
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roundtrip_all_8bit_values() {
        for i in 0..=255u32 {
            let v = i as f32 / 255.0;
            let back = (linear_to_srgb(srgb_to_linear(v)) * 255.0).round() as u32;
            assert_eq!(back, i);
        }
    }
    #[test]
    fn known_points() {
        assert!((srgb_to_linear(0.5) - 0.214_041).abs() < 1e-5);
        assert!((luma(1.0, 1.0, 1.0) - 1.0).abs() < 1e-6);
    }
}
