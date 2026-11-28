//! Profils de raccourcis clavier « familiers ». Un raccourci est une convention d'usage,
//! pas une ressource protégée : aucune icône, disposition d'interface, nom de commande ni
//! texte propriétaire n'est repris. Couverture **partielle** : une commande non redéfinie par un
//! profil retombe sur la disposition Norélias (voir FEATURE_MATRIX.md).

use crate::profile::ShortcutProfileId;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Command {
    Undo, Redo, Crop, ZoomFit, Zoom100, BeforeAfter, TogglePanels, GridView, LoupeView, DevelopView,
    Rate0, Rate1, Rate2, Rate3, Rate4, Rate5, Pick, Reject, CopySettings, PasteSettings, Export, SearchTools, OpenAcademy,
}
impl Command {
    pub const ALL: [Command; 23] = [
        Command::Undo, Command::Redo, Command::Crop, Command::ZoomFit, Command::Zoom100, Command::BeforeAfter,
        Command::TogglePanels, Command::GridView, Command::LoupeView, Command::DevelopView, Command::Rate0,
        Command::Rate1, Command::Rate2, Command::Rate3, Command::Rate4, Command::Rate5, Command::Pick, Command::Reject,
        Command::CopySettings, Command::PasteSettings, Command::Export, Command::SearchTools, Command::OpenAcademy,
    ];
}

/// Combinaison normalisée : modificateurs dans l'ordre `Mod+Alt+Shift+Touche` (Mod = Ctrl ou ⌘).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Chord(String);

#[derive(Debug, PartialEq, Eq)]
pub enum ChordError { Empty, UnknownModifier(String), MissingKey }

impl Chord {
    pub fn parse(s: &str) -> Result<Chord, ChordError> {
        let parts: Vec<&str> = s.split('+').map(str::trim).collect();
        if parts.iter().all(|p| p.is_empty()) { return Err(ChordError::Empty); }
        let (key, mods) = parts.split_last().unwrap();
        if key.is_empty() { return Err(ChordError::MissingKey); }
        let (mut m, mut a, mut sh) = (false, false, false);
        for md in mods {
            match md.to_lowercase().as_str() {
                "mod" | "ctrl" | "cmd" | "control" => m = true,
                "alt" | "option" => a = true,
                "shift" => sh = true,
                other => return Err(ChordError::UnknownModifier(other.to_string())),
            }
        }
        let mut out = String::new();
        if m { out.push_str("Mod+"); }
        if a { out.push_str("Alt+"); }
        if sh { out.push_str("Shift+"); }
        out.push_str(&if key.chars().count() == 1 { key.to_uppercase() } else { key.to_string() });
        Ok(Chord(out))
    }
    pub fn as_str(&self) -> &str { &self.0 }
}

fn base() -> Vec<(Command, &'static str)> {
    use Command::*;
    vec![
        (Undo, "Mod+Z"), (Redo, "Mod+Shift+Z"), (Crop, "C"), (ZoomFit, "Mod+0"), (Zoom100, "Mod+1"), (BeforeAfter, "\\"),
        (TogglePanels, "Tab"), (GridView, "G"), (LoupeView, "E"), (DevelopView, "D"),
        (Rate0, "0"), (Rate1, "1"), (Rate2, "2"), (Rate3, "3"), (Rate4, "4"), (Rate5, "5"), (Pick, "P"), (Reject, "X"),
        (CopySettings, "Mod+Shift+C"), (PasteSettings, "Mod+Shift+V"), (Export, "Mod+Shift+E"), (SearchTools, "Mod+K"), (OpenAcademy, "F1"),
    ]
}

/// Écarts par rapport à la disposition de base, selon les conventions usuelles de chaque famille de logiciels.
fn overrides(id: ShortcutProfileId) -> Vec<(Command, &'static str)> {
    use Command::*;
    match id {
        ShortcutProfileId::Norelias => vec![],
        // Photoshop : annulation par pas = Ctrl+Alt+Z, rétablir = Ctrl+Shift+Z, recadrage = C.
        ShortcutProfileId::PhotoshopLike => vec![(Undo, "Mod+Alt+Z"), (ZoomFit, "Mod+0"), (Zoom100, "Mod+1"), (Export, "Mod+Alt+Shift+S")],
        // Lightroom : G grille, E loupe, D développement, R recadrage, \ avant/après.
        ShortcutProfileId::LightroomLike => vec![(Crop, "R"), (Export, "Mod+Shift+E"), (CopySettings, "Mod+Shift+C"), (PasteSettings, "Mod+Shift+V")],
        // Capture One : notes 0–5 au clavier, Ctrl+Z / Ctrl+Shift+Z ; couverture volontairement minimale.
        ShortcutProfileId::CaptureOneLike => vec![(Undo, "Mod+Z"), (Redo, "Mod+Shift+Z")],
    }
}

/// Nombre de commandes redéfinies par un profil (indicateur de couverture).
pub fn override_count(id: ShortcutProfileId) -> usize {
    let b: BTreeMap<_, _> = base().into_iter().collect();
    overrides(id).into_iter().filter(|(c, k)| b.get(c) != Some(k)).count()
}

#[derive(Debug, PartialEq, Eq)]
pub enum RebindError { Conflict(Command), Chord(ChordError) }

#[derive(Clone, Debug)]
pub struct ShortcutMap { map: BTreeMap<Command, Chord> }

impl ShortcutMap {
    pub fn for_profile(id: ShortcutProfileId) -> Self {
        let mut map: BTreeMap<Command, Chord> = base().into_iter().map(|(c, k)| (c, Chord::parse(k).unwrap())).collect();
        for (c, k) in overrides(id) { map.insert(c, Chord::parse(k).unwrap()); }
        Self { map }
    }
    pub fn get(&self, c: Command) -> Option<&Chord> { self.map.get(&c) }
    pub fn lookup(&self, chord: &Chord) -> Option<Command> { self.map.iter().find(|(_, k)| *k == chord).map(|(c, _)| *c) }
    /// Personnalisation : refuse une combinaison déjà utilisée par une autre commande.
    pub fn rebind(&mut self, c: Command, chord: &str) -> Result<(), RebindError> {
        let ch = Chord::parse(chord).map_err(RebindError::Chord)?;
        if let Some(other) = self.lookup(&ch) { if other != c { return Err(RebindError::Conflict(other)); } }
        self.map.insert(c, ch);
        Ok(())
    }
    pub fn conflicts(&self) -> Vec<(Command, Command)> {
        let v: Vec<_> = self.map.iter().collect();
        let mut out = vec![];
        for i in 0..v.len() { for j in i + 1..v.len() { if v[i].1 == v[j].1 { out.push((*v[i].0, *v[j].0)); } } }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const IDS: [ShortcutProfileId; 4] = [ShortcutProfileId::Norelias, ShortcutProfileId::PhotoshopLike, ShortcutProfileId::LightroomLike, ShortcutProfileId::CaptureOneLike];

    #[test]
    fn every_profile_is_complete_and_conflict_free() {
        for id in IDS {
            let m = ShortcutMap::for_profile(id);
            for c in Command::ALL { assert!(m.get(c).is_some(), "{id:?} n'a pas de raccourci pour {c:?}"); }
            assert!(m.conflicts().is_empty(), "{id:?} : {:?}", m.conflicts());
        }
    }
    #[test]
    fn profiles_differ_where_conventions_differ() {
        let ps = ShortcutMap::for_profile(ShortcutProfileId::PhotoshopLike);
        let lr = ShortcutMap::for_profile(ShortcutProfileId::LightroomLike);
        assert_eq!(ps.get(Command::Undo).unwrap().as_str(), "Mod+Alt+Z");
        assert_eq!(lr.get(Command::Crop).unwrap().as_str(), "R");
        assert!(override_count(ShortcutProfileId::PhotoshopLike) > 0);
        assert_eq!(override_count(ShortcutProfileId::Norelias), 0);
    }
    #[test]
    fn chord_parsing_normalises() {
        assert_eq!(Chord::parse("shift+ctrl+z").unwrap().as_str(), "Mod+Shift+Z");
        assert_eq!(Chord::parse("Cmd+Alt+s").unwrap(), Chord::parse("mod+alt+S").unwrap());
        assert_eq!(Chord::parse(""), Err(ChordError::Empty));
        assert_eq!(Chord::parse("Mod+"), Err(ChordError::MissingKey));
        assert!(matches!(Chord::parse("Hyper+X"), Err(ChordError::UnknownModifier(_))));
    }
    #[test]
    fn rebinding_detects_conflicts() {
        let mut m = ShortcutMap::for_profile(ShortcutProfileId::Norelias);
        assert_eq!(m.rebind(Command::Crop, "G"), Err(RebindError::Conflict(Command::GridView)));
        m.rebind(Command::Crop, "Mod+Shift+K").unwrap();
        assert_eq!(m.lookup(&Chord::parse("mod+shift+k").unwrap()), Some(Command::Crop));
        m.rebind(Command::Crop, "Mod+Shift+K").unwrap(); // idempotent
    }
}
