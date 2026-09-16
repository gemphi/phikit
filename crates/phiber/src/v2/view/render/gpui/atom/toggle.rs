//! Stable, rounded Topbar controls for canonical zone visibility.

use super::super::{color::to_hsla, ctx::BRenderCtx};
use super::icon_path;
use crate::v2::theme::BStyleRole;
use crate::v2::view::BZoneToggle;
use gpui::prelude::*;
use gpui::{AnyElement, ClickEvent, SharedString, div, px, svg};
use std::rc::Rc;

pub fn icon_name(control: &BZoneToggle) -> String {
    let base = control.zone.icon();
    if control.visible {
        base.to_string()
    } else {
        format!("{base}_outline")
    }
}

fn control(control: &BZoneToggle, c: &BRenderCtx) -> AnyElement {
    let size = c.style.dimensions.button_height;
    let radius = px(c.style.radius_for(BStyleRole::Button));
    let id_text = format!("topbar-zone-toggle-{}", control.zone.slug());
    let handler = c.h.clone();
    let action = control.action.clone();
    let hover = to_hsla(c.style.colors.bg_hover);
    let color = if control.visible {
        c.style.colors.accent
    } else {
        c.style.colors.text_dim
    };
    div()
        .id(SharedString::from(id_text.clone()))
        .debug_selector(move || id_text.clone())
        .flex()
        .items_center()
        .justify_center()
        .flex_shrink_0()
        .w(px(size))
        .h(px(size))
        .rounded(radius)
        .cursor_pointer()
        .hover(move |style| style.bg(hover).rounded(radius))
        .child(
            svg()
                .path(icon_path(&icon_name(control)))
                .size(px(c.style.typography.header))
                .flex_shrink_0()
                .text_color(to_hsla(color)),
        )
        .on_click(
            move |_event: &ClickEvent, window: &mut gpui::Window, cx: &mut gpui::App| {
                (Rc::clone(&handler))(&action, window, cx);
            },
        )
        .into_any_element()
}

pub fn group(controls: &[BZoneToggle], c: &BRenderCtx) -> AnyElement {
    let selector = "topbar-zone-toggle-group";
    let mut group = div()
        .id(selector)
        .debug_selector(|| selector.to_string())
        .flex()
        .flex_row()
        .items_center()
        .justify_end()
        .flex_shrink_0()
        .gap(px(c.style.spacing.gap_xs));
    for item in controls {
        group = group.child(control(item, c));
    }
    group.into_any_element()
}
