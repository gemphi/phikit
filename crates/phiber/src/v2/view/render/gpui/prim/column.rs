//! Focused renderer for `BView::Column` — flex-col filling its slot.

use super::super::{ctx::BRenderCtx, walk};
use crate::v2::view::BView;
use gpui::prelude::*;
use gpui::{AnyElement, div, px};

pub struct BColumn<'a>(pub &'a [BView]);

impl BColumn<'_> {
    pub fn render(&self, c: &BRenderCtx<'_>) -> AnyElement {
        let mut element = div()
            .flex()
            .flex_col()
            .flex_1()
            .size_full()
            .min_h_0()
            .min_w_0()
            .gap(px(c.style.spacing.gap_xs));
        for child in self.0 {
            element = element.child(walk(child, c));
        }
        element.into_any_element()
    }
}

use super::render_contract::BFocusedRenderer;

impl BFocusedRenderer for BColumn<'_> {
    fn render(&self, c: &BRenderCtx) -> AnyElement {
        BColumn::render(self, c)
    }
}
