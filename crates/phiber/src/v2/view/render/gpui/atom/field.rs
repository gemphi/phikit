//! Field atom — a bordered value read-out used inside property rows.
//! Blender ref: `uiItemR` renders numeric/enum/string RNA props as slim
//! rounded boxes with a subtle bg and centre-aligned value text.

use super::super::{color::to_hsla, ctx::BRenderCtx};
use crate::v2::theme::BStyleRole;
use gpui::prelude::*;
use gpui::{AnyElement, SharedString, div, px};

pub fn view(text: &str, c: &BRenderCtx) -> AnyElement {
    div()
        .flex()
        .flex_1()
        .flex_row()
        .items_center()
        .justify_center()
        .h(px(c.style.dimensions.row_height))
        .px(px(c.style.spacing.pad_sm))
        .rounded(px(c.style.radius_for(BStyleRole::Field)))
        .bg(to_hsla(c.style.colors.bg_hover))
        .border_1()
        .border_color(to_hsla(c.style.colors.border))
        .text_color(to_hsla(c.style.colors.text))
        .text_size(px(c.style.typography.body))
        .child(SharedString::from(text.to_string()))
        .into_any_element()
}
