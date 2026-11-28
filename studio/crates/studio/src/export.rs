//! Exportation PNG / JPEG 8 bits sRGB avec filigrane facultatif.
//!
//! Règles (testées) :
//! * `add_watermark = false` ⇒ aucun filigrane, fichier identique à une exportation sans réglage de filigrane ;
//! * `also_without_watermark` ⇒ deux fichiers (`_filigrane` et `_sans-filigrane`) ;
//! * le copyright **invisible** est indépendant du filigrane visible : PNG → bloc iTXt UTF-8 ; JPEG → fichier
//!   annexe `.xmp` (l'écriture EXIF/XMP intégrée au JPEG est prévue) ;
//! * aucun fichier existant n'est jamais écrasé ; tout est vérifié avant d'écrire quoi que ce soit.

use crate::watermark::{apply, Brand, Watermark, WmError};
use norelias_core::Image;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format { Png, Jpeg { quality: u8 } }
impl Format { fn ext(self) -> &'static str { match self { Format::Png => "png", Format::Jpeg { .. } => "jpg" } } }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CopyrightMeta { pub notice: String, pub url: Option<String> }

#[derive(Clone, Debug)]
pub struct ExportSettings {
    pub format: Format,
    /// Case « Ajouter mon filigrane ».
    pub add_watermark: bool,
    pub also_without_watermark: bool,
    pub watermark: Option<Watermark>,
    pub copyright: Option<CopyrightMeta>,
    /// Plus grand côté en pixels (réduction seulement).
    pub max_side: Option<usize>,
}
impl Default for ExportSettings {
    fn default() -> Self { Self { format: Format::Png, add_watermark: false, also_without_watermark: false, watermark: None, copyright: None, max_side: None } }
}

#[derive(Debug)]
pub enum ExportError { Watermark(WmError), WatermarkNotConfigured, Exists(PathBuf), BadName, Io(std::io::Error), Encode(String) }
impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self { Self::Watermark(e) => write!(f, "filigrane : {e}"), Self::WatermarkNotConfigured => write!(f, "« Ajouter mon filigrane » est coché mais aucun filigrane n'est défini"),
            Self::Exists(p) => write!(f, "{} existe déjà : aucun fichier n'est écrasé", p.display()), Self::BadName => write!(f, "nom de fichier invalide"),
            Self::Io(e) => write!(f, "erreur disque : {e}"), Self::Encode(e) => write!(f, "encodage : {e}") }
    }
}
impl std::error::Error for ExportError {}
impl From<std::io::Error> for ExportError { fn from(e: std::io::Error) -> Self { Self::Io(e) } }

/// Nettoie un nom de fichier en conservant l'Unicode : retire séparateurs de chemin, caractères de contrôle et
/// caractères interdits sous Windows, noms réservés, points/espaces finaux ; borne à 200 octets.
pub fn sanitize_filename(name: &str) -> Option<String> {
    let mut s: String = name.chars().map(|c| if c.is_control() || "/\\:*?\"<>|".contains(c) { '_' } else { c }).collect();
    s = s.trim().trim_end_matches(['.', ' ']).trim_start_matches('.').to_string();
    while s.len() > 200 { s.pop(); }
    let stem = s.split('.').next().unwrap_or("").to_uppercase();
    let reserved = ["CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "LPT1", "LPT2", "LPT3"];
    if s.is_empty() || reserved.contains(&stem.as_str()) { return None; }
    Some(s)
}

fn encode(img: &Image, format: Format, copyright: Option<&CopyrightMeta>) -> Result<Vec<u8>, ExportError> {
    let rgb = img.to_srgb8();
    let mut out = Vec::new();
    match format {
        Format::Png => {
            let mut enc = png::Encoder::new(&mut out, img.width as u32, img.height as u32);
            enc.set_color(png::ColorType::Rgb);
            enc.set_depth(png::BitDepth::Eight);
            enc.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
            if let Some(c) = copyright {
                enc.add_itxt_chunk("Copyright".into(), c.notice.clone()).map_err(|e| ExportError::Encode(e.to_string()))?;
                if let Some(u) = &c.url { enc.add_itxt_chunk("Source".into(), u.clone()).map_err(|e| ExportError::Encode(e.to_string()))?; }
            }
            let mut w = enc.write_header().map_err(|e| ExportError::Encode(e.to_string()))?;
            w.write_image_data(&rgb).map_err(|e| ExportError::Encode(e.to_string()))?;
            w.finish().map_err(|e| ExportError::Encode(e.to_string()))?;
        }
        Format::Jpeg { quality } => {
            image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, quality.clamp(1, 100))
                .encode(&rgb, img.width as u32, img.height as u32, image::ExtendedColorType::Rgb8).map_err(|e| ExportError::Encode(e.to_string()))?;
        }
    }
    Ok(out)
}

fn xmp_sidecar(c: &CopyrightMeta) -> String {
    let esc = |s: &str| s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;");
    format!("<?xpacket begin=\"\u{feff}\" id=\"W5M0MpCehiHzreSzNTczkc9d\"?>\n<x:xmpmeta xmlns:x=\"adobe:ns:meta/\"><rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\">\n <rdf:Description xmlns:dc=\"http://purl.org/dc/elements/1.1/\" xmlns:xmpRights=\"http://ns.adobe.com/xap/1.0/rights/\" xmpRights:Marked=\"True\"{}>\n  <dc:rights><rdf:Alt><rdf:li xml:lang=\"x-default\">{}</rdf:li></rdf:Alt></dc:rights>\n </rdf:Description>\n</rdf:RDF></x:xmpmeta>\n<?xpacket end=\"w\"?>\n",
        c.url.as_ref().map(|u| format!(" xmpRights:WebStatement=\"{}\"", esc(u))).unwrap_or_default(), esc(&c.notice))
}

struct Planned { path: PathBuf, bytes: Vec<u8>, sidecar: Option<(PathBuf, String)> }

fn plan(img: &Image, base: &str, dir: &Path, s: &ExportSettings, brand: &Brand) -> Result<Vec<Planned>, ExportError> {
    let base = sanitize_filename(base).ok_or(ExportError::BadName)?;
    let src = match s.max_side { Some(m) if m > 0 => img.fit_within(m), _ => img.clone() };
    let mut variants: Vec<(Image, &'static str)> = vec![];
    if s.add_watermark {
        let wm = s.watermark.as_ref().ok_or(ExportError::WatermarkNotConfigured)?;
        let marked = apply(&src, wm, brand).map_err(ExportError::Watermark)?;
        if s.also_without_watermark { variants.push((marked, "_filigrane")); variants.push((src, "_sans-filigrane")); } else { variants.push((marked, "")); }
    } else {
        variants.push((src, ""));
    }
    let mut out = vec![];
    for (im, suffix) in variants {
        let path = dir.join(format!("{base}{suffix}.{}", s.format.ext()));
        let bytes = encode(&im, s.format, s.copyright.as_ref())?;
        let sidecar = match (&s.copyright, s.format) { (Some(c), Format::Jpeg { .. }) => Some((path.with_extension("xmp"), xmp_sidecar(c))), _ => None };
        out.push(Planned { path, bytes, sidecar });
    }
    Ok(out)
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), ExportError> {
    use std::io::Write;
    let tmp = path.with_extension(format!("{}.part", path.extension().and_then(|e| e.to_str()).unwrap_or("")));
    { let mut f = std::fs::OpenOptions::new().write(true).create_new(true).open(&tmp)?; f.write_all(bytes)?; f.sync_all()?; }
    if path.exists() { let _ = std::fs::remove_file(&tmp); return Err(ExportError::Exists(path.to_path_buf())); }
    std::fs::rename(tmp, path)?;
    Ok(())
}

/// Exporte une image. Tout est encodé et vérifié avant la première écriture ; aucun fichier existant n'est écrasé.
pub fn export_one(img: &Image, base_name: &str, dir: &Path, s: &ExportSettings, brand: &Brand) -> Result<Vec<PathBuf>, ExportError> {
    let planned = plan(img, base_name, dir, s, brand)?;
    for p in &planned {
        for target in std::iter::once(&p.path).chain(p.sidecar.iter().map(|(sp, _)| sp)) { if target.exists() { return Err(ExportError::Exists(target.clone())); } }
    }
    std::fs::create_dir_all(dir)?;
    let mut files = vec![];
    for p in planned {
        write_new(&p.path, &p.bytes)?;
        if let Some((sp, x)) = p.sidecar { write_new(&sp, x.as_bytes())?; }
        files.push(p.path);
    }
    Ok(files)
}

/// Application en lot : mêmes réglages pour chaque image (orientation adaptée image par image).
/// Une erreur sur une image n'arrête pas les autres ; chaque résultat est retourné.
pub fn export_batch(items: &[(String, Image)], dir: &Path, s: &ExportSettings, brand: &Brand) -> Vec<Result<Vec<PathBuf>, ExportError>> {
    items.iter().map(|(name, img)| export_one(img, name, dir, s, brand)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::watermark::*;
    use norelias_core::synth;

    fn tmp(n: &str) -> PathBuf { let d = std::env::temp_dir().join(format!("norelias-export-{n}-{}", std::process::id())); let _ = std::fs::remove_dir_all(&d); d }
    fn brand() -> Brand { Brand { studio_name: "Norélias Photography".into(), photographer_name: "Élodie".into(), holder: "".into(), year: 2026, logo: None } }
    fn settings(wm: bool, dual: bool) -> ExportSettings { ExportSettings { add_watermark: wm, also_without_watermark: dual, watermark: Some(Watermark::default()), ..Default::default() } }
    fn img() -> Image { synth::scene(640, 420, 0.0) }
    fn decode(p: &Path) -> Vec<u8> { image::open(p).unwrap().to_rgb8().into_raw() }

    #[test]
    fn unchecked_box_means_no_watermark_at_all() {
        let d = tmp("off");
        let with_settings_unchecked = export_one(&img(), "a", &d, &settings(false, false), &brand()).unwrap();
        let none_defined = export_one(&img(), "b", &d, &ExportSettings::default(), &brand()).unwrap();
        assert_eq!(std::fs::read(&with_settings_unchecked[0]).unwrap(), std::fs::read(&none_defined[0]).unwrap(), "octet pour octet identique à une exportation sans filigrane");
        assert_eq!(decode(&with_settings_unchecked[0]), img().to_srgb8(), "pixels = source");
        let _ = std::fs::remove_dir_all(&d);
    }
    #[test]
    fn checked_box_marks_the_file() {
        let d = tmp("on");
        let f = export_one(&img(), "a", &d, &settings(true, false), &brand()).unwrap();
        assert_eq!(f.len(), 1);
        assert_ne!(decode(&f[0]), img().to_srgb8());
        let _ = std::fs::remove_dir_all(&d);
    }
    #[test]
    fn dual_export_makes_both_versions() {
        let d = tmp("dual");
        let f = export_one(&img(), "séance", &d, &settings(true, true), &brand()).unwrap();
        let names: Vec<String> = f.iter().map(|p| p.file_name().unwrap().to_string_lossy().into()).collect();
        assert_eq!(names, vec!["séance_filigrane.png", "séance_sans-filigrane.png"]);
        assert_eq!(decode(&f[1]), img().to_srgb8());
        assert_ne!(decode(&f[0]), decode(&f[1]));
        let g = export_one(&img(), "x", &d, &settings(false, true), &brand()).unwrap();
        assert_eq!(g.len(), 1, "case décochée : « aussi sans filigrane » n'a pas d'effet");
        let _ = std::fs::remove_dir_all(&d);
    }
    #[test]
    fn checked_without_definition_is_an_error_not_a_silent_skip() {
        let s = ExportSettings { add_watermark: true, watermark: None, ..Default::default() };
        assert!(matches!(export_one(&img(), "a", &tmp("nodef"), &s, &brand()), Err(ExportError::WatermarkNotConfigured)));
    }
    #[test]
    fn never_overwrites_and_writes_nothing_on_conflict() {
        let d = tmp("conflict");
        export_one(&img(), "a", &d, &settings(true, true), &brand()).unwrap();
        let before: Vec<_> = std::fs::read_dir(&d).unwrap().map(|e| e.unwrap().file_name()).collect();
        std::fs::remove_file(d.join("a_sans-filigrane.png")).unwrap();
        let r = export_one(&img(), "a", &d, &settings(true, true), &brand());
        assert!(matches!(r, Err(ExportError::Exists(_))));
        assert!(!d.join("a_sans-filigrane.png").exists(), "rien n'est écrit si un des fichiers existe déjà");
        assert!(before.len() == 2);
        assert!(!std::fs::read_dir(&d).unwrap().any(|e| e.unwrap().file_name().to_string_lossy().ends_with(".part")));
        let _ = std::fs::remove_dir_all(&d);
    }
    #[test]
    fn copyright_metadata_is_independent_of_the_visible_watermark() {
        let d = tmp("meta");
        let c = Some(CopyrightMeta { notice: "© 2026 Norélias Photography — Tous droits réservés".into(), url: Some("https://norelias.ca".into()) });
        let read = |p: &Path| { let dec = png::Decoder::new(std::io::BufReader::new(std::fs::File::open(p).unwrap())); let r = dec.read_info().unwrap();
            r.info().utf8_text.iter().map(|t| (t.keyword.clone(), t.get_text().unwrap())).collect::<Vec<_>>() };
        let no_wm_with_meta = export_one(&img(), "a", &d, &ExportSettings { copyright: c.clone(), ..settings(false, false) }, &brand()).unwrap();
        assert_eq!(read(&no_wm_with_meta[0])[0], ("Copyright".into(), "© 2026 Norélias Photography — Tous droits réservés".into()), "copyright présent sans filigrane");
        let wm_no_meta = export_one(&img(), "b", &d, &settings(true, false), &brand()).unwrap();
        assert!(read(&wm_no_meta[0]).is_empty(), "filigrane sans métadonnée de copyright");
        let _ = std::fs::remove_dir_all(&d);
    }
    #[test]
    fn jpeg_export_with_xmp_sidecar_and_quality() {
        let d = tmp("jpg");
        let mut s = ExportSettings { format: Format::Jpeg { quality: 90 }, copyright: Some(CopyrightMeta { notice: "© 2026 A & B".into(), url: None }), ..settings(true, false) };
        let hi = export_one(&img(), "a", &d, &s, &brand()).unwrap();
        s.format = Format::Jpeg { quality: 20 };
        let lo = export_one(&img(), "b", &d, &s, &brand()).unwrap();
        assert!(std::fs::metadata(&hi[0]).unwrap().len() > std::fs::metadata(&lo[0]).unwrap().len());
        let xmp = std::fs::read_to_string(d.join("a.xmp")).unwrap();
        assert!(xmp.contains("© 2026 A &amp; B") && xmp.contains("dc:rights"));
        assert_eq!(image::open(&hi[0]).unwrap().to_rgb8().dimensions(), (640, 420));
        let _ = std::fs::remove_dir_all(&d);
    }
    #[test]
    fn resize_and_unicode_names_and_hostile_names() {
        let d = tmp("names");
        let s = ExportSettings { max_side: Some(200), ..settings(false, false) };
        let f = export_one(&img(), "Norélias_日本語 №1", &d, &s, &brand()).unwrap();
        assert!(f[0].file_name().unwrap().to_string_lossy().starts_with("Norélias_日本語 №1"));
        assert_eq!(image::open(&f[0]).unwrap().to_rgb8().width(), 200_u32.min(640));
        let f = export_one(&img(), "../../etc/passwd", &d, &settings(false, false), &brand()).unwrap();
        assert_eq!(f[0].parent().unwrap(), d, "aucune évasion hors du dossier");
        for bad in ["", "   ", "CON", "nul.txt", "...", "."] { assert!(sanitize_filename(bad).is_none(), "{bad:?}"); }
        assert_eq!(sanitize_filename("a:b*c?.png").unwrap(), "a_b_c_.png");
        let _ = std::fs::remove_dir_all(&d);
    }
    #[test]
    fn batch_applies_to_every_image_and_isolates_failures() {
        let d = tmp("batch");
        let items = vec![("p1".to_string(), synth::scene(300, 200, 0.0)), ("p2".to_string(), synth::scene(200, 300, 0.0)), ("CON".to_string(), synth::scene(100, 100, 0.0))];
        let r = export_batch(&items, &d, &settings(true, false), &brand());
        assert!(r[0].is_ok() && r[1].is_ok() && matches!(r[2], Err(ExportError::BadName)));
        assert_ne!(decode(&d.join("p2.png")), items[1].1.to_srgb8());
        let _ = std::fs::remove_dir_all(&d);
    }
}
