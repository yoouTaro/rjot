//! A Markdown copy of everything, for Obsidian and friends.
//!
//! rjot owns one folder, `<chosen dir>/rjot/`: `Notes/`, `Queue/` and
//! `Archive/` hold one `.md` file per item (with front matter), `images/`
//! the captured images, and `Scratch.md` the scratchpad. Files are only
//! rewritten when their content changed, and the only files ever deleted
//! are `.md` files rjot wrote itself (they carry an `rjot_id:` line).

use crate::store::{Item, Kind};
use crate::time;
use std::collections::HashSet;
use std::io;
use std::path::{Path, PathBuf};

/// The folder rjot writes into, inside the one the user chose.
pub fn root(chosen: &Path) -> PathBuf {
    chosen.join("rjot")
}

/// A snapshot to write, taken under the store lock and run outside it.
pub struct Job {
    pub dir: PathBuf,
    pub items: Vec<Item>,
    pub scratch: String,
    pub images: PathBuf,
}

impl Job {
    pub fn run(&self) -> io::Result<()> {
        export(&self.dir, &self.items, &self.scratch, &self.images)
    }
}

/// Paths compared the way APFS does by default: ignoring case.
fn key(path: &Path) -> String {
    path.to_string_lossy().to_lowercase()
}

pub fn export(chosen: &Path, items: &[Item], scratch: &str, images_src: &Path) -> io::Result<()> {
    let root = root(chosen);
    let folders = ["Notes", "Queue", "Archive"];
    for f in folders {
        std::fs::create_dir_all(root.join(f))?;
    }
    let mut written: HashSet<String> = HashSet::new();
    for item in items {
        let folder = if item.archived {
            "Archive"
        } else if item.kind == Kind::Note {
            "Notes"
        } else {
            "Queue"
        };
        let path = root.join(folder).join(file_name(item));
        write_if_changed(&path, &markdown(item))?;
        written.insert(key(&path));
        if let Some(image) = &item.image {
            let dest = root.join("images").join(image);
            if !dest.exists() {
                std::fs::create_dir_all(root.join("images"))?;
                // A missing source image just leaves the link dangling.
                let _ = std::fs::copy(images_src.join(image), &dest);
            }
        }
    }
    write_if_changed(&root.join("Scratch.md"), scratch)?;

    // Drop files of items that were deleted, moved or renamed.
    for f in folders {
        for entry in std::fs::read_dir(root.join(f))?.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "md") && !written.contains(&key(&path)) && ours(&path) {
                std::fs::remove_file(&path)?;
            }
        }
    }
    Ok(())
}

/// `<first words> (<id>).md`; the id keeps names unique and stable.
pub fn file_name(item: &Item) -> String {
    let words: String = item
        .text
        .split_whitespace()
        .take(8)
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .filter(|c| c.is_alphanumeric() || " -_'".contains(*c))
        .collect();
    let mut stem: String = words.trim().chars().take(60).collect();
    if stem.is_empty() {
        stem = if item.image.is_some() { "Image".into() } else { "Note".into() };
    }
    format!("{} ({}).md", stem.trim_end(), item.id)
}

pub fn markdown(item: &Item) -> String {
    let mut out = String::from("---\n");
    out.push_str(&format!("rjot_id: {}\n", item.id));
    out.push_str(&format!("kind: {}\n", if item.kind == Kind::Note { "note" } else { "prompt" }));
    out.push_str(&format!("created: {}\n", time::iso(item.created)));
    if let Some(s) = &item.source {
        out.push_str(&format!("source: {}\n", yaml(s)));
    }
    if let Some(t) = &item.title {
        out.push_str(&format!("title: {}\n", yaml(t)));
    }
    if let Some(u) = &item.url {
        out.push_str(&format!("url: {}\n", yaml(u)));
    }
    if let Some(c) = item.color {
        out.push_str(&format!("color: {}\n", c.name().to_lowercase()));
    }
    for (flag, name) in [(item.pinned, "pinned"), (item.done, "done"), (item.archived, "archived")] {
        if flag {
            out.push_str(&format!("{name}: true\n"));
        }
    }
    out.push_str("---\n\n");
    if !item.text.is_empty() {
        out.push_str(item.text.trim_end());
        out.push('\n');
    }
    if let Some(image) = &item.image {
        if !item.text.is_empty() {
            out.push('\n');
        }
        out.push_str(&format!("![](../images/{image})\n"));
    }
    if let (Some(url), Some(title)) = (&item.url, &item.title) {
        out.push_str(&format!("\n[{}]({url})\n", title.replace(['[', ']'], "")));
    } else if let Some(url) = &item.url {
        out.push_str(&format!("\n<{url}>\n"));
    }
    out
}

/// A double-quoted YAML string.
fn yaml(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', " "))
}

/// Whether `path` is a file rjot wrote: front matter with an `rjot_id:`.
fn ours(path: &Path) -> bool {
    std::fs::read_to_string(path)
        .map(|t| t.starts_with("---\n") && t.lines().take(3).any(|l| l.starts_with("rjot_id: ")))
        .unwrap_or(false)
}

fn write_if_changed(path: &Path, content: &str) -> io::Result<()> {
    if std::fs::read_to_string(path).is_ok_and(|old| old == content) {
        return Ok(());
    }
    std::fs::write(path, content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Color;

    fn item(id: u64, kind: Kind, text: &str) -> Item {
        Item {
            id,
            kind,
            text: text.into(),
            source: None,
            created: 1_792_152_000,
            done: false,
            color: None,
            url: None,
            title: None,
            image: None,
            pinned: false,
            archived: false,
        }
    }

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("rjot-export-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn names() {
        assert_eq!(file_name(&item(7, Kind::Note, "Capture now, sort later.")), "Capture now sort later (7).md");
        assert_eq!(file_name(&item(8, Kind::Note, "  ")), "Note (8).md");
        let mut img = item(9, Kind::Note, "");
        img.image = Some("9.png".into());
        assert_eq!(file_name(&img), "Image (9).md");
        assert_eq!(file_name(&item(10, Kind::Note, "a/b: c?")), "ab c (10).md");
    }

    #[test]
    fn front_matter() {
        let mut i = item(3, Kind::Note, "hello");
        i.source = Some("Safari".into());
        i.title = Some("A \"quoted\" page".into());
        i.url = Some("https://example.com/x".into());
        i.color = Some(Color::Green);
        i.pinned = true;
        let md = markdown(&i);
        assert!(md.starts_with("---\nrjot_id: 3\nkind: note\ncreated: 2026-10-16T12:00:00Z\n"));
        assert!(md.contains("title: \"A \\\"quoted\\\" page\"\n"));
        assert!(md.contains("color: green\npinned: true\n---\n\nhello\n"));
        assert!(md.ends_with("[A \"quoted\" page](https://example.com/x)\n"));
    }

    #[test]
    fn export_writes_moves_and_cleans() {
        let dir = temp("cycle");
        let images = dir.join("src-images");
        std::fs::create_dir_all(&images).unwrap();
        std::fs::write(images.join("2.png"), b"png").unwrap();
        let mut img = item(2, Kind::Note, "");
        img.image = Some("2.png".into());
        let mut items = vec![item(1, Kind::Note, "first"), img, item(3, Kind::Prompt, "ask")];
        // A file of the user's own in our folder must survive.
        std::fs::create_dir_all(root(&dir).join("Notes")).unwrap();
        std::fs::write(root(&dir).join("Notes/mine.md"), "my own notes").unwrap();

        export(&dir, &items, "pad", &images).unwrap();
        let r = root(&dir);
        assert!(r.join("Notes/first (1).md").exists());
        assert!(r.join("Queue/ask (3).md").exists());
        assert!(r.join("images/2.png").exists());
        assert_eq!(std::fs::read_to_string(r.join("Scratch.md")).unwrap(), "pad");

        items[0].text = "renamed".into();
        items[2].archived = true;
        export(&dir, &items, "pad", &images).unwrap();
        assert!(!r.join("Notes/first (1).md").exists());
        assert!(r.join("Notes/renamed (1).md").exists());
        assert!(!r.join("Queue/ask (3).md").exists());
        assert!(r.join("Archive/ask (3).md").exists());
        assert!(r.join("Notes/mine.md").exists(), "never delete files rjot didn't write");

        // A rename that only changes case keeps the file (APFS ignores case).
        items[0].text = "Renamed".into();
        export(&dir, &items, "pad", &images).unwrap();
        let notes: Vec<String> = std::fs::read_dir(r.join("Notes")).unwrap().flatten().map(|e| e.file_name().to_string_lossy().to_lowercase()).collect();
        assert!(notes.contains(&"renamed (1).md".to_string()), "{notes:?}");
    }
}
