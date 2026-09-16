//! Focused renderer for `BView::Row` — flex-row with bounded height.

use super::super::{ctx::BRenderCtx, walk};
use crate::v2::view::BView;
use gpui::prelude::*;
use gpui::{AnyElement, div, px};

pub struct BRow<'a>(pub &'a [BView]);

impl BRow<'_> {
    pub fn render(&self, c: &BRenderCtx<'_>) -> AnyElement {
        // A row inside a header hugs its content (natural width, centred by
        // the enclosing header Center pane) so header controls don't balloon
        // into full-width bars. Everywhere else the row fills its flex parent
        // (w_full) and stretches Column / Panel children to the row height.
        let in_header = matches!(
            c.region,
            Some(crate::v2::region::BRegionType::Header)
                | Some(crate::v2::region::BRegionType::ToolHeader)
        );
        let mut element = div()
            .flex()
            .flex_row()
            .items_center()
            .min_h_0()
            .min_w_0()
            .gap(px(c.style.spacing.gap_xs));
        if !in_header {
            element = element.items_stretch().w_full().h_full();
        }
        for child in self.0 {
            element = element.child(walk(child, c));
        }
        element.into_any_element()
    }
}

use super::render_contract::BFocusedRenderer;

impl BFocusedRenderer for BRow<'_> {
    fn render(&self, c: &BRenderCtx) -> AnyElement {
        BRow::render(self, c)
    }
}
