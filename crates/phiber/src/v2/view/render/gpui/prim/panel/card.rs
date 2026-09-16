//! Focused renderer for `BPanelRole::Card` — grouped accordion section.

use super::super::super::{color::to_hsla, ctx::BRenderCtx, walk};
use super::style::panel_bg;
use crate::v2::theme::BStyleRole;
use crate::v2::view::BView;
use crate::v2::view::prim::BPanelRole;
use gpui::prelude::*;
use gpui::{AnyElement, div, px};

pub struct BCard<'a>(pub &'a [BView]);

impl BCard<'_> {
    pub fn render(&self, c: &BRenderCtx) -> AnyElement {
        // Properties cards share the pane surface. Heading and rows own gutters.
        let mut element = div()
            .flex()
            .flex_col()
            .w_full()
            .pb(px(c.style.spacing.pad_xs))
            .gap(px(c.style.spacing.gap_xs))
            .rounded(px(c.style.radius_for(BStyleRole::Card)))
            .border(px(c.style.strokes.border))
            .border_color(to_hsla(c.style.colors.border))
            .bg(panel_bg(BPanelRole::Card, c));
        for child in self.0 {
            element = element.child(walk(child, c));
        }
        element.into_any_element()
    }
}

use super::super::render_contract::BFocusedRenderer;

impl BFocusedRenderer for BCard<'_> {
    fn render(&self, c: &BRenderCtx) -> AnyElement {
        BCard::render(self, c)
    }
}
