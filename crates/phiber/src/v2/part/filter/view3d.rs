//! The 3D viewport's header popovers — `bl_ui/space_view3d.py`.
//!
//! Blender's `VIEW3D_HT_header` ends with three controls, in this order:
//! the gizmo popover (`VIEW3D_PT_gizmo_display`), the overlay popover
//! (`VIEW3D_PT_overlay`) and the shading radio with its own popover
//! (`VIEW3D_PT_shading`, `shading.rs`). Only the shading radio was here, so
//! two of the three were missing from the editor you spend the most time in.
//!
//! The rows are not new. `props::tool::OPTIONS` and `::OVERLAYS` are the
//! Properties editor's Tool tab, and they key on `ctx.buts.<rna>` — the same
//! keys the canvas reads when it decides whether to draw the floor and the
//! axes. So the popover, the panel and the canvas are one set of values
//! reached three ways; a second table for the header is how the header would
//! come to disagree with the panel.

use crate::v2::part::props::prop::BPropSpec;
use crate::v2::state::val::{V_FALSE, V_TRUE};
use crate::v2::state::{BKey as _, BState, MenuKey};
use crate::v2::view::BView;
use crate::v2::view::prim::{BAction, BControl, BPopoverSide};
use crate::v2::host::BHostType;

/// One row of a popover: the box over the key the panel and the canvas use.
fn check(state: &BState, heading: &str, spec: &BPropSpec) -> BView {
    let on = matches!(spec.default, Some(crate::v2::part::props::prop::BDefault::Bool(true)));
    BView::check_row(state, heading, spec.value_key(), on, spec.label)
}

/// A group of boxes under one heading, as every other popover lays them out.
fn group(state: &BState, heading: &str, specs: &'static [BPropSpec]) -> Vec<BView> {
    specs
        .iter()
        .enumerate()
        .map(|(n, spec)| check(state, if n == 0 { heading } else { "" }, spec))
        .collect()
}

/// `VIEW3D_PT_gizmo_display`'s rows.
pub fn gizmo_body(state: &BState) -> Vec<BView> {
    group(state, "Gizmos", crate::v2::part::props::tool::OPTIONS)
}

/// `VIEW3D_PT_overlay`'s rows, under Blender's two headings.
pub fn overlay_body(state: &BState) -> Vec<BView> {
    use crate::v2::part::props::tool::{OVERLAYS, OVERLAY_OBJECTS};
    let (guides, objects) = OVERLAYS.split_at(OVERLAY_OBJECTS);
    let mut rows = group(state, "Guides", guides);
    rows.extend(group(state, "Objects", objects));
    rows
}

/// One header popover, keyed the way every other header popover is
/// (`MenuKey::Panel`, as the transport's are).
pub(super) fn popover(state: &BState, title: &'static str, icon: &'static str, rows: Vec<BView>) -> BView {
    let key = MenuKey::Panel(BHostType::View3D.slug(), title).path();
    let open = state.get(&key) == Some(V_TRUE);
    let toggle = BAction::Set(key.clone(), if open { V_FALSE } else { V_TRUE }.to_string());
    let anchor = BView::Button(
        BControl::new().with_icon(icon).with_tooltip(title).with_active(open).with_action(toggle),
    );
    BView::popover(anchor, rows, open, BAction::Set(key, V_FALSE.to_string()), BPopoverSide::BelowEnd)
}

/// The gizmo popover.
pub fn gizmos(state: &BState) -> BView {
    popover(state, "Gizmos", "transform_gizmo", gizmo_body(state))
}

/// The overlay popover.
pub fn overlays(state: &BState) -> BView {
    popover(state, "Overlays", "layers", overlay_body(state))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_two_popovers_read_the_panels_own_keys() {
        // The Tool tab writes these and the canvas reads them, so the header
        // must key on the same `ButsKey::Value`. A key of the header's own
        // would silently fork the setting in three.
        use crate::v2::state::ButsKey;
        for spec in crate::v2::part::props::tool::OPTIONS {
            assert_eq!(spec.value_key(), ButsKey::Value(spec.key).path(), "{}", spec.key);
        }
        for spec in crate::v2::part::props::tool::OVERLAYS {
            assert_eq!(spec.value_key(), ButsKey::Value(spec.key).path(), "{}", spec.key);
        }
    }

    #[test]
    fn each_popover_draws_one_row_per_property() {
        let state = BState::new();
        assert_eq!(gizmo_body(&state).len(), crate::v2::part::props::tool::OPTIONS.len());
        assert_eq!(overlay_body(&state).len(), crate::v2::part::props::tool::OVERLAYS.len());
    }

    #[test]
    fn the_two_popovers_do_not_share_an_open_key() {
        // One key would open both at once.
        let gizmo = MenuKey::Panel(BHostType::View3D.slug(), "Gizmos").path();
        let overlay = MenuKey::Panel(BHostType::View3D.slug(), "Overlays").path();
        assert_ne!(gizmo, overlay);
    }
}
