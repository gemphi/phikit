//! Area walker — fixed side-column widths come from `BAreaLayout::size()`;
//! the enclosing Zone owns variable-axis fractions and height allocation.
//! Left/Right areas scroll vertically and all hosts fill their assigned slot.

use super::{color::to_hsla, ctx::BRenderCtx, surface::BCorners, walk};
use crate::v2::area::BAreaType;
use crate::v2::host::{BHostType, host_slug};
use crate::v2::theme::BStyleRole;
use crate::v2::view::BView;
use crate::v2::view::render::BAreaLayout;
use gpui::prelude::*;
use gpui::{AnyElement, SharedString, div, px};

fn child_host(v: &BView) -> BHostType {
    if let BView::Host { kind, .. } = v {
        *kind
    } else {
        BHostType::Empty
    }
}

fn card(el: gpui::Div, c: &BRenderCtx) -> gpui::Div {
    // Full-bleed hosts fill their zone slot flush with no card chrome (no
    // border / bg tint / radius) so Blender paints the whole area directly.
    // The set is owned by `BHostType::fills_content`. Properties (Buts) is
    // also flush but keeps its own recessed surface, hence the extra arm.
    if c.host == BHostType::Buts || c.host.fills_content() {
        // Properties: zero radius, flush with workspace.
        // Content-fill hosts: no chrome — the host bg paints its own surface.
        el
    } else {
        el.bg(to_hsla(c.style.workspace_background()))
            .border(px(c.style.strokes.border))
            .border_color(to_hsla(c.style.colors.border))
            .rounded(px(c.style.radius_for(BStyleRole::Area)))
    }
}

pub fn view(kind: BAreaType, child: &BView, c: &BRenderCtx) -> AnyElement {
    let host = child_host(child);

    // The Activity rail is chrome, not a content panel: pin it to the narrow
    // activity width and skip the rounded card + scroll wrapper so it reads
    // as a slim vertical bar flush against the workspace.
    if host == BHostType::Activity {
        return div()
            .flex()
            .flex_col()
            .w(px(c.style.dimensions.activity_width))
            .h_full()
            .min_h_0()
            .flex_shrink_0()
            .child(walk(child, c))
            .into_any_element();
    }

    let sz = kind.size();
    let mut el = div().flex().flex_col().flex_1().min_h_0();
    // The zone owns the variable-axis split. Areas only establish the fixed
    // side-column width; applying Top/Bottom heights here caused nested hosts
    // to overflow or collapse instead of filling their allocated slot.
    el = if let Some(w) = sz.width {
        el.w(px(w))
    } else {
        el.w_full().min_w_0()
    };
    el = card(el, c);

    // The host is the opaque descendant that reaches every area corner. Give
    // it explicit ownership so it can paint a compatible inner arc; clipping
    // with `overflow_hidden` is intentionally avoided because GPUI's mask is
    // rectangular rather than radius-aware.
    let child_context = c.with_corners(BCorners::ALL);
    if kind.is_scrollable() {
        let id = SharedString::from(format!("area-{:?}-{}", kind, host_slug(host)));
        let scroller = div()
            .id(id)
            .flex()
            .flex_col()
            .flex_1()
            .size_full()
            .min_h_0()
            .min_w_0()
            .overflow_y_scroll()
            .child(walk(child, &child_context));
        return el.child(scroller).into_any_element();
    }
    el.child(walk(child, &child_context)).into_any_element()
}
