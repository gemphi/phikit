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
use super::flex::{BAlign, BFlex, BFlexed, BLen, BOverflow, BSpace, BType};
use super::atom::icon_path;
use super::text_input::BFieldText;
use crate::v2::view::prim::BAction;
use gpui::prelude::*;
use gpui::{AnyElement, ClickEvent, MouseButton, MouseDownEvent, SharedString, px, svg};
use std::rc::Rc;

/// The search field's own containers, named once.
///
/// The typing field and the plain one built the SAME row and the SAME text
/// cell — `flex row, items_center, gap_sm, flex_1` and `flex_1, min_w_0,
/// overflow_hidden` — in two places, which is sixteen hand-written style
/// calls for two containers and two chances for them to drift apart.
enum BSearchPart {
    /// The field's row: the lead icon, then the text, filling the shell.
    Row,
    /// The text cell: takes what the icon leaves and clips rather than
    /// pushing the icon out when the text is long.
    TextCell,
}

impl BFlexed for BSearchPart {
    fn flex(&self, _ctx: &BRenderCtx) -> BFlex {
        match self {
            Self::Row => BFlex::row().align(BAlign::Center).gap(BSpace::GapSm).grows(),
            Self::TextCell => BFlex::cell().grows().min_width(BLen::Zero).overflow(BOverflow::Clip),
        }
    }
}


/// The search icon that leads the field, tinted `color`.
fn lead_icon(color: crate::v2::theme::BColor, ctx: &BRenderCtx) -> impl IntoElement {
    svg()
        .path(icon_path("search"))
        .size(px(ctx.style.dimensions.icon_size_sm))
        .flex_shrink_0()
        .text_color(to_hsla(color))
}

pub fn view(
    placeholder: &str,
    value: &str,
    editing: bool,
    on_change: &BAction,
    ctx: &BRenderCtx,
) -> AnyElement {
    // Being typed into, with a real window behind the pass: the field draws its
    // caret, selection and IME composition through `EntityInputHandler`
    // (`BFieldText`), and a click places the caret.
    if editing
        && let Some(field_text) = BFieldText::of(value, placeholder, ctx)
    {
        let app = field_text.app.clone();
        let inner = BSearchPart::Row
            .flex(ctx)
            .div(ctx)
            .child(lead_icon(ctx.style.colors.text, ctx))
            .child(BSearchPart::TextCell.flex(ctx).div(ctx).child(field_text));
        let id_str = element_id("search", Some(placeholder), on_change);
        return field::shell(true, ctx)
            .id(SharedString::from(id_str.clone()))
            .debug_selector(|| id_str)
            .cursor_text()
            .child(inner)
            .on_mouse_down(MouseButton::Left, move |ev: &MouseDownEvent, _window, cx| {
                let (pos, shift) = (ev.position, ev.modifiers.shift);
                app.update(cx, |app, cx| {
                    app.place_caret_at_point(pos, shift);
                    cx.notify();
                });
            })
            .into_any_element();
    }

    let is_empty = value.trim().is_empty();
    let display_text = match (editing, is_empty) {
        // Caret fallback for a pass with no window behind it (headless render):
        // the pipe stands in for the caret the element would draw.
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

    let inner = BSearchPart::Row
        .flex(ctx)
        .div(ctx)
        // Search icon on the left, matching Blender's `ICON_VIEWZOOM`.
        .child(lead_icon(text_color, ctx))
        .child(
            BSearchPart::TextCell
                .flex(ctx)
                .type_size(BType::Body)
                .div(ctx)
                .text_color(to_hsla(text_color))
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
