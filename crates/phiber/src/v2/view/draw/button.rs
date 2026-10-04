//! Button atom with stable action identity and resolved semantic styling.

use super::color::to_hsla;
use super::ctx::BRenderCtx;
use super::chrome;
use super::flex::{BDim, BEdge, BFill, BFlex, BJustify, BLen, BRadius};
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
        // Its own prefix, so a capture distinguishes a stepper from a chooser.
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

/// A swatch button's face: a square of the colour, bordered so a colour close to
/// the panel's own shade still reads as a swatch rather than as a hole.
fn swatch_face(color: crate::v2::theme::BColor, ctx: &BRenderCtx) -> BFlex {
    // An icon's size, the way the zone glyph states it: there is no `BDim` for it.
    let size = BLen::Measured(ctx.style.dimensions.icon_size);
    BFlex::cell()
        .width(size)
        .height(size)
        .keeps_size()
        .radius(BRadius::Role(BStyleRole::Field))
        .border(BEdge::All)
        .fill(BFill::SwatchHsl(color))
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
    // A control not in force is faded, its icon and text alike (`muted`) — the
    // same ink at `control_muted_alpha`, not a different colour. It used to take
    // `text_dim`, which is the shade an INACTIVE tab's label takes, so "off" and
    // "not available" were one look; a weight on the normal ink keeps them two.
    let ink = to_hsla(ctx.style.colors.text);
    let ink = if ctl.muted {
        gpui::Hsla { a: ink.a * ctx.style.effects.control_muted_alpha, ..ink }
    } else {
        ink
    };
    let mut row = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(ctx.style.spacing.gap_sm))
        .px(px(if label.is_none() {
            // An icon-only button is a SQUARE, everywhere, and that is decided
            // here rather than per host.
            //
            // Its height has always been `button_height`; its width was its
            // padding twice plus an icon, and the padding differs inside a
            // strip and out (`pad_md` against `pad_sm`). So the same transport
            // button came out one width in the Timeline's header and another
            // in a panel, and every icon rail disagreed with every other about
            // how wide an icon is. Blender has no such drift: `UI_UNIT_X` and
            // `UI_UNIT_Y` are both `U.widget_unit`, so an icon button is a unit
            // square by construction.
            //
            // The centring below does the rest: with no label there is nothing
            // for the padding to separate.
            0.0
        } else if chrome::in_strip(ctx) {
            // In a strip every control is a menu, so it takes the menu's own
            // horizontal padding. Mixing pad_sm and pad_md meant the hover fill
            // sat tighter on an icon than on the menu beside it.
            ctx.style.spacing.pad_md
        } else {
            ctx.style.spacing.pad_sm
        }))
        .h(px(ctx.style.dimensions.button_height))
        .rounded(radius)
        .text_color(ink)
        .text_size(px(ctx.style.typography.body));
    if label.is_none() {
        // Said as a `BFlex`, not three raw calls: the style budget refused the
        // raw version and was right to — a width and a centring are exactly
        // what the design system is for, and it already carries both.
        row = BFlex::cell()
            .width(BLen::Dim(BDim::ButtonHeight))
            .keeps_size()
            .justify(BJustify::Center)
            .styled(row, ctx);
    }
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
    // A swatch button's face is the colour itself, in the icon's slot, because
    // that is what this control shows. Stated as a `BFlex` rather than hand-rolled
    // divs: the style budget caught the hand-rolled version, and it was right —
    // `BCtrlPart::Swatch` describes the same square the same way.
    if let Some(color) = ctl.swatch {
        row = row.child(swatch_face(color, ctx).div(ctx));
    } else if !icon.is_empty() {
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
    let slot = 2.0 * ctx.style.spacing.pad_md + ctx.style.dimensions.icon_size;
    let spans = f32::from(ctl.span.max(1));
    let base = if ctl.span > 1 {
        BFlex::row().min_width(BLen::Measured(spans * slot + (spans - 1.0) * ctx.style.spacing.gap_xs)).div(ctx)
    } else {
        div()
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
    // On AND not in force now stack, because there IS a rule for what that
    // looks like: muted is a weight on whatever the control paints, so the fill
    // still says "on" and the fade says "not here". This used to pass
    // `active && !muted`, which dropped the fill entirely — the X-ray button in
    // Material Preview is on, and it painted exactly like one that is off
    // (`part/modebar/shading.rs`, `tests/a_muted_control_still_shows_its_state.rs`).
    let mut element = chrome::paint_state(element, ctl.active, ctl.muted, ctx).on_click(
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
