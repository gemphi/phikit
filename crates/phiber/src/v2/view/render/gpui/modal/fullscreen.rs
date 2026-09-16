//! Viewport-filling Splash/Welcome and About modal form.

use super::super::{color::resolved_gradient, ctx::BRenderCtx, walk};
use super::{title::BTitle, welcome::BWelcome};
use crate::v2::modal::BModalType;
use crate::v2::theme::BSurfaceTone;
use crate::v2::view::BView;
use gpui::prelude::*;
use gpui::{AnyElement, div, px};

pub struct BFullscreen<'a> {
    pub kind: BModalType,
    pub body: &'a BView,
}

impl BFullscreen<'_> {
    pub fn render(&self, c: &BRenderCtx) -> AnyElement {
        let maximized = super::is_maximized(c);
        div()
            .absolute()
            .inset_0()
            .size_full()
            .flex()
            .flex_col()
            .bg(resolved_gradient(c.style.surface(BSurfaceTone::Modal)))
            .text_color(super::super::color::to_hsla(c.style.colors.text))
            .occlude()
            .child(
                BTitle {
                    kind: self.kind,
                    maximized,
                    rounded: false,
                }
                .render(c),
            )
            .child(if self.kind == BModalType::Splash {
                BWelcome(self.body).render(c)
            } else {
                div()
                    .flex_1()
                    .size_full()
                    .min_h_0()
                    .overflow_hidden()
                    .p(px(c.style.spacing.pad_lg))
                    .child(walk(self.body, c))
                    .into_any_element()
            })
            .into_any_element()
    }
}

use super::super::prim::render_contract::BFocusedRenderer;

impl BFocusedRenderer for BFullscreen<'_> {
    fn render(&self, c: &super::super::ctx::BRenderCtx) -> gpui::AnyElement {
        BFullscreen::render(self, c)
    }
}
