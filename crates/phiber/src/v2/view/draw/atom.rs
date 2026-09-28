//! Atom walkers — `Label`, `Icon`, `Button`, `Canvas`, `Empty`.
//!
//! Icons render via `gpui::svg().path("icons/<name>.svg")`; if an icon
//! slug has no matching asset, `Label` renders the text fallback naturally.
//! Buttons are stateful (`.id`) so `.on_click` can dispatch `BAction`
//! through the render context handler.

use super::color::to_hsla;
use super::ctx::BRenderCtx;
use crate::v2::host::BHostType;
use crate::v2::view::prim::BControl;
use gpui::prelude::*;
use gpui::{AnyElement, SharedString, div, px, svg};

pub(super) fn icon_path(name: &str) -> String {
    let clean = name.replace('-', "_").replace(' ', "_").to_lowercase();
    format!("icons/{}.svg", clean)
}

pub fn label(text: &str, ctx: &BRenderCtx) -> AnyElement {
    let mut element = div()
        .text_color(to_hsla(ctx.style.colors.text))
        .text_size(px(ctx.style.typography.body));
    if super::chrome::in_strip(ctx) {
        // Shorter than the strip, so it sits on the centre line rather than
        // on the caps' baseline — the status bar's read-out was riding low
        // against the button beside it.
        element = element.self_center().flex_shrink_0();
    }
    element.child(SharedString::from(text.to_string())).into_any_element()
}

pub fn icon(name: &str, ctx: &BRenderCtx) -> AnyElement {
    if name.is_empty() {
        return div().into_any_element();
    }
    if let Some(mark) = super::brand::view(name, ctx) {
        return mark;
    }
    svg()
        .path(icon_path(name))
        .size(px(ctx.style.dimensions.icon_size))
        .flex_shrink_0()
        .text_color(to_hsla(ctx.style.colors.text))
        .into_any_element()
}

pub fn button(ctl: &BControl, ctx: &BRenderCtx) -> AnyElement {
    super::button::view(ctl, ctx)
}

pub fn canvas(host: BHostType, camera: &crate::v2::scene::camera_view::BCameraSlot, ctx: &BRenderCtx) -> AnyElement {
    super::canvas::view(host, camera, ctx)
}

pub fn empty() -> AnyElement {
    div().into_any_element()
}

/// Flexible spacer — grows along the parent's stacking axis to push the
/// following views to the far end (activity bar three-region layout).
pub fn spacer() -> AnyElement {
    div().flex_1().into_any_element()
}
