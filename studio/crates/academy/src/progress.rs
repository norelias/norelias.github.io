//! Suivi de progression **facultatif** : rien n'est enregistré si l'utilisateur ne l'a pas activé.

use norelias_onboarding::profile::Profile;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::{fs, io::Write, path::PathBuf};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Progress {
    pub lessons_done: BTreeSet<String>,
    pub exercises_passed: BTreeSet<String>,
}

impl Progress {
    /// Enregistre une leçon terminée. Retourne `false` (et ne retient rien) si le suivi est désactivé.
    pub fn record_lesson(&mut self, profile: &Profile, id: &str) -> bool {
        if !profile.track_progress || crate::lessons::get(id).is_none() { return false; }
        self.lessons_done.insert(id.to_string());
        true
    }
    pub fn record_exercise(&mut self, profile: &Profile, id: &str) -> bool {
        if !profile.track_progress || crate::exercises::get(id).is_none() { return false; }
        self.exercises_passed.insert(id.to_string());
        true
    }
    /// Pourcentage du parcours recommandé terminé.
    pub fn percent_of(&self, path: &[&crate::lessons::Lesson]) -> u32 {
        if path.is_empty() { return 0; }
        (path.iter().filter(|l| self.lessons_done.contains(l.id)).count() * 100 / path.len()) as u32
    }
}

pub struct ProgressStore { path: PathBuf }
impl ProgressStore {
    pub fn new(dir: impl Into<PathBuf>) -> Self { Self { path: dir.into().join("academy.json") } }
    pub fn load(&self) -> Progress { fs::read(&self.path).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default() }
    /// N'écrit rien si le suivi est désactivé.
    pub fn save(&self, profile: &Profile, p: &Progress) -> std::io::Result<bool> {
        if !profile.track_progress { return Ok(false); }
        if let Some(d) = self.path.parent() { fs::create_dir_all(d)?; }
        let tmp = self.path.with_extension("json.tmp");
        { let mut f = fs::File::create(&tmp)?; f.write_all(serde_json::to_string_pretty(p)?.as_bytes())?; f.sync_all()?; }
        fs::rename(tmp, &self.path)?;
        Ok(true)
    }
    /// Efface tout l'historique de progression.
    pub fn erase(&self) -> std::io::Result<()> { if self.path.exists() { fs::remove_file(&self.path)?; } Ok(()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nothing_is_recorded_or_written_when_tracking_is_off() {
        let dir = std::env::temp_dir().join(format!("norelias-progress-off-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let p = Profile::default();
        assert!(!p.track_progress);
        let mut pr = Progress::default();
        assert!(!pr.record_lesson(&p, "exposure"));
        assert!(!pr.record_exercise(&p, "ex_exposure"));
        assert_eq!(pr, Progress::default());
        let s = ProgressStore::new(&dir);
        assert!(!s.save(&p, &pr).unwrap());
        assert!(!dir.join("academy.json").exists());
    }
    #[test]
    fn records_persist_when_enabled_and_can_be_erased() {
        let dir = std::env::temp_dir().join(format!("norelias-progress-on-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let mut p = Profile::default();
        p.track_progress = true;
        let mut pr = Progress::default();
        assert!(pr.record_lesson(&p, "exposure"));
        assert!(!pr.record_lesson(&p, "inexistante"));
        let s = ProgressStore::new(&dir);
        assert!(s.save(&p, &pr).unwrap());
        assert_eq!(s.load(), pr);
        s.erase().unwrap();
        assert_eq!(s.load(), Progress::default());
        let _ = fs::remove_dir_all(&dir);
    }
    #[test]
    fn percent() {
        let mut p = Profile::default();
        p.track_progress = true;
        let mut pr = Progress::default();
        let path = crate::lessons::path_for(&p);
        assert_eq!(pr.percent_of(&path), 0);
        for l in &path { pr.record_lesson(&p, l.id); }
        assert_eq!(pr.percent_of(&path), 100);
    }
}
