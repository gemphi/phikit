//! Menubar anchor with stable geometry and resolved semantic styling.

use super::flex::BFlex;
use super::color::to_hsla;
use super::ctx::BRenderCtx;
use super::{brand, chrome};
use super::atom::icon_path;
use crate::v2::theme::BStyleRole;
use crate::v2::view::prim::BControl;
use gpui::prelude::*;
use gpui::{AnyElement, ClickEvent, SharedString, div, px, svg};
use std::rc::Rc;

/// A menu anchor is a button: the button height, centred on the strip's
/// line like every other control in it — the editor switcher, a pulldown
/// menu and an icon toggle are one height and one shape, as the approved
/// mock-ups draw them (Blender's pulldowns are `UI_UNIT_Y` buttons too).
fn resolved_height(ctx: &BRenderCtx) -> f32 {
    ctx.style.dimensions.button_height
}

pub fn view(ctl: &BControl, ctx: &BRenderCtx) -> AnyElement {
    let icon = ctl.icon.as_deref().unwrap_or("");
    let label = ctl.label.as_deref();
    let active = ctl.active;
    let action = ctl.action.as_ref();
    let hover = ctl.hover.as_ref();
    let r = px(ctx.style.radius_for(BStyleRole::MenuItem));
    // In a header strip a menu anchor is a cap cut into the strip like its
    // tabs (`BCapSide::Below`: the top corners rounded, the bottom flush with
    // the rule the popover drops from). Anywhere else — a dropdown in a
    // panel's row — it is a free control, rounded on every corner. One rule,
    // read off where the anchor stands (`BCapSide::of`).
    // A menu anchor is a control like every other: the control height,
    // centred, rounded on every corner — it clears the header's edges rather
    // than running into them, and its popover opens a gap below it.
    let cap = super::tab::cap::BCapSide::Free;
    let mut row = cap.round(div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(ctx.style.spacing.gap_sm))
        .h(px(resolved_height(ctx)))
        .px(px(ctx.style.spacing.pad_md)), r)
        .text_color(to_hsla(ctx.style.colors.text))
        .text_size(px(ctx.style.typography.body));
    // A property row's value column hands its whole width to one control, so
    // the dropdown there lines up with the label opposite it and with every
    // value below it rather than hugging its own longest option.
    row = if ctx.fill {
        row.w_full().min_w_0().justify_between()
    } else {
        row.flex_shrink_0()
    };

    if !icon.is_empty() {
        // The app's mark is not one more icon in the row: it is drawn larger
        // and in the brand's own two tones, so the head of the top bar reads
        // as the product rather than as another menu.
        //
        row = if ctl.brand {
            // The mark sits on the cap band's centre line with every other
            // control in the strip (`pane::BPaneCell`); it is not lifted.
            row.child(
                BFlex::row()
                    .keeps_size()
                    .div(ctx)
                    .child(brand::mark(ctx.style.dimensions.brand_size, "brand-mark", ctx)),
            )
        } else {
            row.child(
                svg()
                    .path(icon_path(icon))
                    .size(px(ctx.style.dimensions.icon_size))
                    .flex_shrink_0()
                    .text_color(to_hsla(ctx.style.colors.text)),
            )
        };
    }
    if let Some(text) = label.filter(|text| !text.is_empty()) {
        row = row.child(SharedString::from(text.to_string()));
    }
    let Some(action) = action else {
        return row.self_center().into_any_element();
    };
    let handler = ctx.handler.clone();
    let action_copy = action.clone();
    let id_text = super::button::element_id(icon, label, action);
    // The outer clickable owns the rounded top-cap and hard-clips its
    // children so a hover fill can never bleed past the corners (was the
    // "squarish on hover" bug — hover bg was painting a full rectangle).
    let element = div()
        .id(SharedString::from(id_text.clone()))
        .debug_selector(|| id_text)
        .cursor_pointer()
        // A cap is as wide as its label. Without this a greedy sibling in the
        // same pane crushed the menus to a few pixels each, which read as the
        // menu names overlapping.
        .flex_shrink_0()
        // Pinned to the strip's bottom — the pane keeps the gap above the
        // cap band — so the cap's own bottom edge is the header's rule and
        // the popover continues from it.
        .self_center()
        .overflow_hidden()
        .child(row);
    let element = cap.round(element, r);
    // Rest / hover / active come from the one strip treatment, so a menu, an
    // icon button and a tab in the same row cannot drift apart. Active is a
    // filled cap only — no accent strip — so the surface stays continuous
    // with the popover that drops beneath it.
    let mut element = chrome::paint(element, active, ctx);
    element = element.on_click(move |_event: &ClickEvent, window, app| {
        (Rc::clone(&handler))(&action_copy, window, app);
    });
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
