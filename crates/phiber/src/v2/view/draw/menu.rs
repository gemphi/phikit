//! Menu-popover row — flat by default, hover-only highlight, matches
//! Blender's `layout.operator` rows. Optional `hover` action fires when
//! the pointer enters the row (submenu anchors use it to open on hover).

use super::color::to_hsla;
use super::ctx::BRenderCtx;
use super::chrome;
use super::atom::icon_path;
use super::flex::{BAlign, BDim, BFill, BFlex, BFlexed, BLen, BOverflow, BRadius, BSpace, BType};
use crate::v2::theme::BStyleRole;
use crate::v2::view::prim::BControl;
use gpui::prelude::*;
use gpui::{AnyElement, ClickEvent, SharedString, px, svg};
use std::rc::Rc;

pub fn view(ctl: &BControl, ctx: &BRenderCtx) -> AnyElement {
    let ic = ctl.icon.as_deref().unwrap_or("");
    let lbl = ctl.label.as_deref().unwrap_or("");
    let sc = ctl.shortcut.as_deref().unwrap_or("");
    let action = ctl.action.as_ref();
    let hover = ctl.hover.as_ref();
    // A menu row is a strip control: flat at rest, filled on hover or when
    // active. A resting fill turned every row into a pill and made a menu
    // look nothing like the toolbar and sidebar it sits next to.
    let mut row = BMenuPart::Row.flex(ctx).div(ctx);
    // Every row reserves the icon slot, drawn or not, so labels start on one
    // line down the whole menu. A row without an icon used to start its label
    // where the icon would have been, which put the labels of one menu on two
    // different left edges — Blender reserves the slot the same way.
    let slot = px(ctx.style.dimensions.icon_size_sm);
    let icon = BMenuPart::IconSlot.flex(ctx).div(ctx);
    row = row.child(if ic.is_empty() {
        icon
    } else {
        icon.child(
            svg()
                .path(icon_path(ic))
                .size(slot)
                .text_color(to_hsla(ctx.style.colors.text_dim)),
        )
    });
    // The label hugs the slot and reads left-aligned; `flex_1` lets the
    // shortcut sit at the far end.
    row = row.child(
        BMenuPart::Label
            .flex(ctx)
            .div(ctx)
            .text_left()
            .child(SharedString::from(lbl.to_string())),
    );
    if !sc.is_empty() {
        row = row.child(
            BMenuPart::Shortcut.flex(ctx).div(ctx).child(SharedString::from(sc.to_string())),
        );
    }
    let radius = px(ctx.style.radius_for(BStyleRole::MenuItem));
    let Some(a) = action else {
        return row.into_any_element();
    };
    let ch = ctx.handler.clone();
    let ac = a.clone();
    let id = SharedString::from(format!("menu-{}", lbl));
    // Rest / hover / active come from the one strip treatment, so a menu row
    // and the anchor that opened it share a shape and a fill. Not `floating`:
    // the popover panel is the floating surface; its rows sit inside it.
    // A menu row rounds as the anchor above it does — the top corners only,
    // the one cap rule every strip control shares (`BCapSide::Below`).
    let element = BMenuPart::Cap(f32::from(radius))
        .flex(ctx)
        .div(ctx)
        .id(id.clone())
        .debug_selector(move || id.to_string())
        .cursor_pointer()
        .child(row);
    let mut element = chrome::paint(element, ctl.active, ctx).on_click(
            move |_e: &ClickEvent, w: &mut gpui::Window, cx: &mut gpui::App| {
                (Rc::clone(&ch))(&ac, w, cx);
            },
        );
    if let Some(h) = hover {
        let hh = ctx.handler.clone();
        let ha = h.clone();
        element = element.on_hover(
            move |hovered: &bool, w: &mut gpui::Window, cx: &mut gpui::App| {
                if *hovered {
                    (Rc::clone(&hh))(&ha, w, cx);
                }
            },
        );
    }
    element.into_any_element()
}

pub fn sep(ctx: &BRenderCtx) -> AnyElement {
    BMenuPart::Separator.flex(ctx).div(ctx).into_any_element()
}

/// The parts of a menu row, in the order they are laid out.
///
/// One enum for the level, as every other level states its parts
/// (`BCardPart`, `BModalPart`, `BLogPart`): the parts of one thing are
/// variants of one type, matched once, not a unit struct each.
enum BMenuPart {
    /// A menu row is a strip control: flat at rest, filled on hover or when
    /// active (`chrome::paint`). A resting fill turned every row into a pill
    /// and made a menu look nothing like the toolbar and sidebar beside it.
    Row,
    /// Every row reserves the icon slot, drawn or not, so labels start on one
    /// line down the whole menu. A row without an icon used to start its
    /// label where the icon would have been, which put one menu's labels on
    /// two different left edges — Blender reserves the slot the same way.
    IconSlot,
    /// The label hugs the slot and reads left-aligned; taking the slack is
    /// what lets the shortcut sit at the far end.
    Label,
    /// The shortcut at the row's end, a step quieter than the label.
    Shortcut,
    /// A menu row rounds as the anchor above it does — the top corners only,
    /// the one cap rule every strip control shares (`BCapSide::Below`) — and
    /// clips, so a hover fill can never bleed past them.
    Cap(f32),
    /// A hairline divider: the stroke width is the thinnest a row can be.
    Separator,
}

impl BFlexed for BMenuPart {
    fn flex(&self, ctx: &BRenderCtx) -> BFlex {
        match self {
            Self::Row => BFlex::row()
                .width(BLen::Full)
                .height(BLen::Dim(BDim::RowHeight))
                .align(BAlign::Center)
                .gap(BSpace::GapSm)
                .pad_x(BSpace::PadSm)
                .ink()
                .type_size(BType::Body),
            Self::IconSlot => {
                let slot = BLen::Measured(ctx.style.dimensions.icon_size_sm);
                BFlex::row().width(slot).height(slot).keeps_size()
            }
            Self::Label => BFlex::row().fills().min_width(BLen::Zero),
            Self::Shortcut => BFlex::row().dim_ink().type_size(BType::Caption),
            Self::Cap(radius) => BFlex::row()
                .width(BLen::Full)
                .radius(BRadius::Top(*radius))
                .overflow(BOverflow::Clip),
            Self::Separator => BFlex::row()
                .width(BLen::Full)
                .height(BLen::Measured(ctx.style.strokes.hairline))
                .margin_y(BSpace::GapXs)
                .fill(BFill::Separator),
        }
    }
}
