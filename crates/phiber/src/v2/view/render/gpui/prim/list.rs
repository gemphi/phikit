//! Focused renderer for `BView::ListRow`.

use super::super::{color::to_hsla, ctx::BRenderCtx, walk};
use crate::v2::theme::BStyleRole;
use crate::v2::view::BView;
use crate::v2::view::prim::BRowShade;
use gpui::prelude::*;
use gpui::{AnyElement, div, px};

pub struct BList<'a> {
    pub children: &'a [BView],
    pub shade: BRowShade,
}

impl BList<'_> {
    pub fn render(&self, c: &BRenderCtx<'_>) -> AnyElement {
        let selected = self.shade == BRowShade::Selected;
        let background = match self.shade {
            BRowShade::Flat => c.style.colors.bg_panel,
            BRowShade::Alt => c.style.colors.row_alt,
            BRowShade::Selected => c.style.colors.row_selected,
        };
        let hover = if selected {
            c.style.selected_hover(background)
        } else {
            c.style.colors.bg_hover
        };
        let height = if selected {
            c.style.dimensions.row_height - 2.0 * c.style.strokes.border
        } else {
            c.style.dimensions.row_height
        };
        let mut element = div()
            .flex()
            .flex_row()
            .items_center()
            .w_full()
            .h(px(height))
            .px(px(c.style.spacing.pad_md))
            .gap(px(c.style.spacing.gap_xs))
            .bg(to_hsla(background))
            .hover(move |style| style.bg(to_hsla(hover)))
            .text_color(to_hsla(c.style.colors.text))
            .text_size(px(c.style.typography.body));
        if selected {
            element = element
                .rounded(px(c.style.radius_for(BStyleRole::MenuItem)))
                .my(px(c.style.strokes.border))
                .mx(px(c.style.strokes.border));
        } else {
            element = element.rounded(px(c.style.radius_for(BStyleRole::MenuItem)));
        }
        for child in self.children {
            element = element.child(walk(child, c));
        }
        element.into_any_element()
    }
}

use super::render_contract::BFocusedRenderer;

impl BFocusedRenderer for BList<'_> {
    fn render(&self, c: &BRenderCtx) -> AnyElement {
        BList::render(self, c)
    }
}
