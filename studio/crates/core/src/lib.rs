//! Moteur d'image de Norélias Studio Pro (v0).
//!
//! Indépendant de toute interface. Pixels en RVB **linéaire** flottant 32 bits ;
//! la conversion vers sRGB 8 bits n'a lieu qu'à la sortie. Aucune dépendance native.
//!
//! Limites actuelles (voir FEATURE_MATRIX.md) : traitement image entière sur CPU,
//! sans tuiles ni GPU ; pas de décodage RAW ; pas de gestion ICC (sRGB supposé).

pub mod color;
pub mod image;
pub mod ops;
pub mod synth;

pub use image::{Image, ImageError};
pub use ops::Op;

/// Langues d'interface prises en charge.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Lang {
    Fr,
    En,
}
