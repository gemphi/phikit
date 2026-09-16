//! Focused renderer for `BPanelRole::Footer`.

use super::super::super::{ctx::BRenderCtx, walk};
use super::style::{panel_bg, surface_gutter};
use crate::v2::view::BView;
use crate::v2::view::prim::BPanelRole;
use gpui::prelude::*;
use gpui::{AnyElement, div, px};

pub struct BFooter<'a>(pub &'a [BView]);

impl BFooter<'_> {
    pub fn render(&self, c: &BRenderCtx) -> AnyElement {
        let mut element = div()
            .flex()
            .flex_row()
            .items_center()
            .w_full()
            .h(px(c.style.dimensions.footer_height))
            .px(px(surface_gutter(c)))
            .gap(px(c.style.spacing.gap_xs))
            .bg(panel_bg(BPanelRole::Footer, c));
        for child in self.0 {
            element = element.child(walk(child, c));
        }
        element.into_any_element()
    }
}

use super::super::render_contract::BFocusedRenderer;

impl BFocusedRenderer for BFooter<'_> {
    fn render(&self, c: &BRenderCtx) -> AnyElement {
        BFooter::render(self, c)
    }
}
