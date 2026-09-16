//! Focused renderer for `BPanelRole::CardHeader`.

use super::super::super::{color::to_hsla, ctx::BRenderCtx, walk};
use super::style::{panel_bg, surface_gutter};
use crate::v2::theme::BStyleRole;
use crate::v2::view::BView;
use crate::v2::view::prim::BPanelRole;
use gpui::prelude::*;
use gpui::{AnyElement, div, px};

pub struct BCardHeader<'a>(pub &'a [BView]);

impl BCardHeader<'_> {
    pub fn render(&self, c: &BRenderCtx) -> AnyElement {
        let mut element = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .w_full()
            .h(px(c.style.dimensions.header_height))
            .px(px(surface_gutter(c)))
            .rounded_t(px(c.style.radius_for(BStyleRole::Card)))
            .bg(panel_bg(BPanelRole::CardHeader, c))
            .border_b(px(c.style.strokes.hairline))
            .border_color(to_hsla(c.style.colors.sep))
            .text_color(to_hsla(c.style.colors.text));
        for child in self.0 {
            element = element.child(walk(child, c));
        }
        element.into_any_element()
    }
}

use super::super::render_contract::BFocusedRenderer;

impl BFocusedRenderer for BCardHeader<'_> {
    fn render(&self, c: &BRenderCtx) -> AnyElement {
        BCardHeader::render(self, c)
    }
}
