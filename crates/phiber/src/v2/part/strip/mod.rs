//! Shared vertical icon-strip state helpers.
//!
//! The Properties navbar, the View3D-family Toolbar and the Sidebar are the
//! same element — a `BView::Rail` — differing only in their entry tables.
//! This module owns the state contract every rail shares so all three parts
//! stay identical: the per-host expand key, the expand test, the paging
//! ring-shift, and the fixed bottom cluster (expand chevron collapsed;
//! prev / collapse / next expanded) expressed as rail entries.
//!
//! Pure functions on `BState` — no view construction beyond the entries
//! themselves; the rail renderer owns layout.

use crate::v2::host::BHostType;
use crate::v2::state::{BPath, BState, val::V_TRUE};
use crate::v2::view::prim::{BAction, BControl, BRailEntry, BRailGroup};

/// Per-host expand-state key: `<host_slug>.<strip>.expanded == "true"` when
/// the strip is showing labels + paging controls.
pub fn expanded_key(host: BHostType, strip: &'static str) -> BPath {
    BPath::owned(vec![host.slug(), strip, "expanded"])
}

/// True when the given strip is currently expanded.
///
/// A strip the user has not touched falls back to [`default_expanded`], so a
/// rail that is a labelled list in Blender opens as one here.
pub fn is_expanded(s: &BState, host: BHostType, strip: &'static str) -> bool {
    match s.get(&expanded_key(host, strip)) {
        Some(value) => value == V_TRUE,
        None => default_expanded(host, strip),
    }
}

/// Whether a strip shows its labels before the user has said otherwise.
///
/// Properties' nav bar is an icon rail — fifteen contexts, each with an icon
/// everyone knows. Preferences' is a list of named categories, and Blender
/// draws it that way: collapsing it to icons left the screen looking like it
/// had no sidebar at all.
pub fn default_expanded(host: BHostType, strip: &'static str) -> bool {
    matches!((host, strip), (BHostType::Prefs, "navbar"))
}

/// Compute the label `offset` positions away from `active` in `labels`,
/// wrapping at both ends so the strip pages as a ring. Returns `""` when the
/// list is empty.
pub fn shifted_label(labels: &[&'static str], active: &str, offset: isize) -> &'static str {
    if labels.is_empty() {
        return "";
    }
    let n = labels.len() as isize;
    let idx = labels
        .iter()
        .position(|l| *l == active)
        .map(|i| i as isize)
        .unwrap_or(0);
    let target = ((idx + offset) % n + n) % n;
    labels[target as usize]
}

fn cluster_entry(
    icon: &'static str,
    label: &'static str,
    action: BAction,
) -> BRailEntry {
    BRailEntry {
        control: BControl::new()
            .with_icon(icon)
            .with_label(label)
            .with_action(action),
        group: BRailGroup::Bottom,
    }
}

/// The fixed bottom cluster of a strip rail, as entries.
///
/// - `host` / `strip` identify the expand-state key.
/// - `select_key` is the state path a paging arrow writes to (the same key
///   the tabs themselves set on click).
/// - `labels` + `active` drive the prev/next targets when expanded.
///
/// Collapsed: a single expand chevron. Expanded: `[prev][collapse][next]`,
/// which the rail renderer lays out as one horizontal row.
pub fn cluster_entries(
    host: BHostType,
    strip: &'static str,
    select_key: BPath,
    labels: &[&'static str],
    active: &str,
    expanded: bool,
) -> Vec<BRailEntry> {
    let key = expanded_key(host, strip);
    if !expanded {
        return vec![cluster_entry(
            "chevron_right",
            "Expand panel",
            BAction::Set(key, V_TRUE.to_string()),
        )];
    }
    let prev_label = shifted_label(labels, active, -1);
    let next_label = shifted_label(labels, active, 1);
    vec![
        cluster_entry(
            "chevron_up",
            "Previous",
            BAction::Set(select_key.clone(), prev_label.to_string()),
        ),
        cluster_entry(
            "chevron_left",
            "Collapse panel",
            BAction::Set(key, String::new()),
        ),
        cluster_entry(
            "chevron_down",
            "Next",
            BAction::Set(select_key, next_label.to_string()),
        ),
    ]
}
