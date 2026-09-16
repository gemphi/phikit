//! Switcher right column — modes & editor capabilities list component struct.

use super::super::super::bview::{ActionHandler, BAction};
use super::super::super::icon::{IconSize, IconView};
use super::super::super::state::BKey;
use super::super::super::theme::BPalette;
use crate::store::hosts::BHostType;
use crate::store::parts::switcher;
use gpui::prelude::*;
use gpui::{
    App, ClickEvent, Div, InteractiveElement, SharedString, Stateful, StatefulInteractiveElement,
    Window, div, px,
};

pub struct SwitcherModesCol {
    pub host: BHostType,
    pub id: String,
}

impl SwitcherModesCol {
    pub fn build(&self, p: &BPalette, on_action: ActionHandler) -> Stateful<Div> {
        let display_host = self.host;

        let mut right_col = div()
            .flex()
            .flex_col()
            .gap(px(p.gap_xs))
            .flex_1()
            .h_full()
            .p(px(p.pad_sm))
            .bg(p.fill_sheen(p.bg_panel))
            .id(SharedString::from(format!("{}-right-scroll", self.id)))
            .overflow_y_scroll()
            .child(
                div()
                    .py(px(p.pad_xs / 2.0))
                    .text_xs()
                    .text_color(p.text_dim)
                    .child(SharedString::from(format!(
                        "MODES: {}",
                        display_host.label().to_uppercase()
                    ))),
            );

        for mode in switcher::set::with_idx(display_host) {
            let handler = on_action.clone();
            let mode_str = mode.label.to_string();
            let target_host_name = format!("{:?}", display_host);

            let mode_item = div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(p.gap_sm))
                .px(px(p.pad_sm))
                .py(px(p.pad_xs + 2.0))
                .rounded(px(p.radius_sm))
                .cursor_pointer()
                .text_color(p.text)
                .text_xs()
                .hover(|s| s.bg(p.fill_sheen(p.bg_hover)).text_color(p.accent))
                .child(
                    IconView::new(mode.icon)
                        .size(IconSize::Sm)
                        .into_element(p.text_dim),
                )
                .child(SharedString::from(mode.label))
                .id(SharedString::from(format!(
                    "mode-item-{:?}-{}",
                    display_host, mode_str
                )))
                .on_click(move |_e: &ClickEvent, w: &mut Window, cx: &mut App| {
                    // Set the mode via shared CtxMode key, and apply the host switch.
                    // This does NOT close the switcher — only the trigger icon toggles it.
                    handler(&BAction::Set(BKey::CtxMode, mode_str.clone()), w, cx);
                    // Apply the actual host switch (SwitcherApplyHost does NOT close dropdowns).
                    handler(
                        &BAction::Set(BKey::SwitcherApplyHost, target_host_name.clone()),
                        w,
                        cx,
                    );
                });

            right_col = right_col.child(mode_item);
        }

        right_col
    }
}
