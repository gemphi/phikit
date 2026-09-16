//! Atom walkers — `Label`, `Icon`, `Button`, `Canvas`, `Empty`.
//!
//! Icons render via `gpui::svg().path("icons/<name>.svg")`; if an icon
//! slug has no matching asset, `Label` renders the text fallback naturally.
//! Buttons are stateful (`.id`) so `.on_click` can dispatch `BAction`
//! through the render context handler.

pub mod anchor;
pub mod button;
pub mod canvas;
pub mod field;
pub mod menu;
pub mod progress;
pub mod rail;
pub mod search;
pub mod tab;
pub mod toggle;
pub mod vlabel;

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

pub fn label(text: &str, c: &BRenderCtx) -> AnyElement {
    div()
        .text_color(to_hsla(c.style.colors.text))
        .text_size(px(c.style.typography.body))
        .child(SharedString::from(text.to_string()))
        .into_any_element()
}

pub fn icon(name: &str, c: &BRenderCtx) -> AnyElement {
    if name.is_empty() {
        return div().into_any_element();
    }
    svg()
        .path(icon_path(name))
        .size(px(c.style.typography.header))
        .flex_shrink_0()
        .text_color(to_hsla(c.style.colors.text))
        .into_any_element()
}

pub fn button(ctl: &BControl, c: &BRenderCtx) -> AnyElement {
    button::view(ctl, c)
}

pub fn canvas(host: BHostType, c: &BRenderCtx) -> AnyElement {
    canvas::view(host, c)
}

pub fn empty() -> AnyElement {
    div().into_any_element()
}

/// Flexible spacer — grows along the parent's stacking axis to push the
/// following views to the far end (activity bar three-region layout).
pub fn spacer() -> AnyElement {
    div().flex_1().into_any_element()
}
