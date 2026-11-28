//! Modèles de filigrane nommés (plusieurs, modifiables, supprimables), stockés localement.

use crate::watermark::Watermark;
use std::collections::BTreeMap;
use std::path::PathBuf;

pub struct TemplateStore { dir: PathBuf }

#[derive(Debug, PartialEq, Eq)]
pub enum TemplateError { BadName, Io(String) }

fn clean(name: &str) -> Result<String, TemplateError> {
    let n = name.trim();
    if n.is_empty() || n.chars().count() > 60 || n.chars().any(|c| c.is_control()) { Err(TemplateError::BadName) } else { Ok(n.to_string()) }
}

impl TemplateStore {
    pub fn new(dir: impl Into<PathBuf>) -> Self { Self { dir: dir.into() } }
    fn all(&self) -> Result<BTreeMap<String, Watermark>, TemplateError> {
        match std::fs::read(self.dir.join("watermarks.json")) {
            Ok(b) => serde_json::from_slice(&b).map_err(|e| TemplateError::Io(e.to_string())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(BTreeMap::new()),
            Err(e) => Err(TemplateError::Io(e.to_string())),
        }
    }
    fn write(&self, m: &BTreeMap<String, Watermark>) -> Result<(), TemplateError> {
        let j = serde_json::to_string_pretty(m).map_err(|e| TemplateError::Io(e.to_string()))?;
        crate::profile::atomic(&self.dir, "watermarks.json", j.as_bytes()).map_err(|e| TemplateError::Io(e.to_string()))
    }
    pub fn list(&self) -> Result<Vec<String>, TemplateError> { Ok(self.all()?.into_keys().collect()) }
    pub fn save(&self, name: &str, wm: &Watermark) -> Result<(), TemplateError> { let mut m = self.all()?; m.insert(clean(name)?, wm.clone()); self.write(&m) }
    pub fn load(&self, name: &str) -> Result<Option<Watermark>, TemplateError> { Ok(self.all()?.get(name.trim()).cloned()) }
    pub fn delete(&self, name: &str) -> Result<bool, TemplateError> { let mut m = self.all()?; let had = m.remove(name.trim()).is_some(); if had { self.write(&m)?; } Ok(had) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::watermark::*;
    #[test]
    fn multiple_templates_roundtrip() {
        let d = std::env::temp_dir().join(format!("norelias-tpl-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        let s = TemplateStore::new(&d);
        assert!(s.list().unwrap().is_empty());
        let a = Watermark { opacity: 0.4, tile_gap_pct: Some(10.0), ..Default::default() };
        let b = Watermark { content: Content::Text { source: TextSource::Custom("© {year} Éloïse".into()), font: FontStyle::Sans, color: [10, 20, 30] },
            shadow: Some(Shadow { dx_pct: 0.2, dy_pct: 0.2, blur_pct: 0.3, color: [0; 3], opacity: 0.5 }), ..Default::default() };
        s.save("Discret — réseaux", &a).unwrap();
        s.save("Signature", &b).unwrap();
        assert_eq!(s.list().unwrap(), vec!["Discret — réseaux", "Signature"]);
        assert_eq!(s.load("Signature").unwrap().unwrap(), b);
        assert!(s.delete("Signature").unwrap() && !s.delete("Signature").unwrap());
        assert_eq!(s.list().unwrap().len(), 1);
        assert_eq!(s.save("  ", &a), Err(TemplateError::BadName));
        let _ = std::fs::remove_dir_all(&d);
    }
}
