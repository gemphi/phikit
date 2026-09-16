//! Compact Image/UV editor canvas for the Shading left column.

use super::super::super::{color::to_hsla, ctx::BRenderCtx};
use gpui::prelude::*;
use gpui::{AnyElement, SharedString, div, px};

pub struct BImage(pub bool);

impl BImage {
    pub fn render(&self, c: &BRenderCtx) -> AnyElement {
        let bg = to_hsla(c.style.viewport_surface());
        let checker = c.style.image_checker_colors();
        let mut image = div()
            .absolute()
            .left(gpui::relative(0.16))
            .right(gpui::relative(0.16))
            .top(gpui::relative(0.18))
            .bottom(gpui::relative(0.18))
            .border(px(c.style.strokes.border))
            .border_color(to_hsla(c.style.colors.border));
        for row in 0..4 {
            for col in 0..4 {
                image = image.child(
                    div()
                        .absolute()
                        .left(gpui::relative(col as f32 / 4.0))
                        .top(gpui::relative(row as f32 / 4.0))
                        .w(gpui::relative(0.25))
                        .h(gpui::relative(0.25))
                        .bg(to_hsla(if (row + col) % 2 == 0 {
                            checker.light
                        } else {
                            checker.dark
                        })),
                );
            }
        }
        div()
            .flex_1()
            .min_h_0()
            .size_full()
            .relative()
            .overflow_hidden()
            .bg(bg)
            .child(image)
            .child(
                div()
                    .absolute()
                    .left(px(c.style.spacing.pad_md))
                    .bottom(px(c.style.spacing.pad_md))
                    .text_color(to_hsla(c.style.colors.text_dim))
                    .text_size(px(c.style.typography.caption))
                    .child(SharedString::from(if self.0 {
                        "UV Editor"
                    } else {
                        "Image Editor"
                    })),
            )
            .into_any_element()
    }
}

use crate::v2::view::render::gpui::prim::render_contract::BFocusedRenderer;

impl BFocusedRenderer for BImage {
    fn render(&self, c: &crate::v2::view::render::gpui::ctx::BRenderCtx) -> gpui::AnyElement {
        BImage::render(self, c)
    }
}
