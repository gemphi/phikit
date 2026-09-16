//! Shared outliner service used by phiber's Outliner host.
//!
//! Pure Rust — no gpui, no OS deps. Consumers reach these types via
//! `apple::services::outliner::…`.
//!
//! Requirements: 14.1, 15.1, 16.x, 17.x, 18.x, 19.x, 21.x
//! Property tests: `crates/apple/tests/outliner_*_props.rs`.

#![allow(dead_code)]

pub mod types;
pub mod filter;
pub mod edit;

pub use types::{
    BDropTarget, BOutlineFilter, BOutlineMode, BOutlineRow, BOutlineRowKind,
    BOutlineTree, BRenameError, BReparentError,
};

/// State-key helpers for the flat KV store phiber uses to track
/// expansion / drag / rename state. Callers concatenate them with the
/// row id or field key as needed.
pub const K_EXPANDED_PREFIX: &str = "outliner.expanded.";

/// Set `outliner.expanded.<id>` in a flat `HashMap<String, String>` store.
/// A `true` writes `"true"`; `false` removes the key.
///
/// Callers pass in `&mut HashMap` because the outliner service is
/// intentionally decoupled from `phiber::BState` — anything with a
/// map-like insert/remove interface works.
pub fn set_expanded(map: &mut std::collections::HashMap<String, String>, id: &str, on: bool) {
    let k = format!("{K_EXPANDED_PREFIX}{id}");
    if on {
        map.insert(k, "true".to_string());
    } else {
        map.remove(&k);
    }
}

/// Read the expansion flag for row `id`. Missing key → `false`.
pub fn is_expanded(map: &std::collections::HashMap<String, String>, id: &str) -> bool {
    let k = format!("{K_EXPANDED_PREFIX}{id}");
    matches!(map.get(&k).map(String::as_str), Some("true"))
}

/// Expand `root_id` and every descendant of `root_id` in `tree`.
/// Descendants that are already expanded remain expanded (idempotent).
pub fn expand_all(
    map: &mut std::collections::HashMap<String, String>,
    tree: &BOutlineTree,
    root_id: &str,
) {
    if let Some(row) = tree.find(root_id) {
        walk(row, |r| set_expanded(map, &r.id, true));
    }
}

/// Collapse only `id` — descendant expansion state is left untouched
/// (Requirement 16.6).
pub fn collapse(map: &mut std::collections::HashMap<String, String>, id: &str) {
    set_expanded(map, id, false);
}

fn walk<F: FnMut(&BOutlineRow)>(row: &BOutlineRow, mut visit: F) {
    fn go<F: FnMut(&BOutlineRow)>(r: &BOutlineRow, f: &mut F) {
        f(r);
        for c in &r.children {
            go(c, f);
        }
    }
    go(row, &mut visit);
}
