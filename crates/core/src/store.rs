//! The note file. Everything lives in one JSON document that is rewritten
//! atomically (temp file + rename) after every change.

use serde::{Deserialize, Serialize};
use std::io;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// Something worth keeping: a captured selection or a typed note.
    Note,
    /// A prompt queued for later, checked off once used.
    Prompt,
}

/// A color label on a note or prompt.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Color {
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Purple,
}

impl Color {
    pub const ALL: [Color; 6] = [Color::Red, Color::Orange, Color::Yellow, Color::Green, Color::Blue, Color::Purple];

    pub fn name(self) -> &'static str {
        match self {
            Color::Red => "Red",
            Color::Orange => "Orange",
            Color::Yellow => "Yellow",
            Color::Green => "Green",
            Color::Blue => "Blue",
            Color::Purple => "Purple",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub id: u64,
    pub kind: Kind,
    pub text: String,
    /// The app the selection came from, for captured items.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Unix seconds.
    pub created: i64,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub done: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Appearance {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Theme name, e.g. "Copper".
    pub theme: String,
    pub appearance: Appearance,
    /// Hide note text (peek with ⌥) and keep the window out of screenshots
    /// and screen sharing.
    pub sensitive: bool,
    /// Play a sound when a selection is saved.
    pub sound: bool,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings { theme: "Copper".into(), appearance: Appearance::System, sensitive: false, sound: true }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Data {
    #[serde(default)]
    settings: Settings,
    #[serde(default)]
    next_id: u64,
    /// Oldest first; the UI shows them newest first.
    #[serde(default)]
    items: Vec<Item>,
    #[serde(default)]
    scratch: String,
}

pub struct Store {
    path: PathBuf,
    data: Data,
}

impl Store {
    /// `~/Library/Application Support/dev.rjot.rjot/notes.json` on macOS.
    pub fn default_path() -> PathBuf {
        directories::ProjectDirs::from("dev", "rjot", "rjot")
            .map(|d| d.data_dir().join("notes.json"))
            .unwrap_or_else(|| PathBuf::from("notes.json"))
    }

    /// Open the file at `path`; a missing file is an empty store. A file
    /// that can't be parsed is moved aside (never overwritten) and the
    /// error returned with the backup's path.
    pub fn open(path: impl Into<PathBuf>) -> Result<Store, String> {
        let path = path.into();
        let data = match std::fs::read(&path) {
            Ok(bytes) => match serde_json::from_slice::<Data>(&bytes) {
                Ok(d) => d,
                Err(e) => {
                    let backup = path.with_extension("json.broken");
                    let _ = std::fs::rename(&path, &backup);
                    return Err(format!("{} could not be read ({e}); moved to {}", path.display(), backup.display()));
                }
            },
            Err(e) if e.kind() == io::ErrorKind::NotFound => Data::default(),
            Err(e) => return Err(format!("{}: {e}", path.display())),
        };
        let mut store = Store { path, data };
        let max = store.data.items.iter().map(|i| i.id).max().unwrap_or(0);
        store.data.next_id = store.data.next_id.max(max + 1);
        Ok(store)
    }

    /// An empty store at `path`, used when the file couldn't be opened.
    pub fn empty(path: impl Into<PathBuf>) -> Store {
        Store { path: path.into(), data: Data { next_id: 1, ..Data::default() } }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn save(&self) -> io::Result<()> {
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let tmp = self.path.with_extension("json.tmp");
        let bytes = serde_json::to_vec_pretty(&self.data).map_err(io::Error::other)?;
        std::fs::write(&tmp, bytes)?;
        std::fs::rename(&tmp, &self.path)
    }

    /// Items of `kind`, newest first.
    pub fn items(&self, kind: Kind) -> impl Iterator<Item = &Item> {
        self.data.items.iter().rev().filter(move |i| i.kind == kind)
    }

    pub fn count(&self, kind: Kind) -> usize {
        self.data.items.iter().filter(|i| i.kind == kind && !i.done).count()
    }

    pub fn get(&self, id: u64) -> Option<&Item> {
        self.data.items.iter().find(|i| i.id == id)
    }

    /// Add an item and return its id. Blank text is ignored. Saving the same
    /// note twice in a row within a few seconds (a double trigger) keeps one.
    pub fn add(&mut self, kind: Kind, text: &str, source: Option<String>, now: i64) -> Option<u64> {
        let text = text.trim_matches(|c: char| c == '\n' || c == '\r');
        if text.trim().is_empty() {
            return None;
        }
        if let Some(last) = self.data.items.last()
            && last.kind == kind
            && last.text == text
            && now - last.created < 5
        {
            return Some(last.id);
        }
        let id = self.data.next_id.max(1);
        self.data.next_id = id + 1;
        self.data.items.push(Item { id, kind, text: text.to_string(), source, created: now, done: false, color: None });
        Some(id)
    }

    /// Remove an item, returning it and its position so it can be restored.
    pub fn remove(&mut self, id: u64) -> Option<(usize, Item)> {
        let pos = self.data.items.iter().position(|i| i.id == id)?;
        Some((pos, self.data.items.remove(pos)))
    }

    /// Put back an item taken out with [`Store::remove`].
    pub fn restore(&mut self, pos: usize, item: Item) {
        let pos = pos.min(self.data.items.len());
        self.data.items.insert(pos, item);
    }

    fn get_mut(&mut self, id: u64) -> Option<&mut Item> {
        self.data.items.iter_mut().find(|i| i.id == id)
    }

    pub fn set_done(&mut self, id: u64, done: bool) {
        if let Some(i) = self.get_mut(id) {
            i.done = done;
        }
    }

    pub fn set_text(&mut self, id: u64, text: &str) {
        if let Some(i) = self.get_mut(id) {
            i.text = text.to_string();
        }
    }

    pub fn set_color(&mut self, id: u64, color: Option<Color>) {
        if let Some(i) = self.get_mut(id) {
            i.color = color;
        }
    }

    /// Move an item to the other list. It keeps its age but loses `done`.
    pub fn set_kind(&mut self, id: u64, kind: Kind) {
        if let Some(i) = self.get_mut(id) {
            i.kind = kind;
            i.done = false;
        }
    }

    /// Delete checked-off prompts. Returns them with their positions, in
    /// order, so restoring them one by one puts each back where it was.
    pub fn clear_done(&mut self) -> Vec<(usize, Item)> {
        let mut removed = Vec::new();
        let mut kept = Vec::with_capacity(self.data.items.len());
        for (pos, item) in std::mem::take(&mut self.data.items).into_iter().enumerate() {
            if item.kind == Kind::Prompt && item.done { removed.push((pos, item)) } else { kept.push(item) }
        }
        self.data.items = kept;
        removed
    }

    pub fn settings(&self) -> &Settings {
        &self.data.settings
    }

    pub fn settings_mut(&mut self) -> &mut Settings {
        &mut self.data.settings
    }

    pub fn scratch(&self) -> &str {
        &self.data.scratch
    }

    pub fn set_scratch(&mut self, text: String) {
        self.data.scratch = text;
    }
}

/// Case-insensitive match of every whitespace-separated word in `query`
/// against the item's text and source.
pub fn matches(item: &Item, query: &str) -> bool {
    let hay = format!("{} {}", item.text, item.source.as_deref().unwrap_or("")).to_lowercase();
    query.split_whitespace().all(|w| hay.contains(&w.to_lowercase()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("rjot-test-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir.join("notes.json")
    }

    #[test]
    fn round_trip() {
        let path = temp_path("round");
        let mut s = Store::open(&path).unwrap();
        let a = s.add(Kind::Note, "hello", Some("Safari".into()), 100).unwrap();
        let b = s.add(Kind::Prompt, "ask about x", None, 101).unwrap();
        s.set_done(b, true);
        s.set_color(a, Some(Color::Green));
        s.set_scratch("draft".into());
        s.settings_mut().sensitive = true;
        s.save().unwrap();

        let s = Store::open(&path).unwrap();
        assert_eq!(s.get(a).unwrap().source.as_deref(), Some("Safari"));
        assert_eq!(s.get(a).unwrap().color, Some(Color::Green));
        assert_eq!(s.get(b).unwrap().color, None);
        assert!(s.get(b).unwrap().done);
        assert_eq!(s.scratch(), "draft");
        assert!(s.settings().sensitive);
        assert_eq!(s.settings().theme, "Copper");
        assert_eq!(s.count(Kind::Prompt), 0);
        assert_eq!(s.items(Kind::Note).count(), 1);
    }

    #[test]
    fn files_without_settings_get_defaults() {
        let path = temp_path("old");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, br#"{"next_id":2,"items":[],"scratch":"","settings":{"theme":"Sage"}}"#).unwrap();
        let s = Store::open(&path).unwrap();
        assert_eq!(s.settings().theme, "Sage");
        assert!(s.settings().sound);
        assert_eq!(s.settings().appearance, Appearance::System);
    }

    #[test]
    fn ids_continue_after_reload() {
        let path = temp_path("ids");
        let mut s = Store::open(&path).unwrap();
        let a = s.add(Kind::Note, "one", None, 1).unwrap();
        s.save().unwrap();
        let mut s = Store::open(&path).unwrap();
        let b = s.add(Kind::Note, "two", None, 2).unwrap();
        assert!(b > a);
    }

    #[test]
    fn blank_and_duplicate_captures() {
        let mut s = Store::empty("x.json");
        assert_eq!(s.add(Kind::Note, "  \n ", None, 0), None);
        let a = s.add(Kind::Note, "same\n", None, 10).unwrap();
        assert_eq!(s.add(Kind::Note, "same", None, 12), Some(a));
        assert_ne!(s.add(Kind::Note, "same", None, 30), Some(a));
        assert_eq!(s.get(a).unwrap().text, "same");
    }

    #[test]
    fn remove_restore_and_clear_done() {
        let mut s = Store::empty("x.json");
        let a = s.add(Kind::Prompt, "a", None, 0).unwrap();
        let b = s.add(Kind::Prompt, "b", None, 1).unwrap();
        let (pos, item) = s.remove(a).unwrap();
        s.restore(pos, item);
        assert_eq!(s.items(Kind::Prompt).map(|i| i.id).collect::<Vec<_>>(), vec![b, a]);
        s.set_done(a, true);
        let removed = s.clear_done();
        assert_eq!(removed.len(), 1);
        assert!(s.get(a).is_none());
        for (pos, item) in removed {
            s.restore(pos, item);
        }
        assert_eq!(s.items(Kind::Prompt).map(|i| i.id).collect::<Vec<_>>(), vec![b, a]);
    }

    #[test]
    fn broken_file_is_kept() {
        let path = temp_path("broken");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, b"{not json").unwrap();
        assert!(Store::open(&path).is_err());
        assert!(path.with_extension("json.broken").exists());
    }

    #[test]
    fn search() {
        let item = Item { id: 1, kind: Kind::Note, text: "Rust borrow checker".into(), source: Some("Claude".into()), created: 0, done: false, color: None };
        assert!(matches(&item, "borrow claude"));
        assert!(matches(&item, ""));
        assert!(!matches(&item, "python"));
    }
}
