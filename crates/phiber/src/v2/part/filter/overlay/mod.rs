//! The box popovers editor headers hang off their right end.
//!
//! Blender ends most editor headers with a popover of checkboxes: what the
//! editor draws ON TOP of its content, as opposed to what it draws. The 3D
//! viewport has one (`view3d`), and so do these:
//!
//! - the four node editors — `NODE_PT_overlay` (`bl_ui/space_node.py`) over
//!   `SpaceNodeEditor.overlay`, one panel polled on the tree type;
//! - the Sequencer — `SEQUENCER_PT_overlay` over
//!   `SpaceSequenceEditor.timeline_overlay`;
//! - the Image and UV editors — `IMAGE_PT_overlay` over
//!   `SpaceImageEditor.overlay`, one space in Blender and one family here;
//! - the Movie Clip editor — `CLIP_PT_display`, which Blender calls Display
//!   rather than Overlays because half of it is about the clip, not the marks
//!   drawn over it;
//! - the Spreadsheet — `SPREADSHEET_PT_filter`, which is a filter rather than
//!   an overlay but is the same control: a popover of boxes on the right end.
//!
//! Seven headers' right ends were empty. The mechanism is one popover, one
//! table per family and one scope per family (`set`); this file is what reads
//! and draws it.

pub mod set;

pub use set::{BOverlayPanel, BOverlaySpec, CLIP, IMAGE, NODE, SEQ, SHEET, panel_for};

use crate::v2::host::BHostType;
use crate::v2::state::val::{V_FALSE, V_TRUE};
use crate::v2::state::{BKey as _, BPath, BState, MenuKey};
use crate::v2::view::BView;
use crate::v2::view::prim::{BAction, BControl, BPopoverSide};

/// True when a host carries a box popover of its own.
pub fn has_overlay(host: BHostType) -> bool {
    panel_for(host).is_some()
}

/// The boxes a host's popover draws, if it has one.
pub fn table_for(host: BHostType) -> Option<&'static [BOverlaySpec]> {
    panel_for(host).map(|panel| panel.boxes)
}

/// The key one box reads and writes, scoped to the family so the hosts that
/// share Blender's RNA struct share their answer.
pub fn key(host: BHostType, spec: &BOverlaySpec) -> BPath {
    let family = panel_for(host).map_or_else(|| host.slug(), |panel| panel.family);
    BPath::owned(vec!["overlay", family, spec.key])
}

/// What a box answers: what the user ticked, or what Blender ships with.
pub fn is_on(state: &BState, host: BHostType, spec: &BOverlaySpec) -> bool {
    match state.get(&key(host, spec)) {
        Some(V_TRUE) => true,
        Some(V_FALSE) => false,
        _ => spec.default_on,
    }
}

/// The scope a family's fold keys sit under — the one its boxes already sit
/// under (`key`), so the header popover and the sidebar tab drawing the same
/// groups fold together instead of each keeping its own triangles.
pub fn scope(host: BHostType) -> [&'static str; 2] {
    ["overlay", panel_for(host).map_or_else(|| host.slug(), |panel| panel.family)]
}

/// The popover's body: its headings as folding cards when Blender draws them
/// as sub-panels — `IMAGE_PT_overlay`'s four and `CLIP_PT_display`'s two —
/// and the flat rows when the table has the one heading that is the panel's
/// own name, as `NODE_PT_overlay` and `SPREADSHEET_PT_filter` are
/// (`filter::fold` states the rule and cites the source).
pub fn body(state: &BState, host: BHostType) -> Vec<BView> {
    let Some(table) = table_for(host) else { return Vec::new() };
    super::fold::body(state, &scope(host), groups(state, host), super::fold::all_open, || {
        table
            .iter()
            .map(|spec| {
                BView::check_row(state, spec.group, key(host, spec), spec.default_on, spec.label)
            })
            .collect()
    })
}

/// The same boxes as `body`, split at their headings — one entry per group,
/// which is one sub-panel in Blender (`CLIP_PT_marker_display` and
/// `CLIP_PT_clip_display` under `CLIP_PT_display`).
///
/// Both the sidebar tab and the header popover draw these as folding cards,
/// over one set of keys (`scope`). One table, two places to reach it: a second
/// copy of the rows — or a second set of fold keys — is how a box comes to be
/// ticked in the popover and clear in the sidebar.
pub fn groups(state: &BState, host: BHostType) -> Vec<(&'static str, Vec<BView>)> {
    let Some(table) = table_for(host) else { return Vec::new() };
    let mut out: Vec<(&'static str, Vec<BView>)> = Vec::new();
    for spec in table {
        if !spec.group.is_empty() {
            out.push((spec.group, Vec::new()));
        }
        let row = BView::check_row(state, "", key(host, spec), spec.default_on, spec.label);
        match out.last_mut() {
            Some((_, rows)) => rows.push(row),
            // The table opens with a heading, so this is unreachable —
            // `every_table_opens_with_a_heading` is what keeps it so.
            None => out.push(("", vec![row])),
        }
    }
    out
}

/// The header's right-hand control.
pub fn view(state: &BState, host: BHostType) -> BView {
    let Some(panel) = panel_for(host) else { return BView::empty() };
    let k = MenuKey::Panel(host.slug(), panel.title).path();
    let open = state.get(&k) == Some(V_TRUE);
    let toggle = BAction::Set(k.clone(), if open { V_FALSE } else { V_TRUE }.to_string());
    let anchor = BView::Button(
        BControl::new()
            .with_icon(panel.icon)
            .with_tooltip(panel.title)
            .with_active(open)
            .with_action(toggle),
    );
    BView::popover(anchor, body(state, host), open, BAction::Set(k, V_FALSE.to_string()), BPopoverSide::BelowEnd)
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
