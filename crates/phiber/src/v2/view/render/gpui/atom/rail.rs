//! Rail atom — THE vertical strip renderer.
//!
//! One element for the Activity bar, the Toolbar, the Properties NavBar and
//! the Sidebar: a single column that fills the region slot it is placed in
//! (the region owns width and strip background) and lays its entries out by
//! group — Top entries stack from the top, Bottom entries pin to the bottom,
//! Center entries float between — with one flexible spacer between each
//! pair of consecutive non-empty groups.
//!
//! Entries render through the tab atom, which already owns the paper-tab
//! geometry, the per-host rail active/hover fills and the hover tooltip, so
//! every debug id (`tab-…`, `tab-…-background`, `tab-icon-…`) and every
//! pixel of the existing rail geometry is preserved.
//!
//! The strip kinds' fixed bottom cluster (expand chevron collapsed;
//! prev/collapse/next expanded) is just more entries in the Bottom group —
//! the part list builds them from state, so this renderer stays declarative:
//! it owns layout only, never behaviour.

use super::super::ctx::BRenderCtx;
use super::tab;
use crate::v2::view::prim::{BRailEntry, BRailGroup, BRailView};
use gpui::{AnyElement, div};
use gpui::prelude::*;

pub fn view(rail: &BRailView, c: &BRenderCtx) -> AnyElement {
    let mut el = div().flex().flex_col().w_full().h_full().min_h_0();

    // Groups in declaration order with a flexible spacer between each pair
    // of consecutive non-empty groups. Strip kinds put their cluster in the
    // Bottom group; the Activity bar uses all three groups.
    let mut pending_spacer = false;
    for group in [BRailGroup::Top, BRailGroup::Center, BRailGroup::Bottom] {
        let group_entries: Vec<&BRailEntry> =
            rail.entries.iter().filter(|e| e.group == group).collect();
        if group_entries.is_empty() {
            continue;
        }
        if pending_spacer {
            el = el.child(div().flex_1());
        }
        pending_spacer = true;
        // The expanded strip cluster (prev / collapse / next) is a horizontal
        // row; every other group stacks vertically.
        if rail.kind.has_strip_cluster() && group == BRailGroup::Bottom && rail.expanded {
            let mut row = div().flex().flex_row().w_full();
            for e in group_entries {
                row = row.child(entry(e, false, c));
            }
            el = el.child(row);
        } else {
            for e in group_entries {
                el = el.child(entry(e, rail.expanded, c));
            }
        }
    }
    el.into_any_element()
}

/// One entry cell. The entry's label renders inline when the strip is
/// expanded and as the hover tooltip beside the strip when collapsed.
fn entry(e: &BRailEntry, expanded: bool, c: &BRenderCtx) -> AnyElement {
    let mut ctl = e.control.clone();
    ctl.tooltip = ctl.label.clone();
    if !expanded {
        ctl.label = None;
    }
    tab::view(&ctl, c)
}
