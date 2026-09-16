//! Focused renderer for `BPanelRole::Sidebar`.

use super::super::super::{color::to_hsla, ctx::BRenderCtx, walk};
use super::scroll::scroll_id;
use super::style::panel_bg;
use crate::v2::theme::BStyleRole;
use crate::v2::view::BView;
use crate::v2::view::prim::BPanelRole;
use gpui::prelude::*;
use gpui::{AnyElement, div, px};

pub struct BSidebar<'a>(pub &'a [BView]);

impl BSidebar<'_> {
    pub fn render(&self, c: &BRenderCtx) -> AnyElement {
        // Outer surface owns paint and geometry; the bounded inner child scrolls.
        let outer_padding = c.style.spacing.pad_sm;
        let outer = div()
            .flex()
            .flex_col()
            .h_full()
            .min_h_0()
            .flex_shrink_0()
            .min_w(px(c.style.sidebar_min_width()))
            .bg(panel_bg(BPanelRole::Sidebar, c))
            .border(px(c.style.strokes.border))
            .border_color(to_hsla(c.style.colors.border))
            .rounded(px(c.style.radius_for(BStyleRole::Panel)));
        let mut inner = div()
            .id(scroll_id("panel-sidebar", self.0))
            .flex()
            .flex_col()
            .flex_1()
            .w_full()
            .min_h_0()
            .min_w_0()
            .p(px(outer_padding))
            .gap(px(c.style.spacing.gap_xs))
            .overflow_y_scroll();
        for child in self.0 {
            inner = inner.child(walk(child, c));
        }
        outer.child(inner).into_any_element()
    }
}

use super::super::render_contract::BFocusedRenderer;

impl BFocusedRenderer for BSidebar<'_> {
    fn render(&self, c: &BRenderCtx) -> AnyElement {
        BSidebar::render(self, c)
    }
}
