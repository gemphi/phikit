//! Recent-projects list, stored as `ctx.recent.0` … `ctx.recent.9`.
//!
//! Phiber's `BState` is a flat string store with no bridge to Zed's
//! `WorkspaceDb`, so the welcome window keeps its own most-recently-used
//! list: every committed `ctx.file.selected` write pushes its path to the
//! front, de-duplicating and dropping the oldest beyond `RECENT_MAX`.

use super::set::RECENT_MAX;
use crate::v2::state::{BPath, BState};

/// `ctx.recent.<n>`. The index is a small fixed set of leaked `&'static str`
/// so `BPath` stays a static-segment path.
fn slot(n: usize) -> BPath {
    const SLOTS: [&str; RECENT_MAX] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];
    BPath::owned(vec!["recent", SLOTS[n.min(RECENT_MAX - 1)]])
}

/// The recent list, most recent first. Stops at the first empty slot.
pub fn recent_paths(s: &BState) -> Vec<String> {
    let mut out = Vec::new();
    for n in 0..RECENT_MAX {
        match s.get(&slot(n)) {
            Some(p) if !p.is_empty() => out.push(p.to_string()),
            _ => break,
        }
    }
    out
}

/// Push `path` to the front of the recent list, de-duplicating.
pub fn push_recent(state: &mut BState, path: &str) {
    if path.is_empty() {
        return;
    }
    let mut list = recent_paths(state);
    list.retain(|p| p != path);
    list.insert(0, path.to_string());
    list.truncate(RECENT_MAX);
    for n in 0..RECENT_MAX {
        state.set(&slot(n), list.get(n).cloned().unwrap_or_default());
    }
}

/// Display name for a path — the final component, or the whole path when it
/// has no separator.
pub fn display_name(path: &str) -> &str {
    path.rsplit(['/', '\\'])
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or(path)
}

/// The directory part of a path, for the dimmed second line of a row.
pub fn parent_dir(path: &str) -> &str {
    match path.rfind(['/', '\\']) {
        Some(i) => &path[..i],
        None => "",
    }
}
