//! Button atom with stable action identity and resolved semantic styling.

use super::color::to_hsla;
use super::ctx::BRenderCtx;
use super::chrome;
use super::flex::{BFlex, BLen};
use super::atom::icon_path;
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
        BAction::Adjust(path, value) => format!("adjust-{}-{value}", path.encode()),
        BAction::Toggle(path) => format!("toggle-{}", path.encode()),
        BAction::Open(path) => format!("open-{}", path.encode()),
        BAction::Reset(path) => format!("reset-{}", path.encode()),
    };
    format!(
        "btn-{role}-{action_id}-{}",
        if role == "icon" { icon } else { "text" }
    )
}

pub fn view(ctl: &BControl, ctx: &BRenderCtx) -> AnyElement {
    let icon = ctl.icon.as_deref().unwrap_or("");
    let label = ctl.label.as_deref();
    let action = ctl.action.as_ref();
    let hover = ctl.hover.as_ref();
    // In a strip every control is a menu, so it takes the menu radius too.
    let radius = px(ctx.style.radius_for(if chrome::in_strip(ctx) {
        BStyleRole::MenuItem
    } else {
        BStyleRole::Button
    }));
    // A control not in force is dimmed, its icon and text alike (`muted`).
    let ink = to_hsla(if ctl.muted { ctx.style.colors.text_dim } else { ctx.style.colors.text });
    let mut row = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(ctx.style.spacing.gap_sm))
        // In a strip every control is a menu, so it takes the menu's own
        // horizontal padding. Mixing pad_sm and pad_md meant the hover fill
        // sat tighter on an icon than on the menu beside it.
        .px(px(if chrome::in_strip(ctx) {
            ctx.style.spacing.pad_md
        } else {
            ctx.style.spacing.pad_sm
        }))
        .h(px(ctx.style.dimensions.button_height))
        .rounded(radius)
        .text_color(ink)
        .text_size(px(ctx.style.typography.body));
    // A labelled button outside a strip is a row of whatever holds it — a
    // bookmark in the source list, a link on the welcome card, an option in
    // a panel — so it runs the parent's width and its active fill lights the
    // whole row, the way a rail's active tab does. In a strip it hugs like
    // every other control. A property row's value column also fills, with
    // the label leading and the hint trailing.
    let is_row = label.is_some() && !chrome::in_strip(ctx);
    if ctx.fill {
        row = row.w_full().min_w_0().justify_between();
    } else if is_row {
        row = row.w_full().min_w_0().justify_start();
    }
    if !icon.is_empty() {
        row = row.child(
            svg()
                .path(icon_path(icon))
                .size(px(ctx.style.dimensions.icon_size))
                .flex_shrink_0()
                .text_color(ink),
        );
    }
    if let Some(text) = label {
        row = row.child(SharedString::from(text.to_string()));
    }
    let Some(action) = action else {
        return row.into_any_element();
    };
    let handler = ctx.handler.clone();
    let action_copy = action.clone();
    let id_text = element_id(icon, label, action);
    // overflow_hidden clips the hover fill to the rounded shape — without
    // it the hover paint drew a full rectangle over the rounded corners
    // (the "squarish on hover" bug).
    // A control spanning several slots is as wide as those icon buttons and
    // the gaps between them, so the row keeps its places (`BControl::span`).
    // A control filling its slot fills it as a whole: the hover shape runs
    // the column's width, so every checkbox in a popover's value column is
    // one width, not a pill cut to its own label. The row inside already
    // filled; this wrapper hugged, so the column's width never reached it.
    let slot = 2.0 * ctx.style.spacing.pad_md + ctx.style.dimensions.icon_size;
    let spans = f32::from(ctl.span.max(1));
    let base = match (ctx.fill, ctl.span > 1) {
        (true, _) => BFlex::row().width(BLen::Full).min_width(BLen::Zero).div(ctx),
        (false, true) => BFlex::row().min_width(BLen::Measured(spans * slot + (spans - 1.0) * ctx.style.spacing.gap_xs)).div(ctx),
        (false, false) => div(),
    };
    let mut element = base
        .id(SharedString::from(id_text.clone()))
        .debug_selector(|| id_text)
        .cursor_pointer()
        .rounded(radius)
        .overflow_hidden()
        .child(row);
    if is_row {
        element = element.w_full();
    }
    if chrome::in_strip(ctx) {
        // Shorter than the strip, so it sits on the strip's centre line while
        // the full-height caps beside it stay pinned to the bottom edge.
        element = element.self_center();
    }
    let mut element = // On AND not in force do not stack: the fill says "in force", so a
    // control that is muted does not take it however true its state is. The
    // X-ray button was painting a full active fill with dim ink over it —
    // "on" and "unavailable" at once, with no rule for what that should look
    // like (`part/modebar/shading.rs`).
    chrome::paint(element, ctl.active && !ctl.muted, ctx).on_click(
        move |_event: &ClickEvent, window, app| {
            (Rc::clone(&handler))(&action_copy, window, app);
        },
    );
    if let Some(hover_action) = hover {
        let handler = ctx.handler.clone();
        let action = hover_action.clone();
        element = element.on_hover(move |hovered, window, app| {
            if *hovered {
                (Rc::clone(&handler))(&action, window, app);
            }
        });
    }
    element.into_any_element()
}
