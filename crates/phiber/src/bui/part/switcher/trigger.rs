//! Switcher trigger button component struct.

use super::super::super::icon::{IconSize, IconView};
use super::super::super::theme::BPalette;
use crate::store::hosts::BHostType;
use gpui::prelude::*;
use gpui::{Div, InteractiveElement, SharedString, div, px};

pub struct SwitcherTrigger {
    pub host: BHostType,
    pub open: bool,
}

impl SwitcherTrigger {
    pub fn build(&self, p: &BPalette) -> Div {
        let host_spec = self.host.items();
        let icon_name = host_spec.icon();
        let label = host_spec.label();

        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(p.gap_sm))
            .cursor_pointer()
            .h(px(p.h_tab))
            .px(px(p.pad_sm))
            .rounded(px(p.radius_sm))
            .border_1()
            .border_color(if self.open { p.accent } else { p.border })
            .bg(if self.open {
                p.fill_sheen(p.bg_icon_active)
            } else {
                p.fill_sheen(p.bg_icon)
            })
            .hover(|s| s.bg(p.fill_sheen(p.bg_icon_hover)).border_color(p.accent))
            .child(
                IconView::new(icon_name)
                    .size(IconSize::Md)
                    .into_element(if self.open { p.accent } else { p.text }),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(if self.open { p.accent } else { p.text })
                    .child(SharedString::from(label.to_string())),
            )
    }
}
