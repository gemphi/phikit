//! Card item component — interactive row with icon, label, badge, and action trigger.

use super::super::bview::{ActionHandler, BAction};
use super::super::icon::{IconSize, IconView};
use super::super::theme::BPalette;
use gpui::prelude::*;
use gpui::{AnyElement, IntoElement, SharedString, StatefulInteractiveElement, div, px};

pub struct CardItemView {
    pub label: String,
    pub icon: Option<String>,
    pub badge: Option<String>,
    pub active: bool,
    pub action: Option<BAction>,
}

impl CardItemView {
    pub fn into_elm(self, p: &BPalette, on_action: ActionHandler) -> AnyElement {
        let bg = if self.active {
            p.bg_active
        } else {
            gpui::transparent_black()
        };
        let mut left = div().flex().flex_row().items_center().gap(px(p.gap_sm));

        if let Some(icon_name) = &self.icon {
            let color = if self.active { p.accent } else { p.text };
            left = left.child(IconView::render(icon_name, IconSize::Sm, color));
        }

        left = left.child(
            div()
                .text_sm()
                .text_color(if self.active { p.text } else { p.text_dim })
                .child(SharedString::from(self.label.clone())),
        );

        let badge_elm = self.badge.map(|b| {
            div()
                .px(px(p.pad_xs))
                .rounded(px(p.radius_xs))
                .bg(p.bg_icon)
                .text_xs()
                .text_color(p.text_dim)
                .child(SharedString::from(b))
        });

        let id = format!("card-item-{}", self.label.replace(' ', "-"));
        let item = div()
            .id(SharedString::from(id))
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .h(px(p.h_tab))
            .px(px(p.pad_sm))
            .rounded(px(p.radius_sm))
            .bg(bg)
            .cursor_pointer()
            .hover(|s| s.bg(p.bg_hover))
            .child(left)
            .children(badge_elm);

        if let Some(act) = self.action {
            let handler = on_action;
            item.on_click(move |_, window, cx| {
                handler(&act, window, cx);
            })
            .into_any_element()
        } else {
            item.into_any_element()
        }
    }
}
