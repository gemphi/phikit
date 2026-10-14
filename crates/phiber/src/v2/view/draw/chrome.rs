//! One treatment for every control that sits in a header strip.
//!
//! A header is a row of menus, so everything in it reads as a menu: the same
//! resting fill, the same hover lift, the same filled active cap. Without
//! this, an icon button rested on nothing while the menu beside it rested on
//! the strip fill, which is what made headers look like several unrelated
//! widget sets sharing a row.

use super::color::{resolved_gradient, to_hsla};
use super::ctx::BRenderCtx;
use crate::v2::host::BHostType;
use crate::v2::theme::BControlState;
use gpui::prelude::*;
use gpui::{Div, Stateful};

/// True inside a strip whose controls follow the menu treatment: the two
/// header rows and the path/tool bar beneath them.
pub fn in_strip(ctx: &BRenderCtx) -> bool {
    ctx.strip
}

/// Whether a control carries a **resting** fill — a visible ground when it
/// is neither hovered nor on.
///
/// Only the activity bar does. Its entries are the app's primary
/// navigation and read as a set of places, so each one keeps a ground to
/// sit on. Everywhere else — menu rows, sidebar entries, header controls,
/// the floating toolbar — a control is invisible until the pointer finds
/// it: a resting fill on each one turns a menu into a column of pills and
/// a sidebar into a stack of chips, which is what made the chrome look
/// like several unrelated widget sets.
///
/// Hover and active are unaffected: a control still lifts under the
/// pointer and still fills when it is on, in every surface.
pub fn rests(ctx: &BRenderCtx) -> bool {
    ctx.host == BHostType::Activity
}

/// Paint one control: the accent at the opacity for its state.
///
/// Rest, hover and active are one hue at three strengths
/// (`style.control_fill`), and [`rests`] decides whether the resting step
/// is painted at all — only the activity bar's is. A menu cap that is open
/// keeps its filled gradient, which is what marks it as the open one.
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
    match fill(BControlState::Rest).filter(|_| rests(ctx)) {
        Some(rest) => element.bg(rest).hover(move |s| s.bg(hover)),
        None => element.hover(move |s| s.bg(hover)),
    }
}
