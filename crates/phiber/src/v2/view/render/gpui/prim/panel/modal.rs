//! Focused renderer for `BPanelRole::Modal`.

use super::super::super::{ctx::BRenderCtx, walk};
use super::style::panel_bg;
use crate::v2::view::BView;
use crate::v2::view::prim::BPanelRole;
use gpui::prelude::*;
use gpui::{AnyElement, div, px};

pub struct BModal<'a>(pub &'a [BView]);

impl BModal<'_> {
    pub fn render(&self, c: &BRenderCtx) -> AnyElement {
        let mut element = div()
            .flex()
            .flex_col()
            .p(px(c.style.spacing.pad_sm))
            .bg(panel_bg(BPanelRole::Modal, c))
            .gap(px(c.style.spacing.gap_xs));
        for child in self.0 {
            element = element.child(walk(child, c));
        }
        element.into_any_element()
    }
}

use super::super::render_contract::BFocusedRenderer;

impl BFocusedRenderer for BModal<'_> {
    fn render(&self, c: &BRenderCtx) -> AnyElement {
        BModal::render(self, c)
    }
}
