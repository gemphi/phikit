//! Real directory enumeration, sorting and filtering.
//! Blender ref: `filelist_readjob_directories.cc`, `filelist_sort.cc`,
//! `filelist_filter.cc`.
//!
//! Everything here is a pure function of `(dir, BState)` — no caching, no
//! background job. A directory listing is cheap enough to redo per frame at
//! the sizes a file dialog shows, and it keeps the widget stateless.

use crate::v2::state::BKey as _;
use crate::v2::state::FileKey;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use super::kind::BFileKind;
use crate::v2::state::{
    BState,
    val::{V_FALSE, V_TRUE},
};

/// One enumerated entry.
#[derive(Clone, Debug, PartialEq)]
pub struct BFileRow {
    pub name: String,
    /// Absolute path — what a selection writes to `ctx.file.selected`.
    pub path: String,
    pub kind: BFileKind,
    pub size: u64,
    /// Seconds since the Unix epoch, or `None` when unavailable.
    pub modified: Option<u64>,
}

impl BFileRow {
    pub fn is_dir(&self) -> bool {
        self.kind == BFileKind::Folder
    }
    pub fn icon(&self) -> &'static str {
        self.kind.icon()
    }

    /// Blender shows directory sizes as blank and file sizes in binary units.
    pub fn format_size(&self) -> String {
        if self.is_dir() {
            return String::new();
        }
        const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
        let mut v = self.size as f64;
        let mut u = 0;
        while v >= 1024.0 && u < UNITS.len() - 1 {
            v /= 1024.0;
            u += 1;
        }
        if u == 0 {
            format!("{} {}", self.size, UNITS[0])
        } else {
            format!("{v:.1} {}", UNITS[u])
        }
    }

    /// `dd Mon yyyy hh:mm`, UTC. Implemented inline because the crate has no date
    /// dependency; this is Howard Hinnant's civil-from-days algorithm.
    pub fn format_time(&self) -> String {
        let Some(secs) = self.modified else {
            return String::new();
        };
        let days = (secs / 86_400) as i64;
        let rem = secs % 86_400;
        let (h, mi) = ((rem / 3600), (rem % 3600) / 60);

        let z = days + 719_468;
        let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let y = yoe + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let y = if m <= 2 { y + 1 } else { y };

        format!("{d:02} {} {y} {h:02}:{mi:02}", MONTHS[(m - 1) as usize])
    }
}

/// Platform path separator, chosen from the directory string itself so a
/// Windows path stays backslashed and a POSIX one stays slashed.
pub fn sep_of(dir: &str) -> char {
    if dir.contains('\\') && !dir.contains('/') {
        '\\'
    } else if cfg!(windows) {
        '\\'
    } else {
        '/'
    }
}

pub fn join(dir: &str, name: &str) -> String {
    let sep = sep_of(dir);
    if dir.ends_with(['/', '\\']) {
        format!("{dir}{name}")
    } else {
        format!("{dir}{sep}{name}")
    }
}

/// The parent of `dir`, or `None` at a filesystem root.
pub fn parent_of(dir: &str) -> Option<String> {
    let trimmed = dir.trim_end_matches(['/', '\\']);
    let cut = trimmed.rfind(['/', '\\'])?;
    // Keep the trailing separator on a drive root ("C:\") and on "/".
    let head = &trimmed[..cut];
    if head.is_empty() {
        return Some("/".to_string());
    }
    if head.ends_with(':') {
        return Some(format!("{head}\\"));
    }
    Some(head.to_string())
}

/// The directory the browser opens on: whatever is stored, else the user's
/// home, else the process working directory.
pub fn current_dir(s: &BState) -> String {
    if let Some(p) = s.get(&FileKey::Path.path()).filter(|p| !p.is_empty()) {
        return p.to_string();
    }
    home_dir()
}

pub fn home_dir() -> String {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .ok()
        .filter(|p| !p.is_empty())
        .or_else(|| {
            std::env::current_dir()
                .ok()
                .map(|p| p.display().to_string())
        })
        .unwrap_or_else(|| ".".to_string())
}

/// Enumerate `dir`. Returns `Err` with a human-readable reason so the browser
/// can show Blender's "Directory does not exist" style message instead of an
/// empty list that looks like a bug.
pub fn read_dir(dir: &str) -> Result<Vec<BFileRow>, String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("{dir}: {e}"))?;
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let meta = entry.metadata().ok();
        let is_dir = meta.as_ref().map(|m| m.is_dir()).unwrap_or(false);
        let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
        let modified = meta
            .as_ref()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs());
        out.push(BFileRow {
            path: join(dir, &name),
            kind: BFileKind::of(&name, is_dir),
            name,
            size: if is_dir { 0 } else { size },
            modified,
        });
    }
    Ok(out)
}

// ── sorting ────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BFileSort {
    Name,
    Extension,
    Time,
    Size,
}

impl BFileSort {
    pub fn label(self) -> &'static str {
        match self {
            Self::Name => "Name",
            Self::Extension => "Extension",
            Self::Time => "Modified Date",
            Self::Size => "Size",
        }
    }
    pub fn from_label(s: &str) -> Option<Self> {
        [Self::Name, Self::Extension, Self::Time, Self::Size]
            .into_iter()
            .find(|k| k.label() == s)
    }
}

pub fn sort_mode(s: &BState) -> BFileSort {
    s.get(&FileKey::Sort.path())
        .and_then(BFileSort::from_label)
        .unwrap_or(BFileSort::Name)
}
pub fn sort_inverted(s: &BState) -> bool {
    s.get(&FileKey::SortInvert.path()) == Some(V_TRUE)
}

fn ext_of(name: &str) -> String {
    name.rsplit_once('.')
        .map(|(_, e)| e.to_ascii_lowercase())
        .unwrap_or_default()
}

/// Sort in place. Directories always come first, as in Blender, and the
/// inversion flag flips only the within-group ordering.
pub fn sort_rows(rows: &mut [BFileRow], mode: BFileSort, inverted: bool) {
    rows.sort_by(|a, b| {
        let dirs = b.is_dir().cmp(&a.is_dir());
        if dirs != std::cmp::Ordering::Equal {
            return dirs;
        }
        let ord = match mode {
            BFileSort::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            BFileSort::Extension => ext_of(&a.name).cmp(&ext_of(&b.name)),
            BFileSort::Time => a.modified.cmp(&b.modified),
            BFileSort::Size => a.size.cmp(&b.size),
        }
        .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        .then_with(|| a.name.cmp(&b.name))
        .then_with(|| a.path.cmp(&b.path));
        if inverted { ord.reverse() } else { ord }
    });
}

// ── filtering ──────────────────────────────────────────────────────────────

pub fn show_hidden(s: &BState) -> bool {
    s.get(&FileKey::ShowHidden.path()) == Some(V_TRUE)
}
/// The master filter switch is OFF by default, matching Blender's file
/// dialog, so a fresh browser shows everything.
pub fn use_filter(s: &BState) -> bool {
    s.get(&FileKey::UseFilter.path()) == Some(V_TRUE)
}

fn flag_enabled(s: &BState, flag: &'static str) -> bool {
    s.get(&FileKey::Filter(flag).path()) != Some(V_FALSE)
}

/// Apply hidden-file, type and search filtering. Folders survive the type
/// filter unless `filter_folder` is explicitly off, so you can always
/// navigate out of a directory whose contents are all filtered away.
pub fn filter_rows(rows: Vec<BFileRow>, s: &BState) -> Vec<BFileRow> {
    let hidden_ok = show_hidden(s);
    let filtering = use_filter(s);
    let query = s.get(&FileKey::Search.path()).unwrap_or("").trim().to_lowercase();

    rows.into_iter()
        .filter(|r| hidden_ok || !r.name.starts_with('.'))
        .filter(|r| {
            if !filtering {
                return true;
            }
            match r.kind.filter_flag() {
                Some(flag) => flag_enabled(s, flag),
                // Unrecognised kinds are hidden while filtering is on.
                None => r.is_dir(),
            }
        })
        .filter(|r| query.is_empty() || r.name.to_lowercase().contains(&query))
        .collect()
}

/// The full pipeline: enumerate, filter, sort.
pub fn rows_for(s: &BState) -> Result<Vec<BFileRow>, String> {
    let dir = current_dir(s);
    let mut rows = filter_rows(read_dir(&dir)?, s);
    sort_rows(&mut rows, sort_mode(s), sort_inverted(s));
    Ok(rows)
}

// ── formatting ─────────────────────────────────────────────────────────────



const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];



/// Timestamp of "now", for tests and for the modal's default sort.
pub fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
