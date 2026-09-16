//! Button atom with stable action identity and resolved semantic styling.

use super::super::{color::to_hsla, ctx::BRenderCtx};
use super::icon_path;
use crate::v2::theme::BStyleRole;
use crate::v2::view::prim::{BAction, BControl};
use gpui::prelude::*;
use gpui::{AnyElement, ClickEvent, SharedString, div, px, svg};
use std::rc::Rc;

pub fn element_id(icon: &str, label: Option<&str>, action: &BAction) -> String {
    let role = if label.is_some_and(|text| !text.is_empty()) {
        "label"
    } else {
        "icon"
    };
    let action_id = match action {
        BAction::Set(path, value) => format!("set-{}-{value}", path.encode()),
        BAction::Toggle(path) => format!("toggle-{}", path.encode()),
        BAction::Open(path) => format!("open-{}", path.encode()),
        BAction::Reset(path) => format!("reset-{}", path.encode()),
    };
    format!(
        "btn-{role}-{action_id}-{}",
        if role == "icon" { icon } else { "text" }
    )
}

pub fn view(ctl: &BControl, c: &BRenderCtx) -> AnyElement {
    let icon = ctl.icon.as_deref().unwrap_or("");
    let label = ctl.label.as_deref();
    let action = ctl.action.as_ref();
    let hover = ctl.hover.as_ref();
    let radius = px(c.style.radius_for(BStyleRole::Button));
    let mut row = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(c.style.spacing.gap_sm))
        .px(px(c.style.spacing.pad_sm))
        .h(px(c.style.dimensions.button_height))
        .rounded(radius)
        .text_color(to_hsla(c.style.colors.text))
        .text_size(px(c.style.typography.body));
    if !icon.is_empty() {
        row = row.child(
            svg()
                .path(icon_path(icon))
                .size(px(c.style.typography.header))
                .flex_shrink_0()
                .text_color(to_hsla(c.style.colors.text)),
        );
    }
    if let Some(text) = label {
        row = row.child(SharedString::from(text.to_string()));
    }
    let Some(action) = action else {
        return row.into_any_element();
    };
    let handler = c.h.clone();
    let action_copy = action.clone();
    let id_text = element_id(icon, label, action);
    // overflow_hidden clips the hover fill to the rounded shape — without
    // it the hover paint drew a full rectangle over the rounded corners
    // (the "squarish on hover" bug).
    let mut element = div()
        .id(SharedString::from(id_text.clone()))
        .debug_selector(|| id_text)
        .cursor_pointer()
        .rounded(radius)
        .overflow_hidden()
        .child(row)
        .hover(|style| style.bg(to_hsla(c.style.colors.bg_hover)))
        .on_click(move |_event: &ClickEvent, window, app| {
            (Rc::clone(&handler))(&action_copy, window, app);
        });
    if let Some(hover_action) = hover {
        let handler = c.h.clone();
        let action = hover_action.clone();
        element = element.on_hover(move |hovered, window, app| {
            if *hovered {
                (Rc::clone(&handler))(&action, window, app);
            }
        });
    }
    element.into_any_element()
}
