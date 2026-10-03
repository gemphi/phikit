//! Menu-popover row — flat by default, hover-only highlight, matches
//! Blender's `layout.operator` rows. Optional `hover` action fires when
//! the pointer enters the row (submenu anchors use it to open on hover).

use super::color::to_hsla;
use super::ctx::BRenderCtx;
use super::chrome;
use super::atom::icon_path;
use super::flex::{BAlign, BDim, BFill, BFlex, BFlexed, BJustify, BLen, BOverflow, BRadius, BSpace, BType};
use crate::v2::theme::{BStyleRole, BSurfaceTone};
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
    let icon = BMenuPart::IconSlot.flex(ctx).div(ctx);
    row = row.child(if ic.is_empty() { icon } else { icon.child(glyph(ic, ctx)) });
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
    // A row that leads to a menu of its own ends with an arrow, in a column
    // of its own after the shortcut. It used to BE the shortcut — the string
    // `"▶"` passed as one — so a submenu could never show its key binding and
    // the arrow was drawn in the shortcut's caption type. It is an icon now,
    // at the icon slot's size, and the shortcut column is free.
    if ctl.submenu {
        row = row.child(BMenuPart::Submenu.flex(ctx).div(ctx).child(glyph("chevron_right", ctx)));
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
    // A menu row rounds on every corner: it floats inside the popover and
    // meets no edge, unlike a header tab.
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

/// The one glyph a menu row draws: the check or icon at its head, the arrow at
/// its tail when it opens a menu of its own. One builder, so the two are the
/// same size and the same shade — and so the file spends its raw styling once
/// rather than once per glyph (`tests/style_budget.rs`).
fn glyph(name: &str, ctx: &BRenderCtx) -> gpui::Svg {
    svg().path(icon_path(name)).size(px(ctx.style.dimensions.icon_size_sm)).text_color(to_hsla(ctx.style.colors.text_dim))
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
    /// The arrow on a row that opens a menu of its own, after the shortcut.
    ///
    /// Its own column, because the arrow and a key binding are two different
    /// things a row can have to say. Sized like the icon slot so the arrow at
    /// one end and the glyph at the other are the same weight.
    Submenu,
    /// A menu row rounds on every corner, and clips, so a hover fill can
    /// never bleed past them.
    ///
    /// Not the top-only cap rule: a tab rounds at the top because it meets
    /// its header's bottom edge and has to sit flush against it. A row
    /// floating inside a popover meets nothing, so the two square corners
    /// that rule gave it had no edge to sit against.
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
                // A row keeps its height when the menu is taller than the
                // space under it: the popover scrolls, it does not squash
                // its rows to fit — the same rule a tree row answers to.
                .keeps_size()
                .align(BAlign::Center)
                .gap(BSpace::GapSm)
                .pad_x(BSpace::PadSm)
                .ink()
                .type_size(BType::Body),
            // The glyph sits on a faint tile of its own, as a button's does
            // — the label beside it does not, so the row reads as an icon
            // and its name rather than as one long block.
            Self::IconSlot => {
                let slot = BLen::Measured(ctx.style.dimensions.icon_size_sm);
                BFlex::row()
                    .width(slot)
                    .height(slot)
                    .keeps_size()
                    .align(BAlign::Center)
                    .justify(BJustify::Center)
                    .fill(BFill::Tone(BSurfaceTone::Card))
                    .radius(BRadius::Role(BStyleRole::Button))
            }
            Self::Label => BFlex::row().fills().min_width(BLen::Zero),
            Self::Shortcut => BFlex::row().dim_ink().type_size(BType::Caption),
            Self::Submenu => {
                let slot = BLen::Measured(ctx.style.dimensions.icon_size_sm);
                BFlex::row().width(slot).height(slot).keeps_size().align(BAlign::Center).justify(BJustify::Center)
            }
            // And the cap keeps its size too, not only the row inside it.
            //
            // `Row` has said `keeps_size` all along, but the cap wrapping it
            // did not — and the cap is the flex child the popover's column
            // actually measures. So a column taller than the space under it
            // shrank the caps, each clipping the row it holds: a File menu's
            // rows painted at 17px, two thirds of a row, while the row itself
            // still believed it was a row. That is what made a menu look short
            // next to everything around it, and it got shorter the longer the
            // menu was. The popover scrolls instead (`Panel`'s `max_height`
            // plus the content's overflow), which is what it was built to do.
            Self::Cap(radius) => BFlex::row()
                .width(BLen::Full)
                .keeps_size()
                .radius(BRadius::All(*radius))
                .overflow(BOverflow::Clip),
            // A gap, not a rule: the space between groups is the grouping.
            // The hairline this used to paint was the dark line running
            // through every menu.
            Self::Separator => BFlex::row()
                .width(BLen::Full)
                .height(BLen::Measured(ctx.style.strokes.hairline))
                .margin_y(BSpace::GapXs),
        }
    }
}
