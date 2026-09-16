//! PropRowView — label-value property row with space-between, left-aligned.

use super::super::theme::BPalette;
use gpui::prelude::*;
use gpui::{AnyElement, SharedString, div, px};

pub struct PropRowView {
    pub label: String,
    pub value: String,
}

impl PropRowView {
    pub fn new(label: String, value: String) -> Self {
        Self { label, value }
    }

    pub fn into_elm(&self, p: &BPalette) -> AnyElement {
        div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .w_full()
            .px(px(p.pad_sm))
            .py(px(p.pad_xs))
            .h(px(p.h_tab))
            .text_color(p.text)
            .text_xs()
            .child(SharedString::from(self.label.clone()))
            .child(
                div()
                    .text_color(p.text_dim)
                    .child(SharedString::from(self.value.clone())),
            )
            .into_any_element()
    }
}
