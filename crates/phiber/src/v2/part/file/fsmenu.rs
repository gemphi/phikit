//! Runtime sidebar sources — system bookmarks and volumes.
//! Blender ref: `fsmenu.cc` + `fsmenu_system_win32.cc` / `_unix.cc`.
//!
//! The static `SYSTEM` table in `set.rs` holds `$HOME`-style tokens; these
//! functions expand them against the real environment and drop the entries
//! whose directory does not exist, so the sidebar never offers a dead link.

use std::path::Path;

use super::listing::home_dir;
use super::listing::join;

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
    // Blender's Windows list goes on with the fonts folder, OneDrive and
    // the screenshots under Pictures.
    if cfg!(windows) {
        let windir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".into());
        out.extend(entry("type", "Fonts", join(&windir, "Fonts")));
    }
    if let Ok(od) = std::env::var("OneDrive") {
        out.extend(entry("cloud", "OneDrive", od));
    }
    out.extend(entry("image", "Screenshots", join(&join(&home, "Pictures"), "Screenshots")));
    out
}

/// `FS_CATEGORY_SYSTEM` — mounted volumes. Windows probes drive letters;
/// elsewhere the common mount roots are offered.
pub fn volumes() -> Vec<BFsEntry> {
    let mut out = Vec::new();
    if cfg!(windows) {
        // Named as Explorer names them: the system drive is the local
        // disk, every other one its letter in brackets.
        let system = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".into());
        for letter in b'A'..=b'Z' {
            let drive = format!("{}:", letter as char);
            let root = format!("{drive}\\");
            if Path::new(&root).is_dir() {
                let label = if drive.eq_ignore_ascii_case(&system) { format!("Local Disk ({drive})") } else { format!("Disk ({drive})") };
                out.push(BFsEntry { icon: "hard_drive", label, path: root });
            }
        }
    } else {
        for root in ["/", "/mnt", "/media"] {
            out.extend(entry("hard_drive", root, root.to_string()));
        }
    }
    out
}
