//! « Mon Studio » : identité professionnelle, filigrane non destructif et exportation.
//!
//! * Le filigrane est une **description** (`Watermark`) appliquée à l'exportation ou à l'aperçu ; l'image source
//!   et la pile de retouches ne sont jamais modifiées.
//! * « Ajouter mon filigrane » décoché ⇒ aucun filigrane dans le fichier (testé octet pour octet).
//! * Les métadonnées de copyright (invisibles) sont gérées séparément du filigrane visible.
//! * Limites v0.1 : logos PNG avec transparence (le SVG est **prévu**, non supporté) ; signature manuscrite =
//!   image importée ou police script (tracé à la main : prévu) ; sortie PNG/JPEG 8 bits sRGB.

pub mod export;
pub mod profile;
pub mod templates;
pub mod watermark;
