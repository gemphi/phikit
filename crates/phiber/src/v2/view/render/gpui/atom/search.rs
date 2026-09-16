//! Search-field atom — a visually interactive text filter field.
//!
//! Blender ref: `UI_BTYPE_TEXT` / `uiItemStringO` search props in header
//! regions (`space_outliner.py` `filter_text`, `space_filebrowser.py`
//! `filter_search`).
//!
//! # Text entry approach
//!
//! Full IME-aware text entry requires implementing `EntityInputHandler` on a
//! GPUI `Entity<V>` — which is outside phiber's stateless element model.
//! This atom therefore provides Phase-1 support:
//!   - Displays the current filter value from `BState`.
//!   - Shows a dimmed placeholder when the value is empty.
//!   - Fires `on_change` with `""` on **click** to clear the active filter
//!     (the most common search interaction: clear and retype).
//!
//! Phase-2 (full keystroke capture) can be added by introducing an
//! `EntityInputHandler` view in `v2/app/search_input.rs` and wiring it
//! through the existing `handle_action` path.

use super::super::{color::to_hsla, ctx::BRenderCtx};
use super::{button::element_id, icon_path};
use crate::v2::theme::BStyleRole;
use crate::v2::view::prim::BAction;
use gpui::prelude::*;
use gpui::{AnyElement, ClickEvent, SharedString, div, px, svg};
use std::rc::Rc;

pub fn view(
    placeholder: &str,
    value: &str,
    on_change: &BAction,
    c: &BRenderCtx,
) -> AnyElement {
    let is_empty = value.trim().is_empty();
    let display_text = if is_empty {
        placeholder.to_string()
    } else {
        value.to_string()
    };

    // Dim the text colour for placeholder mode.
    let text_color = if is_empty {
        let mut col = c.style.colors.text;
        col.3 = (col.3 * 0.45).clamp(0.0, 1.0);
        col
    } else {
        c.style.colors.text
    };

    // The clear action: clicking fires BAction::Set(path, "") to reset the
    // filter. Only wire the click when on_change is a Set action.
    let clear_action = match on_change {
        BAction::Set(path, _) => Some(BAction::Set(path.clone(), String::new())),
        _ => None,
    };

    let inner = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(c.style.spacing.gap_sm))
        .flex_1()
        // Search icon on the left, matching Blender's `ICON_VIEWZOOM`.
        .child(
            svg()
                .path(icon_path("search"))
                .size(px(c.style.typography.body))
                .flex_shrink_0()
                .text_color(to_hsla(text_color)),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .overflow_hidden()
                .text_color(to_hsla(text_color))
                .text_size(px(c.style.typography.body))
                .child(SharedString::from(display_text)),
        );

    // Wrapper with the same visual treatment as `field.rs`.
    let wrapper = div()
        .flex()
        .flex_1()
        .flex_row()
        .items_center()
        .h(px(c.style.dimensions.row_height))
        .px(px(c.style.spacing.pad_sm))
        .rounded(px(c.style.radius_for(BStyleRole::Field)))
        .bg(to_hsla(c.style.colors.bg_hover))
        .border_1()
        .border_color(to_hsla(c.style.colors.border))
        .child(inner);

    // Wire the click handler when we have a valid clear action.
    if let Some(action) = clear_action {
        let handler = c.h.clone();
        let action_clone = action.clone();
        let id_str = element_id("search", Some(placeholder), &action);
        div()
            .id(SharedString::from(id_str.clone()))
            .debug_selector(|| id_str)
            .cursor_pointer()
            .flex_1()
            .child(wrapper)
            .hover(|s| s.bg(to_hsla(c.style.colors.bg_hover)))
            .on_click(move |_: &ClickEvent, window, app| {
                (Rc::clone(&handler))(&action_clone, window, app);
            })
            .into_any_element()
    } else {
        div().flex_1().child(wrapper).into_any_element()
    }
}
