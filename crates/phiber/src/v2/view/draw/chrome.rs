//! One treatment for every control that sits in a header strip.
//!
//! A header is a row of menus, so everything in it reads as a menu: the same
//! resting fill, the same hover lift, the same filled active cap. Without
//! this, an icon button rested on nothing while the menu beside it rested on
//! the strip fill, which is what made headers look like several unrelated
//! widget sets sharing a row.

use super::color::{resolved_gradient, to_hsla};
use super::ctx::BRenderCtx;
use crate::v2::theme::BControlState;
use gpui::prelude::*;
use gpui::{Div, Stateful};

/// True inside a strip whose controls follow the menu treatment: the two
/// header rows and the path/tool bar beneath them.
pub fn in_strip(ctx: &BRenderCtx) -> bool {
    ctx.strip
}

/// Paint one control: the accent at the opacity for its state.
///
/// Rest, hover and active are one hue at three strengths
/// (`style.control_fill`). Docked chrome paints nothing at rest — a resting
/// fill turns a header into a row of pills — while floating chrome rests on
/// the faintest step so it stays legible over the canvas. A menu cap that is
/// open keeps its filled gradient, which is what marks it as the open one.
pub fn paint(element: Stateful<Div>, active: bool, ctx: &BRenderCtx) -> Stateful<Div> {
    paint_state(element, active, false, ctx)
}

/// [`paint`], and `muted` as a weight on it (`effects.control_muted_alpha`):
/// a control on but not in force keeps its "on" fill, faded — so the two
/// states stack instead of the fade dropping the fill. An open menu cap in
/// a strip, muted, takes the solid active fill faded rather than the
/// gradient, which carries no weight.
pub fn paint_state(element: Stateful<Div>, active: bool, muted: bool, ctx: &BRenderCtx) -> Stateful<Div> {
    let weight = if muted { ctx.style.effects.control_muted_alpha } else { 1.0 };
    let faded = move |c: gpui::Hsla| gpui::Hsla { a: c.a * weight, ..c };
    if active && in_strip(ctx) && !muted {
        return element.bg(resolved_gradient(ctx.style.host_item_active_gradient(ctx.host)));
    }
    let fill = |state| ctx.style.control_fill(state, ctx.floating).map(to_hsla).map(faded);
    if active {
        let on = fill(BControlState::Active).unwrap_or_else(|| faded(to_hsla(ctx.style.colors.accent)));
        return element.bg(on);
    }
    let hover = fill(BControlState::Hover).unwrap_or_else(|| faded(to_hsla(ctx.style.colors.bg_hover)));
    match fill(BControlState::Rest) {
        Some(rest) => element.bg(rest).hover(move |s| s.bg(hover)),
        None => element.hover(move |s| s.bg(hover)),
    }
}
