//! Focused renderer for `BPanelRole::Body` — generic scrolling body panel.
//!
//! Any leading `BView::TableHeader` children are rendered **outside** the
//! scroll container (sticky, inside the outer surface div) so the column
//! heading stays visible while file/content rows scroll beneath it.
//! All other children go into the bounded inner scroll div.
//!
//! This is the outer-surface / bounded-scroll separation required by task
//! 4.3: the outer div owns `bg + border + radius`; the inner div owns
//! `flex_1 min_h_0 min_w_0 overflow_y_scroll`.

use super::super::super::{color::to_hsla, ctx::BRenderCtx, walk};
use super::scroll::scroll_id;
use super::style::panel_bg;
use crate::v2::theme::BStyleRole;
use crate::v2::view::BView;
use crate::v2::view::prim::BPanelRole;
use gpui::prelude::*;
use gpui::{AnyElement, div, px};

pub struct BBody<'a>(pub &'a [BView]);

impl BBody<'_> {
    pub fn render(&self, c: &BRenderCtx) -> AnyElement {
        // Split: TableHeader rows are sticky (outside the scroll viewport);
        // everything else scrolls inside the bounded inner container.
        let sticky_end = self
            .0
            .iter()
            .take_while(|v| matches!(v, BView::TableHeader { .. }))
            .count();
        let sticky = &self.0[..sticky_end];
        let scrollable = &self.0[sticky_end..];

        // Outer surface owns paint, border, radius, and size constraints.
        // The bounded inner child owns only viewport motion and content spacing.
        let outer_padding = c.style.spacing.pad_sm;
        let outer = div()
            .flex()
            .flex_col()
            .flex_1()
            .w_full()
            .min_h_0()
            .min_w_0()
            .bg(panel_bg(BPanelRole::Body, c))
            .border(px(c.style.strokes.border))
            .border_color(to_hsla(c.style.colors.border))
            .rounded(px(c.style.radius_for(BStyleRole::Panel)));

        // Sticky headers sit inside the outer surface div but above the scroll
        // container, so the column geometry aligns with the scrolling rows.
        // Build the outer with sticky headers chained before the inner scroll div.
        let mut outer = sticky
            .iter()
            .fold(outer, |acc, header| acc.child(walk(header, c)));

        // Bounded scroll container: content spacing and overflow only.
        let mut inner = div()
            .id(scroll_id("panel-body", scrollable))
            .flex()
            .flex_col()
            .flex_1()
            .w_full()
            .min_h_0()
            .min_w_0()
            .p(px(outer_padding))
            .overflow_y_scroll()
            .gap(px(c.style.spacing.gap_xs));
        for child in scrollable {
            inner = inner.child(walk(child, c));
        }
        outer.child(inner).into_any_element()
    }
}

use super::super::render_contract::BFocusedRenderer;

impl BFocusedRenderer for BBody<'_> {
    fn render(&self, c: &BRenderCtx) -> AnyElement {
        BBody::render(self, c)
    }
}
