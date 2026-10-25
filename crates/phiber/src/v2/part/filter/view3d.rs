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

/// The scope the viewport's fold keys sit under, beside the families'
/// (`overlay::scope`), so one rule reads them all.
const SCOPE: [&str; 2] = ["overlay", "view3d"];

/// `VIEW3D_PT_gizmo_display`'s rows. One panel in Blender — its "Viewport
/// Gizmos" / "Object Gizmos" headings are `col.label`, not sub-panels — so
/// its one heading stays a flat list and grows no triangle.
pub fn gizmo_body(state: &BState) -> Vec<BView> {
    group(state, "Gizmos", crate::v2::part::props::tool::OPTIONS)
}

/// `VIEW3D_PT_overlay`'s rows under Blender's two headings, each a folding
/// card: Blender's are the sub-panels `VIEW3D_PT_overlay_guides` and
/// `VIEW3D_PT_overlay_object`, which carry their own collapse triangle
/// inside the popover (`filter::fold`).
pub fn overlay_body(state: &BState) -> Vec<BView> {
    super::fold::cards(state, &SCOPE, overlay_groups(state), super::fold::all_open)
}

/// The same boxes split at Blender's two headings, headless — what the cards
/// fold over, and what a flat read of the popover counts.
pub fn overlay_groups(state: &BState) -> Vec<super::fold::BGroup> {
    use crate::v2::part::props::tool::{OVERLAYS, OVERLAY_OBJECTS};
    let (guides, objects) = OVERLAYS.split_at(OVERLAY_OBJECTS);
    vec![("Guides", group(state, "", guides)), ("Objects", group(state, "", objects))]
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
    popover(state, "Overlays", "overlays", overlay_body(state))
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
        // Gizmos is one panel in Blender, so it is still one row per box.
        assert_eq!(gizmo_body(&state).len(), crate::v2::part::props::tool::OPTIONS.len());
        // Overlays is two sub-panels, so it is two cards — and between them
        // still one row per box, none lost in the fold.
        let cards = overlay_body(&state);
        assert_eq!(cards.len(), 2, "Guides and Objects, not fourteen loose rows");
        let rows: usize = overlay_groups(&state).iter().map(|(_, rows)| rows.len()).sum();
        assert_eq!(rows, crate::v2::part::props::tool::OVERLAYS.len());
    }

    #[test]
    fn the_overlay_headings_fold_and_the_gizmo_popover_does_not() {
        use crate::v2::view::BView;
        let state = BState::new();
        // `VIEW3D_PT_overlay_guides` / `_object`: a triangle each.
        for view in overlay_body(&state) {
            let BView::Card(card) = view else { panic!("a folding card per heading") };
            assert!(card.fold.is_some(), "a heading with no triangle is the long list again");
        }
        // `VIEW3D_PT_gizmo_display`: no sub-panels, so no cards.
        assert!(!gizmo_body(&state).iter().any(|v| matches!(v, BView::Card(_))));
    }

    #[test]
    fn the_two_popovers_do_not_share_an_open_key() {
        // One key would open both at once.
        let gizmo = MenuKey::Panel(BHostType::View3D.slug(), "Gizmos").path();
        let overlay = MenuKey::Panel(BHostType::View3D.slug(), "Overlays").path();
        assert_ne!(gizmo, overlay);
    }
}
