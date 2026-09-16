//! Blender ref: `uiItemR` — every RNA prop renders as label + kind-specific
//! control (checkbox, number field, enum dropdown, vec3 triple, colour).
//!
//! `BPropKind::Enum` renders as a **popover-anchored dropdown**: the anchor
//! button shows the current selection (or the property label as placeholder),
//! and its body is a menu of `BEnumItem`s. Clicking an item writes the item's
//! key to `ctx.buts.<spec.key>` via `BAction::Set` — mirroring how Blender's
//! `Add Modifier` operator, physics-add and constraint-add pickers work.

use super::prop::{BPropKind, BPropSpec};
use crate::v2::state::{BPath, BState, val::V_TRUE};
use crate::v2::view::prim::{BAction, BControl, BPopoverSide};
use crate::v2::view::BView;

/// State key that stores the currently-selected enum value for `spec`.
/// Segments live under `buts` so all Property-panel enum dropdowns share
/// one root and can be introspected uniformly.
fn value_key(spec: &BPropSpec) -> BPath {
    BPath::owned(vec!["buts", spec.key])
}

/// State key gating whether the popover for `spec` is open. Uses a `dropdown`
/// segment so it never collides with the value key even when the value key is
/// named similarly.
fn open_key(spec: &BPropSpec) -> BPath {
    BPath::owned(vec!["buts", "dropdown", spec.key])
}

/// Build the popover body — one `BView::button_action` per enum item. Each
/// item sets the value key AND closes the popover by clearing the open key
/// via a compound-effect chain (BAction only carries one write; the popover
/// itself closes on click-outside, so a single Set(value_key, ...) suffices
/// for the user-visible behaviour).
///
/// The currently-selected item is prefixed with a `check` icon so the user
/// can see which value is active — mirrors Blender's `uiItemR` enum menus
/// (the active enum flag renders with a `RADIOBUT_ON` glyph).
fn enum_body(
    spec: &BPropSpec,
    items: &'static [super::prop::BEnumItem],
    selected: &str,
) -> Vec<BView> {
    items
        .iter()
        .map(|item| {
            let icon = if item.key == selected { "check" } else { "" };
            BView::button_action(
                icon,
                item.label,
                BAction::Set(value_key(spec), item.key.to_string()),
            )
        })
        .collect()
}

fn value(spec: &BPropSpec, s: &BState) -> BView {
    match spec.kind {
        BPropKind::Bool => BView::button("check_square"),
        BPropKind::Int { .. } => BView::field("0"),
        BPropKind::Float { .. } => BView::field("0.000"),
        BPropKind::Vec3 { .. } => BView::row(vec![
            BView::field("X 0.000"),
            BView::field("Y 0.000"),
            BView::field("Z 0.000"),
        ]),
        BPropKind::Vec4 { .. } => BView::row(vec![
            BView::field("R 1.0"),
            BView::field("G 1.0"),
            BView::field("B 1.0"),
            BView::field("A 1.0"),
        ]),
        BPropKind::Enum(items) => {
            // Resolve the currently-selected item (falls back to the first
            // choice's label — same visual as before, but now interactive).
            let selected = s.get(&value_key(spec)).unwrap_or("");
            let anchor_label = items
                .iter()
                .find(|item| item.key == selected)
                .map(|item| item.label)
                .or_else(|| items.first().map(|item| item.label))
                .unwrap_or("—")
                .to_string();
            let ok = open_key(spec);
            let open = s.get(&ok) == Some(V_TRUE);
            let anchor = BView::MenuAnchor(
                BControl::new()
                    .with_icon("chevron_down")
                    .with_label(anchor_label)
                    .with_active(open)
                    .with_action(BAction::Toggle(ok.clone())),
            );
            BView::popover(
                anchor,
                enum_body(spec, items, selected),
                open,
                BAction::Reset(ok),
                BPopoverSide::Below,
            )
        }
        BPropKind::Str => BView::field(""),
        BPropKind::Color { .. } => BView::field("● color"),
        BPropKind::Pointer { target } => BView::field(target),
    }
}

pub fn prop_row(spec: &BPropSpec, s: &BState) -> BView {
    // A property row is a Panel with role `Section` so the renderer splits it
    // 50/50 (label left, control right) — Blender's `uiItemR` split factor.
    // Using a Panel(Section) rather than a bare Row lets the renderer own the
    // justify-between + equal-basis layout instead of clamping to content.
    BView::panel(
        crate::v2::view::prim::BPanelRole::Section,
        vec![BView::label(spec.label), value(spec, s)],
    )
}
