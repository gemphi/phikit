use super::super::{atom::icon_path, color::to_hsla, ctx::BRenderCtx};
use crate::v2::theme::BStyleRole;
use crate::v2::view::{BAction, BColAlign, BSortDirection};
use gpui::prelude::*;
use gpui::{AnyElement, ClickEvent, SharedString, div, px, svg};
use std::rc::Rc;

pub struct BSort<'a> {
    pub label: &'a str,
    pub direction: Option<BSortDirection>,
    pub align: BColAlign,
    pub action: &'a BAction,
}

impl BSort<'_> {
    pub fn render(&self, c: &BRenderCtx) -> AnyElement {
        let indicator_size = c.style.typography.body;
        let mut indicator = div()
            .flex()
            .flex_shrink_0()
            .items_center()
            .justify_center()
            .size(px(indicator_size));
        if let Some(direction) = self.direction {
            let icon = match direction {
                BSortDirection::Ascending => "chevron_up",
                BSortDirection::Descending => "chevron_down",
            };
            indicator = indicator.child(
                svg()
                    .path(icon_path(icon))
                    .size(px(indicator_size))
                    .text_color(to_hsla(c.style.colors.text_dim)),
            );
        }

        let mut content = div()
            .flex()
            .flex_row()
            .items_center()
            .w_full()
            .min_w_0()
            .gap(px(c.style.spacing.gap_xs));
        content = match self.align {
            BColAlign::Start => content.justify_start(),
            BColAlign::End => content.justify_end(),
        };
        content = content
            .child(SharedString::from(self.label.to_string()))
            .child(indicator);

        let handler = c.h.clone();
        let action = self.action.clone();
        let id_text = format!(
            "file-sort-{}",
            self.label.to_ascii_lowercase().replace(' ', "-")
        );
        let radius = px(c.style.radius_for(BStyleRole::MenuItem));
        div()
            .id(SharedString::from(id_text.clone()))
            .debug_selector(move || id_text.clone())
            .cursor_pointer()
            .w_full()
            .h_full()
            .rounded(radius)
            .child(content)
            .hover(move |style| style.bg(to_hsla(c.style.colors.bg_hover)).rounded(radius))
            .on_click(move |_event: &ClickEvent, window, app| {
                (Rc::clone(&handler))(&action, window, app);
            })
            .into_any_element()
    }
}

use super::render_contract::BFocusedRenderer;

impl BFocusedRenderer for BSort<'_> {
    fn render(&self, c: &BRenderCtx) -> AnyElement {
        BSort::render(self, c)
    }
}
