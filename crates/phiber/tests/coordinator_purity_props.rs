//! Property 4 — Coordinator purity (runtime).
//!
//! Feature: phiber-blender-space-parity
//! Validates: Requirements 50.3, 52.14
//!
//! For every level in `{Space, Zone, Area, Host, Region, Pane}`, the
//! coordinator's `build(state)` emits either the level's own `BView::<Level>`
//! variant or `BView::Empty` — no primitives.
//!
//! The compile-time / structural lint in `views_no_primitives_above_part.rs`
//! walks source files. This runtime check walks the actual emitted
//! `BView` tree and asserts no `Row`/`Column`/`Panel`/etc. variant appears
//! at coordinator depth (i.e. above any `BView::Part`).

use phiber::v2::space::{BSpaceItem, BSpaceType};
use phiber::v2::state::default_state;
use phiber::v2::view::{BView, BViewItem};

/// True iff `v` is a primitive variant (never legal at coordinator depth).
fn is_primitive(v: &BView) -> bool {
    matches!(
        v,
        BView::Row { .. }
            | BView::Column { .. }
            | BView::Panel { .. }
            | BView::Label { .. }
            | BView::VLabel { .. }
            | BView::Field { .. }
            | BView::Icon { .. }
            | BView::Button(_)
            | BView::SortHeader { .. }
            | BView::AccordionHeader { .. }
            | BView::MenuItem(_)
            | BView::MenuSep
            | BView::Tab(_)
            | BView::Popover { .. }
            | BView::Canvas { .. }
    )
}

/// Walk the tree; return the first primitive found *outside* a Part.body.
fn find_primitive_above_part<'a>(v: &'a BView, inside_part: bool) -> Option<&'a BView> {
    if !inside_part && is_primitive(v) {
        return Some(v);
    }
    match v {
        BView::Space { children, .. }
        | BView::Zone { children, .. }
        | BView::Host { children, .. }
        | BView::Region { children, .. }
        | BView::Pane { children, .. } => {
            for c in children {
                if let Some(f) = find_primitive_above_part(c, false) {
                    return Some(f);
                }
            }
            None
        }
        BView::Area { child, .. } => find_primitive_above_part(child, false),
        BView::Part { body, .. } => find_primitive_above_part(body, true),
        // Primitives inside a Part body are fine — walk their children if any.
        BView::Row { children } | BView::Column { children } | BView::Panel { children, .. } => {
            for c in children {
                if let Some(f) = find_primitive_above_part(c, inside_part) {
                    return Some(f);
                }
            }
            None
        }
        BView::Popover { anchor, body, .. } => find_primitive_above_part(anchor, inside_part)
            .or_else(|| {
                body.iter()
                    .find_map(|c| find_primitive_above_part(c, inside_part))
            }),
        _ => None,
    }
}

#[test]
fn every_space_stays_pure_above_part() {
    let state = default_state();
    for &space in BSpaceType::ALL {
        let root = BSpaceItem::new(space).build(&state);
        if let Some(prim) = find_primitive_above_part(&root, false) {
            panic!("space {space:?} emits a primitive at coordinator depth: {prim:?}",);
        }
    }
}
