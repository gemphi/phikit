//! Field atom — a bordered value read-out used inside property rows.
//! Blender ref: `uiItemR` renders numeric/enum/string RNA props as slim
//! rounded boxes with a subtle bg and centre-aligned value text.

use super::color::to_hsla;
use super::ctx::BRenderCtx;
use crate::v2::theme::BStyleRole;
use gpui::prelude::*;
use gpui::{AnyElement, SharedString, div, px};

/// The box every input shares: one rounded, clipped surface that owns its
/// fill and its border. Read-outs and typed fields differ only in what they
/// put inside it, so an input can never end up with a second, square box
/// painted behind its rounded one.
pub fn shell(editing: bool, ctx: &BRenderCtx) -> gpui::Stateful<gpui::Div> {
    let border = if editing {
        ctx.style.colors.accent
    } else {
        ctx.style.colors.border
    };
    div()
        .id(SharedString::from("field-shell"))
        .flex()
        .flex_1()
        .flex_row()
        .items_center()
        .min_w_0()
        // The fill, the focus border and any hover all stop at the rounded
        // edge, so nothing bleeds onto the strip behind the field.
        .overflow_hidden()
        .h(px(ctx.style.dimensions.row_height))
        // A text margin either side (Blender's `UI_TEXT_MARGIN_X`, 0.4 of a
        // unit): the text — and where it clips — stops short of the edge.
        .px(px(ctx.style.spacing.pad_md))
        .rounded(px(ctx.style.radius_for(BStyleRole::Field)))
        .bg(to_hsla(ctx.style.colors.bg_control))
        .border(px(ctx.style.strokes.border))
        .border_color(to_hsla(border))
        .text_color(to_hsla(ctx.style.colors.text))
        .text_size(px(ctx.style.typography.body))
}

pub fn view(text: &str, ctx: &BRenderCtx) -> AnyElement {
    shell(false, ctx)
        .justify_center()
        .child(SharedString::from(text.to_string()))
        .into_any_element()
}
