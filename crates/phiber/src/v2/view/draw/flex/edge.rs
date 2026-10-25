//! A `BFlex`'s edge and corners as GPUI style: its border (`BEdge`) and its
//! radius (`BRadius`).
//!
//! Split out of `apply.rs` for the file cap, as `paint.rs` was. It is still
//! the element's own mechanics — the only other place a border or a corner
//! becomes GPUI style.

use super::apply::listed_weight;
use super::element::BFlex;
use super::token::{BEdge, BRadius};
use super::super::color::to_hsla;
use super::super::ctx::BRenderCtx;

use gpui::prelude::*;
use gpui::px;

/// How strong the accent tint under a chosen list entry is.
const LISTED_TINT: f32 = 0.12;

impl BFlex {
    /// `element` with this container's border and corners.
    pub(super) fn edged<E: Styled>(&self, element: E, ctx: &BRenderCtx) -> E {
        let mut element = element;
        match self.border {
            Some(BEdge::All) => {
                element = element
                    .border(px(ctx.style.strokes.border))
                    .border_color(to_hsla(ctx.style.colors.border))
            }
            // A heading's rule (`colors.rule`): every header, card title and
            // section closes on the same, visible line.
            Some(BEdge::Bottom) => {
                element = element
                    .border_b(px(ctx.style.strokes.border))
                    .border_color(to_hsla(ctx.style.colors.rule))
            }
            Some(BEdge::Guide) => {
                let guide = ctx.style.separator_stroke();
                element = element.border_l(px(guide.width)).border_color(to_hsla(guide.color))
            }
            Some(BEdge::Ink(ink)) => {
                element = element.border(px(ctx.style.strokes.hairline)).border_color(ink.color(ctx))
            }
            Some(BEdge::Ring(ink)) => {
                element = element.border(px(ctx.style.strokes.border * 2.0)).border_color(ink.color(ctx))
            }
            Some(BEdge::Listed { chosen, muted }) => {
                let weight = listed_weight(muted, ctx);
                let accent = to_hsla(ctx.style.colors.accent);
                element = element.border(px(ctx.style.strokes.border));
                element = if chosen {
                    element
                        .border_color(gpui::Hsla { a: accent.a * weight, ..accent })
                        .bg(gpui::Hsla { a: LISTED_TINT * weight, ..accent })
                } else {
                    element.border_color(gpui::transparent_black())
                };
            }
            Some(BEdge::Active) => element = element.border(px(ctx.style.strokes.border)).border_color(gpui::transparent_black()),
            None => {}
        }
        match self.radius {
            Some(BRadius::Role(role)) => element = element.rounded(px(ctx.style.radius_for(role))),
            Some(BRadius::All(r)) => element = element.rounded(px(r)),
            Some(BRadius::Top(r)) => element = element.rounded_t(px(r)),
            Some(BRadius::Pill) => element = element.rounded_full(),
            Some(BRadius::Corners { tl, tr, bl, br, radius }) => {
                let r = px(radius);
                if tl {
                    element = element.rounded_tl(r);
                }
                if tr {
                    element = element.rounded_tr(r);
                }
                if bl {
                    element = element.rounded_bl(r);
                }
                if br {
                    element = element.rounded_br(r);
                }
            }
            None => {}
        }
        element
    }
}
