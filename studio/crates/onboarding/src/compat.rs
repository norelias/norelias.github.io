//! Compatibilité avec les anciens logiciels : importation de presets XMP et reconnaissance de
//! formats de catalogue. **Rien n'est promis au-delà de ce qui est testé** : chaque réglage non converti
//! est listé avec sa raison dans le rapport de migration.
//!
//! La conversion est une *approximation visuelle* : les algorithmes d'origine sont propriétaires et
//! les rendus ne sont pas identiques pixel à pixel. Aucune ressource propriétaire n'est incluse.

use norelias_core::Op;

#[derive(Debug, Default, PartialEq)]
pub struct MigrationReport {
    pub ops: Vec<Op>,
    /// (réglage source, opération créée)
    pub mapped: Vec<(String, String)>,
    /// (réglage source, raison)
    pub unmapped: Vec<(String, String)>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum XmpError { NotXmp, NoSettings }

const METADATA_ONLY: &[&str] = &[
    "Version", "ProcessVersion", "HasSettings", "PresetType", "Cluster", "UUID", "SupportsAmount", "SupportsColor",
    "SupportsMonochrome", "SupportsHighDynamicRange", "SupportsNormalDynamicRange", "SupportsSceneReferred",
    "SupportsOutputReferred", "CameraModelRestriction", "Copyright", "ContactInfo", "ToneCurveName2012",
    "ConvertToGrayscale", "AlreadyApplied", "CameraProfile", "WhiteBalance",
];

/// Lit les attributs `crs:Nom="valeur"` d'un fichier XMP de preset.
pub fn import_xmp_preset(xml: &str) -> Result<MigrationReport, XmpError> {
    if !xml.contains("<x:xmpmeta") && !xml.contains("<rdf:RDF") { return Err(XmpError::NotXmp); }
    let mut attrs: Vec<(String, String)> = vec![];
    let mut rest = xml;
    while let Some(i) = rest.find("crs:") {
        rest = &rest[i + 4..];
        let name_end = rest.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).unwrap_or(rest.len());
        let name = &rest[..name_end];
        let after = &rest[name_end..];
        if let Some(a) = after.strip_prefix("=\"") {
            if let Some(end) = a.find('"') { attrs.push((name.to_string(), a[..end].to_string())); }
        }
    }
    if attrs.is_empty() { return Err(XmpError::NoSettings); }

    let mut r = MigrationReport::default();
    let num = |v: &str| v.trim().trim_start_matches('+').parse::<f32>().ok();
    for (name, val) in attrs {
        if METADATA_ONLY.contains(&name.as_str()) { continue; }
        let push = |r: &mut MigrationReport, op: Op| { r.mapped.push((name.clone(), op.name().to_string())); r.ops.push(op); };
        let reason = |s: &str| (name.clone(), s.to_string());
        let Some(v) = num(&val) else { r.unmapped.push(reason("valeur non numérique")); continue };
        match name.as_str() {
            "Exposure2012" | "Exposure" => push(&mut r, Op::Exposure { ev: v }),
            "Contrast2012" | "Contrast" => push(&mut r, Op::Contrast { amount: (v / 100.0 * 0.5).clamp(-0.5, 0.5) }),
            "Saturation" => push(&mut r, Op::Saturation { factor: (1.0 + v / 100.0).max(0.0) }),
            "Highlights2012" if v < 0.0 => push(&mut r, Op::Highlights { amount: (-v / 100.0).clamp(0.0, 1.0) }),
            "Highlights2012" => r.unmapped.push(reason("éclaircissement des hautes lumières : pas d'équivalent intégré")),
            "Shadows2012" if v > 0.0 => push(&mut r, Op::Shadows { amount: (v / 100.0).clamp(0.0, 1.0) }),
            "Shadows2012" => r.unmapped.push(reason("assombrissement des ombres : pas d'équivalent intégré")),
            "Sharpness" => push(&mut r, Op::Sharpen { amount: (v / 150.0).clamp(0.0, 1.0) }),
            "LuminanceSmoothing" => push(&mut r, Op::Denoise { strength: (v / 100.0).clamp(0.0, 1.0) }),
            "Temperature" | "Tint" => r.unmapped.push(reason("dépend du profil de l'appareil et du fichier RAW : conversion impossible sans développement RAW")),
            "Vibrance" | "Clarity2012" | "Texture" | "Dehaze" | "Whites2012" | "Blacks2012" =>
                r.unmapped.push(reason("outil équivalent pas encore intégré")),
            n if n.starts_with("Parametric") || n.starts_with("ToneCurve") => r.unmapped.push(reason("courbes : outil pas encore intégré")),
            n if n.contains("Hue") || n.contains("Saturation") || n.contains("Luminance") => r.unmapped.push(reason("HSL/mélangeur : outil pas encore intégré")),
            n if n.starts_with("Vignette") || n.starts_with("Grain") || n.starts_with("Lens") => r.unmapped.push(reason("effet pas encore intégré")),
            _ => r.unmapped.push(reason("réglage inconnu ou sans équivalent")),
        }
    }
    Ok(r)
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ForeignFormat { LightroomCatalog, CaptureOneCatalog, Psd, Unknown }

/// Reconnaissance par extension et signature. Reconnaître n'est pas importer : l'import de catalogue
/// est **prévu** (voir FEATURE_MATRIX.md) ; cette fonction permet d'afficher un diagnostic honnête.
pub fn detect_format(file_name: &str, header: &[u8]) -> ForeignFormat {
    let ext = file_name.rsplit('.').next().unwrap_or("").to_lowercase();
    if header.starts_with(b"8BPS") { return ForeignFormat::Psd; }
    match ext.as_str() {
        "lrcat" if header.starts_with(b"SQLite format 3\0") => ForeignFormat::LightroomCatalog,
        "cocatalog" | "cocatalogdb" => ForeignFormat::CaptureOneCatalog,
        _ => ForeignFormat::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const SAMPLE: &str = r#"<x:xmpmeta xmlns:x="adobe:ns:meta/"><rdf:RDF><rdf:Description
   crs:Version="15.0" crs:ProcessVersion="11.0" crs:HasSettings="True"
   crs:Exposure2012="+0.50" crs:Contrast2012="+20" crs:Highlights2012="-40" crs:Shadows2012="+30"
   crs:Saturation="-10" crs:Vibrance="+25" crs:Temperature="5500" crs:Sharpness="60" crs:LuminanceSmoothing="25"
   crs:GrainAmount="20" crs:Clarity2012="+10"/></rdf:RDF></x:xmpmeta>"#;

    #[test]
    fn imports_supported_settings_as_real_ops() {
        let r = import_xmp_preset(SAMPLE).unwrap();
        assert!(r.ops.contains(&Op::Exposure { ev: 0.5 }));
        assert!(r.ops.contains(&Op::Highlights { amount: 0.4 }));
        assert!(r.ops.contains(&Op::Shadows { amount: 0.3 }));
        assert!(r.ops.iter().any(|o| matches!(o, Op::Saturation { factor } if (*factor - 0.9).abs() < 1e-6)));
        assert!(r.ops.iter().any(|o| matches!(o, Op::Denoise { strength } if (*strength - 0.25).abs() < 1e-6)));
        assert_eq!(r.mapped.len(), r.ops.len());
    }
    #[test]
    fn unsupported_settings_are_reported_with_reasons_not_silently_dropped() {
        let r = import_xmp_preset(SAMPLE).unwrap();
        let names: Vec<_> = r.unmapped.iter().map(|u| u.0.as_str()).collect();
        for n in ["Vibrance", "Temperature", "GrainAmount", "Clarity2012"] { assert!(names.contains(&n), "{n} absent du rapport : {names:?}"); }
        assert!(r.unmapped.iter().all(|u| !u.1.is_empty()));
        assert!(!names.contains(&"Version"), "les métadonnées ne sont pas des réglages");
    }
    #[test]
    fn converted_preset_really_changes_pixels() {
        let r = import_xmp_preset(SAMPLE).unwrap();
        let img = norelias_core::synth::scene(64, 48, 0.0);
        let out = norelias_core::ops::apply_stack(&img, &r.ops);
        assert_ne!(out, img);
    }
    #[test]
    fn rejects_non_xmp_and_empty() {
        assert_eq!(import_xmp_preset("hello"), Err(XmpError::NotXmp));
        assert_eq!(import_xmp_preset("<x:xmpmeta></x:xmpmeta>"), Err(XmpError::NoSettings));
    }
    #[test]
    fn detects_foreign_formats_by_signature() {
        assert_eq!(detect_format("studio.lrcat", b"SQLite format 3\0abc"), ForeignFormat::LightroomCatalog);
        assert_eq!(detect_format("studio.lrcat", b"garbage"), ForeignFormat::Unknown);
        assert_eq!(detect_format("a.psd", b"8BPS\0\x01"), ForeignFormat::Psd);
        assert_eq!(detect_format("x.cocatalog", b""), ForeignFormat::CaptureOneCatalog);
    }
}
