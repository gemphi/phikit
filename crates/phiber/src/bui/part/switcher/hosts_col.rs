//! Switcher left column — grouped editors picker component struct.

use super::super::super::bview::{ActionHandler, BAction};
use super::super::super::icon::{IconSize, IconView};
use super::super::super::state::BKey;
use super::super::super::theme::BPalette;
use crate::store::hosts::BHostType;
use crate::store::parts::switcher;
use gpui::prelude::*;
use gpui::{Div, InteractiveElement, SharedString, Stateful, StatefulInteractiveElement, div, px};

pub struct SwitcherHostsCol {
    pub host: BHostType,
    pub id: String,
}

impl SwitcherHostsCol {
    pub fn build(&self, p: &BPalette, on_action: ActionHandler) -> Stateful<Div> {
        let switchable = switcher::set::default_switchable();
        let mut left_col = div()
            .flex()
            .flex_col()
            .gap(px(p.gap_xs))
            .w(px(260.0))
            .h_full()
            .p(px(p.pad_sm))
            .border_r_1()
            .border_color(p.border)
            .id(SharedString::from(format!("{}-left-scroll", self.id)))
            .overflow_y_scroll()
            .child(
                div()
                    .px(px(p.pad_sm))
                    .py(px(p.pad_xs / 2.0))
                    .text_xs()
                    .text_color(p.text_dim)
                    .child(SharedString::from("EDITORS")),
            );

        for group in switcher::set::host_groups() {
            left_col = left_col.child(
                div()
                    .px(px(p.pad_sm))
                    .pt(px(p.pad_xs))
                    .text_xs()
                    .text_color(p.accent)
                    .child(SharedString::from(group.name)),
            );

            for &h in group.hosts {
                if !switchable.contains(&h) {
                    continue;
                }
                let spec = h.items();
                let is_active = h == self.host;
                let h_name = format!("{:?}", h);
                let handler = on_action.clone();

                let mut host_item = div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(p.gap_sm))
                    .px(px(p.pad_sm))
                    .py(px(p.pad_xs + 2.0))
                    .rounded(px(p.radius_sm))
                    .cursor_pointer()
                    .text_color(if is_active { p.accent } else { p.text })
                    .text_xs()
                    .hover(|s| s.bg(p.fill_sheen(p.bg_hover)).text_color(p.accent))
                    .child(
                        IconView::new(spec.icon())
                            .size(IconSize::Sm)
                            .into_element(if is_active { p.accent } else { p.text_dim }),
                    )
                    .child(SharedString::from(spec.label().to_string()));

                if is_active {
                    host_item = host_item
                        .bg(p.fill_sheen(p.bg_active))
                        .border_l_1()
                        .border_color(p.accent);
                }

                let click_id = format!("{}-host-{}", self.id, spec.label());
                let host_item =
                    host_item
                        .id(SharedString::from(click_id))
                        .on_click(move |_, window, cx| {
                            // Set the preview host (filters modes column) AND apply the actual
                            // host switch — both do NOT close the switcher popup.
                            handler(
                                &BAction::Set(BKey::SwitcherPreviewHost, h_name.clone()),
                                window,
                                cx,
                            );
                            handler(
                                &BAction::Set(BKey::SwitcherApplyHost, h_name.clone()),
                                window,
                                cx,
                            );
                        });

                left_col = left_col.child(host_item);
            }
        }

        left_col
    }
}
