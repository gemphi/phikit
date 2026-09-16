//! Focused renderer for `BPanelRole::Section` — property row.

use super::super::super::{ctx::BRenderCtx, walk};
use super::style::{panel_bg, text_gutter};
use crate::v2::view::BView;
use crate::v2::view::prim::BPanelRole;
use gpui::prelude::*;
use gpui::{AnyElement, div, px};

pub struct BSection<'a>(pub &'a [BView]);

impl BSection<'_> {
    pub fn render(&self, c: &BRenderCtx) -> AnyElement {
        let selector = match self.0.first() {
            Some(BView::Label { text }) => format!("section-{text}"),
            _ => "section".to_string(),
        };
        let mut element = div()
            .id(gpui::SharedString::from(selector.clone()))
            .debug_selector(move || selector.clone())
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .gap(px(c.style.spacing.gap_xs))
            .w_full()
            .h(px(c.style.dimensions.row_height))
            .px(px(text_gutter(c)));
        if c.host != crate::v2::host::BHostType::Buts {
            element = element.bg(panel_bg(BPanelRole::Section, c));
        }
        for child in self.0 {
            element = element.child(div().flex().flex_1().min_w_0().child(walk(child, c)));
        }
        element.into_any_element()
    }
}

use super::super::render_contract::BFocusedRenderer;

impl BFocusedRenderer for BSection<'_> {
    fn render(&self, c: &BRenderCtx) -> AnyElement {
        BSection::render(self, c)
    }
}
