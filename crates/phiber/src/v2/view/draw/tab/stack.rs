//! A stacked rail entry — the activity bar's primary, as Slack draws its
//! rail: the icon with the entry's name under it, one control the width
//! of the rail less its insets, rounded, painted like every docked control
//! (`chrome::paint`: nothing at rest, the hover shade under the pointer,
//! the accent when active). The name is the entry's own, so it needs no
//! tooltip; the ids (`tab-<icon>-<label>`, `tab-icon-…`) match the other
//! caps so the geometry tests find it where they found the free cap.

use super::super::atom::icon_path;
use super::super::chrome;
use super::super::color::to_hsla;
use super::super::ctx::BRenderCtx;
use super::super::flex::{BAlign, BFlex, BInk, BJustify, BLen, BRadius, BSpace, BType};
use crate::v2::theme::BStyleRole;
use crate::v2::view::prim::BControl;
use gpui::prelude::*;
use gpui::{AnyElement, ClickEvent, SharedString, px, svg};
use std::rc::Rc;

pub fn rail_stack(ctl: &BControl, ctx: &BRenderCtx) -> AnyElement {
    let ic = ctl.icon.as_deref().unwrap_or("");
    let label = ctl.label.as_deref().or(ctl.tooltip.as_deref()).unwrap_or(ic);
    let active = ctl.active;
    // The rail less its card border and one inset either side.
    let width = ctx.style.dimensions.activity_width - 2.0 * (ctx.style.strokes.border + ctx.style.spacing.pad_xs);
    let icon_ink = to_hsla(ctx.style.icon_ink(active));
    let id_text = format!("tab-{ic}-{}", label.to_lowercase());
    let icon_id_text = format!("tab-icon-{ic}-{}", label.to_lowercase());
    let icon = BFlex::cell()
        .keeps_size()
        .div(ctx)
        .id(SharedString::from(icon_id_text.clone()))
        .debug_selector(move || icon_id_text.clone())
        .child(svg().path(icon_path(ic)).size(px(ctx.style.dimensions.icon_size)).text_color(icon_ink));
    let stack = BFlex::column()
        .width(BLen::Measured(width))
        .keeps_size()
        .align(BAlign::Center)
        .justify(BJustify::Center)
        .gap(BSpace::GapXs)
        .pad_y(BSpace::PadSm)
        .radius(BRadius::Role(BStyleRole::Tab))
        .type_size(BType::Caption)
        .ink_tone(if active { BInk::Text } else { BInk::Dim })
        .div(ctx)
        .id(SharedString::from(id_text.clone()))
        .debug_selector({
            let id = id_text.clone();
            move || id.clone()
        })
        .cursor_pointer()
        .child(icon)
        .child(SharedString::from(label.to_string()));
    let stack = chrome::paint(stack, active, ctx);
    let Some(action) = ctl.action.clone() else { return stack.into_any_element() };
    let handler = ctx.handler.clone();
    stack
        .on_click(move |_: &ClickEvent, window: &mut gpui::Window, app: &mut gpui::App| {
            (Rc::clone(&handler))(&action, window, app);
        })
        .into_any_element()
}
