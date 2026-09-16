//! Runtime sidebar sources — system bookmarks and volumes.
//! Blender ref: `fsmenu.cc` + `fsmenu_system_win32.cc` / `_unix.cc`.
//!
//! The static `SYSTEM` table in `set.rs` holds `$HOME`-style tokens; these
//! functions expand them against the real environment and drop the entries
//! whose directory does not exist, so the sidebar never offers a dead link.

use std::path::Path;

use super::listing::{home_dir, join};

/// A resolved sidebar row.
#[derive(Clone, Debug, PartialEq)]
pub struct BFsEntry {
    pub icon: &'static str,
    pub label: String,
    pub path: String,
}

fn entry(icon: &'static str, label: &str, path: String) -> Option<BFsEntry> {
    Path::new(&path).is_dir().then(|| BFsEntry {
        icon,
        label: label.to_string(),
        path,
    })
}

/// `FS_CATEGORY_SYSTEM_BOOKMARKS` — the user's shell folders that exist.
pub fn system_bookmarks() -> Vec<BFsEntry> {
    let home = home_dir();
    let mut out = Vec::new();
    out.extend(entry("home", "Home", home.clone()));
    for (icon, name) in [
        ("monitor",  "Desktop"),
        ("doc",      "Documents"),
        ("download", "Downloads"),
        ("speaker",  "Music"),
        ("image",    "Pictures"),
        ("film",     "Videos"),
    ] {
        out.extend(entry(icon, name, join(&home, name)));
    }
    if let Ok(od) = std::env::var("OneDrive") {
        out.extend(entry("cloud", "OneDrive", od));
    }
    out
}

/// `FS_CATEGORY_SYSTEM` — mounted volumes. Windows probes drive letters;
/// elsewhere the common mount roots are offered.
pub fn volumes() -> Vec<BFsEntry> {
    let mut out = Vec::new();
    if cfg!(windows) {
        for letter in b'A'..=b'Z' {
            let root = format!("{}:\\", letter as char);
            if Path::new(&root).is_dir() {
                out.push(BFsEntry {
                    icon: "hard_drive",
                    label: root.clone(),
                    path: root,
                });
            }
        }
    } else {
        for root in ["/", "/mnt", "/media"] {
            out.extend(entry("hard_drive", root, root.to_string()));
        }
    }
    out
}
