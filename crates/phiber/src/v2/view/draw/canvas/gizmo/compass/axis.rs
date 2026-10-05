//! One projected compass axis: spoke plus positive and negative caps.

use crate::v2::state::{BKey as _, View3dKey};
use super::super::super::super::color::to_hsla;
use super::super::super::super::ctx::BRenderCtx;
use crate::v2::view::prim::BAction;
use gpui::prelude::*;
use gpui::{ClickEvent, Hsla, SharedString, div, hsla, px};
use std::rc::Rc;

pub(super) struct BAxis {
    pub letter: &'static str,
    pub color: Hsla,
    pub cx: f32,
    pub cy: f32,
    pub depth: f32,
    pub positive: bool,
    pub cap: f32,
}

impl BAxis {
    /// The viewpoint this cap looks from. Z needs a pitch, not a yaw, which
    /// is why a cap names a viewpoint and lets `state::follow` turn it into
    /// both angles — writing a yaw alone left the Z caps doing nothing
    /// meaningful and the X caps ignoring pitch.
    fn viewpoint(&self) -> &'static str {
        match (self.letter, self.positive) {
            ("X", true) => "right",
            ("X", false) => "left",
            ("Y", true) => "front",
            ("Y", false) => "back",
            (_, true) => "top",
            (_, false) => "bottom",
        }
    }

    pub fn spoke(
        color: Hsla,
        dx: f32,
        dy: f32,
        depth: f32,
        box_size: f32,
        center: f32,
        ctx: &BRenderCtx,
    ) -> gpui::Div {
        let depth_mix = (depth + 1.0) / 2.0;
        let alpha = ctx.style.effects.axis_alpha * (0.7 + 0.3 * depth_mix);
        let line = hsla(color.h, color.s, color.l, alpha);
        let mut spoke = div().absolute().inset_0().w(px(box_size)).h(px(box_size));
        // Beads are vector approximation mechanics because GPUI rotates only SVGs.
        const BEADS: usize = 5;
        let bead = ctx.style.strokes.grid * 2.6;
        for index in 1..=BEADS {
            let t = index as f32 / (BEADS as f32 + 1.0);
            let x = center + dx * t;
            let y = center + dy * t;
            spoke = spoke.child(
                div()
                    .absolute()
                    .top(px(y - bead / 2.0))
                    .left(px(x - bead / 2.0))
                    .w(px(bead))
                    .h(px(bead))
                    .rounded_full()
                    .bg(line),
            );
        }
        spoke
    }

    fn positive(&self, ctx: &BRenderCtx) -> gpui::Stateful<gpui::Div> {
        let depth_mix = (self.depth + 1.0) / 2.0;
        let diameter = self.cap * (0.86 + 0.14 * depth_mix);
        let alpha = ctx.style.effects.axis_alpha + (1.0 - ctx.style.effects.axis_alpha) * depth_mix;
        let fill = hsla(self.color.h, self.color.s, self.color.l, alpha);
        let rim = hsla(
            self.color.h,
            self.color.s,
            (self.color.l + ctx.style.effects.node_grid_alpha).min(1.0),
            alpha,
        );
        let hover = hsla(
            self.color.h,
            self.color.s,
            (self.color.l + ctx.style.effects.selected_hover_lift).min(1.0),
            1.0,
        );
        let handler = ctx.handler.clone();
        let action = BAction::Set(View3dKey::View.path(), self.viewpoint().to_string());
        let letter = self.letter;
        let selector = format!("compass-{letter}-pos");
        div()
            .id(SharedString::from(selector.clone()))
            .debug_selector(move || selector)
            .absolute()
            .top(px(self.cy - diameter / 2.0))
            .left(px(self.cx - diameter / 2.0))
            .w(px(diameter))
            .h(px(diameter))
            .rounded_full()
            .bg(fill)
            .border(px(ctx.style.strokes.border))
            .border_color(rim)
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .text_color(to_hsla(ctx.style.colors.text))
            .text_size(px(ctx.style.typography.caption))
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .child(SharedString::from(letter))
            .hover(move |style| style.bg(hover))
            .on_click(move |_e: &ClickEvent, window, app| {
                (Rc::clone(&handler))(&action, window, app);
            })
    }

    fn negative(&self, ctx: &BRenderCtx) -> gpui::Stateful<gpui::Div> {
        let depth_mix = (self.depth + 1.0) / 2.0;
        let diameter = self.cap * (0.68 + 0.14 * depth_mix);
        let ring = hsla(
            self.color.h,
            self.color.s,
            self.color.l,
            ctx.style.effects.axis_alpha,
        );
        let fill_alpha = ctx.style.effects.viewport_grid_alpha * 2.0;
        let fill = hsla(self.color.h, self.color.s, self.color.l, fill_alpha);
        let hover = hsla(
            self.color.h,
            self.color.s,
            self.color.l,
            ctx.style.effects.axis_alpha,
        );
        let handler = ctx.handler.clone();
        let action = BAction::Set(View3dKey::View.path(), self.viewpoint().to_string());
        let letter = self.letter;
        div()
            .id(SharedString::from(format!("compass-{letter}-neg")))
            .absolute()
            .top(px(self.cy - diameter / 2.0))
            .left(px(self.cx - diameter / 2.0))
            .w(px(diameter))
            .h(px(diameter))
            .rounded_full()
            .border(px(ctx.style.strokes.border * 2.0))
            .border_color(ring)
            .bg(fill)
            .cursor_pointer()
            .hover(move |style| style.bg(hover))
            .on_click(move |_e: &ClickEvent, window, app| {
                (Rc::clone(&handler))(&action, window, app);
            })
    }

    pub fn render(&self, ctx: &BRenderCtx) -> gpui::Stateful<gpui::Div> {
        if self.positive {
            self.positive(ctx)
        } else {
            self.negative(ctx)
        }
    }
}
