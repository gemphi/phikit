//! The containers that keep state between frames — the one that unfolds
//! over the theme's motion duration, and the scrollers — built on
//! [`BFlex::div`] with the identity GPUI needs to hold that state by.

use super::element::BFlex;
use super::token::BLen;
use super::super::ctx::BRenderCtx;
use gpui::prelude::*;
use gpui::{Div, px};

impl BFlex {
    /// The container unfolding: drawn at no height the first frame, growing
    /// to its declared height over the theme's motion duration, once. The
    /// drawer the action log opens over the status bar. With reduced motion
    /// it is drawn at full height from the first frame.
    pub fn div_unfolding(
        &self,
        id: impl Into<gpui::ElementId>,
        ctx: &BRenderCtx,
    ) -> gpui::AnimationElement<Div> {
        use gpui::AnimationExt as _;
        let full = match self.height {
            Some(BLen::Dim(d)) => d.px(ctx),
            Some(BLen::Measured(v)) => v,
            _ => 0.0,
        };
        let motion = std::time::Duration::from_secs_f32(ctx.style.effects.motion_ms.max(0.0) / 1000.0);
        let animation = gpui::Animation::new(motion).with_easing(gpui::ease_out_quint());
        self.div(ctx).with_animation(id, animation, move |element, t| element.h(px(full * t)))
    }

    /// The container as a vertical scroller. Scrolling keeps state (the
    /// offset), so GPUI needs an id to hold it by — which is why this is the
    /// one constructor that takes one.
    pub fn scroller(&self, id: impl Into<gpui::ElementId>, ctx: &BRenderCtx) -> gpui::Stateful<Div> {
        self.div(ctx).id(id).overflow_y_scroll()
    }

    /// The container as a horizontal strip that scrolls when its content is
    /// wider than it: Blender's header regions, which the wheel pans
    /// sideways (`V2D_SCROLL_HORIZONTAL_HIDE`, no bar drawn). GPUI turns a
    /// vertical wheel into a sideways scroll for an x-only scroller.
    pub fn strip_scroller(&self, id: impl Into<gpui::ElementId>, ctx: &BRenderCtx) -> gpui::Stateful<Div> {
        self.div(ctx).id(id).overflow_x_scroll()
    }
}
