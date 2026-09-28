//! Search-field atom — a visually interactive text filter field.
//!
//! Blender ref: `UI_BTYPE_TEXT` / `uiItemStringO` search props in header
//! regions (`space_outliner.py` `filter_text`, `space_filebrowser.py`
//! `filter_search`).
//!
//! # Typing
//!
//! Clicking the field puts the caret in it (`state::edit`), which the window
//! root reads to send every key press to this field's state key. The field
//! shows the caret and an accent border while it has it; Return confirms,
//! Escape restores the value it started with.

use crate::v2::state::{BKey as _};
use super::color::to_hsla;
use super::ctx::BRenderCtx;
use super::button::element_id;
use super::field;
use super::atom::icon_path;
use crate::v2::view::prim::BAction;
use gpui::prelude::*;
use gpui::{AnyElement, ClickEvent, SharedString, div, px, svg};
use std::rc::Rc;

pub fn view(
    placeholder: &str,
    value: &str,
    editing: bool,
    on_change: &BAction,
    ctx: &BRenderCtx,
) -> AnyElement {
    let is_empty = value.trim().is_empty();
    let display_text = match (editing, is_empty) {
        // The caret sits after the text while typing (Blender draws one too).
        (true, _) => format!("{value}|"),
        (false, true) => placeholder.to_string(),
        (false, false) => value.to_string(),
    };

    // Dim the text colour for placeholder mode.
    let text_color = if is_empty && !editing {
        let mut col = ctx.style.colors.text;
        col.3 = (col.3 * 0.45).clamp(0.0, 1.0);
        col
    } else {
        ctx.style.colors.text
    };

    // Clicking puts the caret here: `ctx.edit` names the key the window's key
    // handler types into (`state::edit`).
    let focus_action = match on_change {
        BAction::Set(path, _) => Some(BAction::Set(crate::v2::state::AppKey::Edit.path(), path.encode())),
        _ => None,
    };

    let inner = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(ctx.style.spacing.gap_sm))
        .flex_1()
        // Search icon on the left, matching Blender's `ICON_VIEWZOOM`.
        .child(
            svg()
                .path(icon_path("search"))
                .size(px(ctx.style.dimensions.icon_size_sm))
                .flex_shrink_0()
                .text_color(to_hsla(text_color)),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .overflow_hidden()
                .text_color(to_hsla(text_color))
                .text_size(px(ctx.style.typography.body))
                .child(SharedString::from(display_text)),
        );

    // Wire the click handler when the field names a key to type into.
    let Some(action) = focus_action else {
        return field::shell(false, ctx).child(inner).into_any_element();
    };
    let handler = ctx.handler.clone();
    let action_clone = action.clone();
    let id_str = element_id("search", Some(placeholder), &action);
    // One box, not a box inside a box: the clickable IS the field, so its
    // rounding and its clip are the ones the fill and the focus border obey.
    // Nesting them is what let the hover fill paint a sharp-cornered
    // rectangle around the rounded field (`tests/chrome_geometry.rs`).
    field::shell(editing, ctx)
        .id(SharedString::from(id_str.clone()))
        .debug_selector(|| id_str)
        .cursor_pointer()
        .child(inner)
        .on_click(move |_: &ClickEvent, window, app| {
            (Rc::clone(&handler))(&action_clone, window, app);
        })
        .into_any_element()
}
