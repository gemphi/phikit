//! Vertical label — a rotated-text effect achieved by stacking each
//! character in its own row (GPUI's `Div` doesn't offer CSS `transform`).
//! Reads top-down like Blender's sidebar tabs. Uses `whitespace_nowrap`
//! + narrow container so caller can size to sidebar collapsed width.

use super::color::to_hsla;
use super::ctx::BRenderCtx;
use gpui::prelude::*;
use gpui::{AnyElement, SharedString, div, px};

pub fn view(text: &str, ctx: &BRenderCtx) -> AnyElement {
    let mut col = div().flex().flex_col().items_center();
    for ch in text.chars() {
        col = col.child(
            div()
                .text_color(to_hsla(ctx.style.colors.text))
                .text_size(px(ctx.style.typography.caption))
                .child(SharedString::from(ch.to_string())),
        );
    }
    col.into_any_element()
}
