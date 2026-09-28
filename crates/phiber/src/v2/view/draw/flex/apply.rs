//! Turning a `BFlex` into a GPUI element — the only place in the renderer that
//! calls GPUI's style builders. Everything above this reads as decisions;
//! this is the mechanics.

use super::element::BFlex;
use super::paint::shadowed;
use super::token::{BAlign, BBasis, BEdge, BJustify, BLen, BRadius, BSelf};
use super::super::color::to_hsla;
use super::super::ctx::BRenderCtx;

use crate::v2::view::layout::BAxis;
use gpui::prelude::*;
use gpui::{Div, div, px};

/// Fully transparent and fully opaque: the two states of a part that is
/// revealed by hover. Not design values, so not theme tokens.
const HIDDEN: f32 = 0.0;
const SHOWN: f32 = 1.0;

impl BFlex {
    /// The container, styled. The only place in the renderer that turns a
    /// layout decision into GPUI style.
    pub fn div(&self, ctx: &BRenderCtx) -> Div {
        let mut element = self.styled(div(), ctx);
        // A hover is a fill like any other, applied here rather than written
        // out as `.hover(|s| s.bg(…))` wherever a hoverable thing is built.
        // `Styled` cannot express it, so it lands on the concrete `Div`.
        if let Some(hover) = self.hover_fill {
            let fill = hover.background(ctx);
            element = element.hover(move |style| style.bg(fill));
        }
        if self.occludes {
            element = element.occlude();
        }
        if let Some(group) = self.group {
            element = element.group(group);
        }
        if let Some(group) = self.revealed_by {
            element = element.opacity(HIDDEN).group_hover(group, |style| style.opacity(SHOWN));
        }
        element
    }

    /// The same decisions applied to an element that is not a `div` — a
    /// canvas that has to cover its parent, say. Occlusion is a `div`'s
    /// alone, so it stays in [`Self::div`].
    pub fn styled<E: Styled>(&self, element: E, ctx: &BRenderCtx) -> E {
        let len = |l: BLen| -> gpui::Length {
            match l {
                BLen::Full => gpui::relative(1.).into(),
                BLen::Zero => gpui::Pixels::ZERO.into(),
                BLen::Dim(d) => px(d.px(ctx)).into(),
                BLen::Share(f) => gpui::relative(f).into(),
                BLen::Measured(v) => px(v).into(),
            }
        };
        let mut element = element;
        if let Some(axis) = self.axis {
            element = element.flex();
            element = match axis {
                BAxis::Row => element.flex_row(),
                BAxis::Column => element.flex_col(),
            };
        }
        element = if self.wrap { element.flex_wrap() } else { element };
        element = match self.self_align {
            Some(BSelf::Center) => element.self_center(),
            Some(BSelf::End) => element.self_end(),
            Some(BSelf::Stretch) => element.self_stretch(),
            None => element,
        };
        {
            let style = element.style();
            if let Some(g) = self.grow {
                style.flex_grow = Some(g);
            }
            if let Some(s) = self.shrink {
                style.flex_shrink = Some(s);
            }
            if let Some(b) = self.basis {
                style.flex_basis = Some(match b {
                    BBasis::NoShare => gpui::relative(0.).into(),
                    BBasis::NoPixels => gpui::Pixels::ZERO.into(),
                    BBasis::Content => gpui::Length::Auto,
                });
            }
            if let Some(w) = self.width {
                style.size.width = Some(len(w));
            }
            if let Some(h) = self.height {
                style.size.height = Some(len(h));
            }
            if let Some(w) = self.min_width {
                style.min_size.width = Some(len(w));
            }
            if let Some(h) = self.min_height {
                style.min_size.height = Some(len(h));
            }
            if let Some(w) = self.max_width {
                style.max_size.width = Some(len(w));
            }
        }
        element = match self.align {
            Some(BAlign::Start) => element.items_start(),
            Some(BAlign::Center) => element.items_center(),
            Some(BAlign::End) => element.items_end(),
            Some(BAlign::Stretch) => element.items_stretch(),
            None => element,
        };
        element = match self.justify {
            Some(BJustify::Start) => element.justify_start(),
            Some(BJustify::Center) => element.justify_center(),
            Some(BJustify::End) => element.justify_end(),
            Some(BJustify::Between) => element.justify_between(),
            None => element,
        };
        if let Some(g) = self.gap {
            element = element.gap(px(g.px(ctx)));
        }
        if let Some(p) = self.pad_top {
            element = element.pt(px(p.px(ctx)));
        }
        if let Some(p) = self.pad_right {
            element = element.pr(px(p.px(ctx)));
        }
        if let Some(p) = self.pad_bottom {
            element = element.pb(px(p.px(ctx)));
        }
        if let Some(p) = self.pad_left {
            element = element.pl(px(p.px(ctx)));
        }
        if let Some(m) = self.margin_left {
            element = element.ml(px(m.px(ctx)));
        }
        if let Some(m) = self.margin_right {
            element = element.mr(px(m.px(ctx)));
        }
        if let Some(m) = self.margin_top {
            element = element.mt(px(m.px(ctx)));
        }
        if let Some(m) = self.margin_bottom {
            element = element.mb(px(m.px(ctx)));
        }
        if let Some(r) = self.raised_by {
            element = element.mt(px(-r.px(ctx)));
        }
        if let Some(fill) = self.fill {
            element = element.bg(fill.background(ctx));
        }
        match self.border {
            Some(BEdge::All) => {
                element = element
                    .border(px(ctx.style.strokes.border))
                    .border_color(to_hsla(ctx.style.colors.border))
            }
            Some(BEdge::Bottom) => {
                element = element
                    .border_b(px(ctx.style.strokes.border))
                    .border_color(to_hsla(ctx.style.colors.border))
            }
            Some(BEdge::Guide) => {
                let guide = ctx.style.separator_stroke();
                element = element.border_l(px(guide.width)).border_color(to_hsla(guide.color))
            }
            Some(BEdge::Ink(ink)) => {
                element = element.border(px(ctx.style.strokes.hairline)).border_color(ink.color(ctx))
            }
            None => {}
        }
        match self.radius {
            Some(BRadius::Role(role)) => element = element.rounded(px(ctx.style.radius_for(role))),
            Some(BRadius::All(r)) => element = element.rounded(px(r)),
            Some(BRadius::Top(r)) => element = element.rounded_t(px(r)),
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
        element = shadowed(element, ctx.style.shadow_for(self.elevation));
        if let Some(ink) = self.ink {
            element = element.text_color(ink.color(ctx));
        }
        element = self.placed(element, ctx);
        element
    }
}
