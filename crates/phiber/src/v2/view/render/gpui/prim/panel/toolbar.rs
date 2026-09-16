//! Focused renderer for `BPanelRole::Toolbar`.

use super::super::super::{ctx::BRenderCtx, walk};
use super::style::{panel_bg, surface_gutter};
use crate::v2::view::BView;
use crate::v2::view::prim::BPanelRole;
use gpui::prelude::*;
use gpui::{AnyElement, div, px};

pub struct BToolbar<'a>(pub &'a [BView]);

impl BToolbar<'_> {
    pub fn render(&self, c: &BRenderCtx) -> AnyElement {
        let mut element = div()
            .flex()
            .flex_row()
            .items_center()
            .w_full()
            .h(px(c.style.dimensions.header_height))
            .px(px(surface_gutter(c)))
            .gap(px(c.style.spacing.gap_xs))
            .bg(panel_bg(BPanelRole::Toolbar, c));
        for child in self.0 {
            element = element.child(walk(child, c));
        }
        element.into_any_element()
    }
}

use super::super::render_contract::BFocusedRenderer;

impl BFocusedRenderer for BToolbar<'_> {
    fn render(&self, c: &BRenderCtx) -> AnyElement {
        BToolbar::render(self, c)
    }
}
