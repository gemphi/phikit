//! Menubar anchor with stable geometry and resolved semantic styling.

use super::super::{
    color::{resolved_gradient, to_hsla},
    ctx::BRenderCtx,
};
use super::icon_path;
use crate::v2::theme::BStyleRole;
use crate::v2::view::prim::{BAction, BControl};
use gpui::prelude::*;
use gpui::{AnyElement, ClickEvent, SharedString, div, px, svg};
use std::rc::Rc;

fn resolved_height(c: &BRenderCtx) -> f32 {
    c.style.dimensions.header_height - c.style.strokes.hairline
}

pub fn view(ctl: &BControl, c: &BRenderCtx) -> AnyElement {
    let icon = ctl.icon.as_deref().unwrap_or("");
    let label = ctl.label.as_deref();
    let active = ctl.active;
    let action = ctl.action.as_ref();
    let hover = ctl.hover.as_ref();
    let r = px(c.style.radius_for(BStyleRole::MenuItem));
    // Menu anchor row: only the top corners are rounded because the popover
    // drops from the bottom — the anchor and its popover form a continuous
    // surface with the top pair rounded and the bottom pair flush.
    let mut row = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(c.style.spacing.gap_sm))
        .h(px(resolved_height(c)))
        .px(px(c.style.spacing.pad_md))
        .rounded_t(r)
        .flex_shrink_0()
        .text_color(to_hsla(c.style.colors.text))
        .text_size(px(c.style.typography.body));

    if active {
        // Active state = filled cap only. No accent border strip — every
        // menu / tab uses fill-only selection so the surface stays
        // continuous with the popover / content that drops beneath it.
        row = row.bg(resolved_gradient(c.style.host_item_active_gradient(c.host)));
    }
    if !icon.is_empty() {
        row = row.child(
            svg()
                .path(icon_path(icon))
                .size(px(c.style.typography.header))
                .flex_shrink_0()
                .text_color(to_hsla(c.style.colors.text)),
        );
    }
    if let Some(text) = label.filter(|text| !text.is_empty()) {
        row = row.child(SharedString::from(text.to_string()));
    }
    let Some(action) = action else {
        return row.into_any_element();
    };
    let handler = c.h.clone();
    let action_copy = action.clone();
    let id_text = super::button::element_id(icon, label, action);
    let hover_fill = to_hsla(c.style.host_item_hover(c.host));
    // The outer clickable owns the rounded top-cap and hard-clips its
    // children so a hover fill can never bleed past the corners (was the
    // "squarish on hover" bug — hover bg was painting a full rectangle).
    let mut element = div()
        .id(SharedString::from(id_text.clone()))
        .debug_selector(|| id_text)
        .cursor_pointer()
        .rounded_t(r)
        .overflow_hidden()
        .child(row);
    if !active {
        element = element.hover(move |style| style.bg(hover_fill));
    }
    element = element.on_click(move |_event: &ClickEvent, window, app| {
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
