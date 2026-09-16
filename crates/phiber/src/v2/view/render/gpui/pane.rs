//! Pane walker — tri-cell horizontal slot inside a region. Left hugs left,
//! Center flexes, Right hugs right. Uses `items_stretch` (default) so a
//! Canvas child in the Center pane can fill the full pane height; button
//! rows keep their own `h(h_button)` and still align cleanly.
//!
//! In every **canvas host** (View3D + node graphs + image/UV/seq/clip) the
//! Toolbar (Left) and Sidebar (Right) panes are rendered as absolute
//! overlays on top of the content so the canvas fills the full host area
//! (Blender's floating panel behaviour).

use super::{color::to_hsla, ctx::BRenderCtx, walk};
use crate::v2::pane::BPaneType;
use crate::v2::view::BView;
use crate::v2::view::render::BPaneLayout;
use gpui::prelude::*;
use gpui::{AnyElement, Hsla, div, hsla, px};

/// True when this pane lives inside a floating-side-panel host's center
/// region and should be rendered as a floating absolute overlay instead of a
/// flex slot. The host set is owned by [`BHostType::floats_side_panels`] (the
/// viewport + graphs + image/uv/seq/clip) — timelines fill content but keep
/// their docked Channels list, so they are excluded.
fn is_floating_pane(kind: BPaneType, c: &BRenderCtx) -> bool {
    c.host.floats_side_panels() && matches!(kind, BPaneType::Left | BPaneType::Right)
}

/// Panel bg with a bit of transparency so the canvas shows through the
/// floating chrome, matching Blender's translucent side panels.
fn floating_bg(c: &BRenderCtx) -> Hsla {
    let base = to_hsla(c.style.colors.bg_panel);
    hsla(base.h, base.s, base.l, 0.88)
}

pub fn view(kind: BPaneType, children: &[BView], c: &BRenderCtx) -> AnyElement {
    // Canvas-host Toolbar / Sidebar: render as absolute overlay, not a flex slot.
    if is_floating_pane(kind, c) {
        let pad = px(c.style.spacing.pad_sm);
        let radius = px(c.style.radius_for(crate::v2::theme::BStyleRole::Panel));
        let bg = floating_bg(c);

        let is_right = kind == BPaneType::Right;
        let mut outer = div()
            .absolute()
            .top(pad)
            .bottom(pad)
            .flex()
            .flex_col()
            .gap(px(c.style.spacing.gap_xs))
            .p(pad)
            .bg(bg)
            .rounded(radius);
        outer = if is_right {
            outer.right(pad)
        } else {
            outer.left(pad)
        };
        for ch in children {
            outer = outer.child(walk(ch, c));
        }
        return outer.into_any_element();
    }

    // A header row lays out its three panes tri-cell: Left hugs left, Center
    // flexes to occupy the gap (so Left/Right land at opposite ends) but its
    // CHILDREN hug-and-centre rather than stretch, and Right hugs right. Only
    // a content-region Center stretches its child to fill (canvas / accordion
    // stack). This split is what stops header content from ballooning into
    // full-width "pills" or overflowing the pane.
    let in_header = matches!(
        c.region,
        Some(crate::v2::region::BRegionType::Header)
            | Some(crate::v2::region::BRegionType::ToolHeader)
    );
    let base = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(c.style.spacing.gap_xs));
    let center = kind.is_center();
    let mut el = if center {
        // Center still takes the flex slot so Left/Right sit at the ends, but
        // in a header it centres its (hugging) content instead of stretching
        // each child to full width.
        let center_el = base.flex_1().h_full().min_h_0().min_w_0();
        if in_header {
            center_el.justify_center()
        } else {
            center_el
        }
    } else if kind.is_right() {
        base.flex_shrink_0().justify_end()
    } else {
        // Left pane: shrink-to-fit, never expand.
        base.flex_shrink_0()
    };
    // Only a CONTENT-region Center stretches its child to fill the slot (a
    // property stack or a canvas). A header Center hugs its content so menus
    // and the mode dropdown keep their natural width.
    let stretch_child = center && !in_header;
    for ch in children {
        if stretch_child {
            el = el.child(
                div()
                    .flex()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .min_h_0()
                    .child(walk(ch, c)),
            );
        } else {
            el = el.child(walk(ch, c));
        }
    }
    el.into_any_element()
}
