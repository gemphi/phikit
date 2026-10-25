//! One treatment for every control that sits in a header strip.
//!
//! A header is a row of menus, so everything in it reads as a menu: the same
//! resting fill, the same hover lift, the same filled active cap. Without
//! this, an icon button rested on nothing while the menu beside it rested on
//! the strip fill, which is what made headers look like several unrelated
//! widget sets sharing a row.

use super::color::{resolved_gradient, to_hsla};
use super::ctx::BRenderCtx;
use super::flex::{BEdge, BFlex};
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
/// Whether a control paints a ground at REST, rather than only on hover.
///
/// This used to be the activity rail alone, and the effect was that almost
/// every icon in the app floated on nothing: a rail of bare glyphs, a
/// render window whose side icons were a column of marks on a flat field.
/// An icon with no ground has no hit target the eye can see, so the user
/// has to hover to find out where the button is.
///
/// Now a ground is also painted wherever the caller asked for a tile
/// (`ctx.floating`, which `button.rs` sets for every icon-only control).
/// The shade itself is still the theme's: `bg_control`, which carries its
/// own alpha, so the tile sits ON the panel rather than replacing it.
pub fn rests(ctx: &BRenderCtx) -> bool {
    ctx.host == BHostType::Activity || ctx.floating
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
    // An entry of a source list (`BEdge::Listed`): a border on hover and
    // when chosen, a faint tint of the accent under the chosen one.
    if ctx.listed && !in_strip(ctx) {
        let entry = BFlex::cell().border(BEdge::Listed { chosen: active, muted });
        return entry.hovered_on(entry.styled(element, ctx), ctx);
    }
    let fill = |state| ctx.style.control_fill(state, ctx.floating).map(to_hsla).map(faded);
    if active {
        return element.bg(faded(active_fill(ctx)));
    }
    let hover = fill(BControlState::Hover).unwrap_or_else(|| faded(to_hsla(ctx.style.colors.bg_hover)));
    match fill(BControlState::Rest).filter(|_| rests(ctx)) {
        Some(rest) => element.bg(rest).hover(move |s| s.bg(hover)),
        None => element.hover(move |s| s.bg(hover)),
    }
}

/// The fill an active control takes outside a strip ([`paint_state`]).
pub fn active_fill(ctx: &BRenderCtx) -> gpui::Hsla {
    ctx.style.control_fill(BControlState::Active, ctx.floating).map(to_hsla).unwrap_or_else(|| to_hsla(ctx.style.colors.accent))
}

/// Ink that reads on `fill`: the theme's `text`, or white where both are
/// dark — a light theme's text on its accent — as Blender draws a selected
/// widget's text in its own `text_sel` rather than the resting text.
pub fn ink_on(fill: gpui::Hsla, text: gpui::Hsla) -> gpui::Hsla {
    match fill.l < 0.5 && text.l < 0.5 {
        true => gpui::Hsla { a: text.a, ..gpui::white() },
        false => text,
    }
}
