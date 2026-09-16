//! Card footer component — status text and footer action buttons.

use super::super::bview::{ActionHandler, BView};
use super::super::theme::BPalette;
use gpui::prelude::*;
use gpui::{AnyElement, IntoElement, SharedString, div, px};

pub struct CardFooterView {
    pub text: Option<String>,
    pub action: Option<Box<BView>>,
}

impl CardFooterView {
    pub fn into_elm(self, p: &BPalette, on_action: ActionHandler) -> AnyElement {
        let mut footer_row = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .h(px(p.h_footer))
            .px(px(p.pad_sm))
            .bg(p.fill_sheen(p.bg_header))
            .border_t_1()
            .border_color(p.border);

        if let Some(footer_text) = &self.text {
            footer_row = footer_row.child(
                div()
                    .text_xs()
                    .text_color(p.text_dim)
                    .child(SharedString::from(footer_text.clone())),
            );
        } else {
            footer_row = footer_row.child(div());
        }

        if let Some(footer_act) = self.action {
            footer_row = footer_row.child(footer_act.into_elm(p, on_action));
        }

        footer_row.into_any_element()
    }
}
