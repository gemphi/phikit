//! Card header component — title, icon, badge, tools, and collapse button.

use super::super::bview::{ActionHandler, BAction, BView};
use super::super::icon::{IconSize, IconView};
use super::super::theme::BPalette;
use gpui::prelude::*;
use gpui::{AnyElement, IntoElement, SharedString, StatefulInteractiveElement, div, px};

pub struct CardHeaderView {
    pub title: Option<String>,
    pub icon: Option<String>,
    pub badge: Option<String>,
    pub tool: Option<Box<BView>>,
    pub collapsible: bool,
    pub collapsed: bool,
    pub collapse_action: Option<BAction>,
}

impl CardHeaderView {
    pub fn into_elm(self, p: &BPalette, on_action: ActionHandler) -> AnyElement {
        let header_row = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .h(px(p.h_tab))
            .px(px(p.pad_sm))
            .bg(p.fill_sheen(p.bg_header))
            .border_b_1()
            .border_color(p.border);

        let mut left = div().flex().flex_row().items_center().gap(px(p.gap_sm));

        if let Some(icon_name) = &self.icon {
            left = left.child(IconView::render(icon_name, IconSize::Sm, p.text));
        }

        if let Some(title_text) = &self.title {
            left = left.child(
                div()
                    .text_sm()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(p.text)
                    .child(SharedString::from(title_text.clone())),
            );
        }

        if let Some(badge_text) = &self.badge {
            left = left.child(
                div()
                    .px(px(p.pad_xs))
                    .py(px(1.0))
                    .rounded(px(p.radius_xs))
                    .bg(p.bg_active)
                    .text_xs()
                    .text_color(p.accent)
                    .child(SharedString::from(badge_text.clone())),
            );
        }

        let mut right = div().flex().flex_row().items_center().gap(px(p.gap_xs));

        if let Some(tool_view) = self.tool {
            right = right.child(tool_view.into_elm(p, on_action.clone()));
        }

        if self.collapsible {
            let chevron_icon = if self.collapsed {
                "chevron_right"
            } else {
                "chevron_down"
            };
            let collapse_btn = div()
                .id("card-collapse-btn")
                .p(px(2.0))
                .rounded(px(p.radius_xs))
                .cursor_pointer()
                .hover(|s| s.bg(p.bg_hover))
                .child(IconView::render(chevron_icon, IconSize::Xs, p.text_dim));

            if let Some(action) = self.collapse_action {
                let handler = on_action;
                right = right.child(collapse_btn.on_click(move |_, window, cx| {
                    handler(&action, window, cx);
                }));
            } else {
                right = right.child(collapse_btn);
            }
        }

        header_row.child(left).child(right).into_any_element()
    }
}
