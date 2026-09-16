//! LabelView — text label with optional icon badge.
//! Popup items are plain (no bg) with lighter color on hover only.

use super::super::bview::ActionHandler;
use super::super::icon::{IconSize, IconView};
use super::super::theme::BPalette;
use gpui::prelude::*;
use gpui::{AnyElement, Div, SharedString, div, px};

pub struct LabelView {
    pub text: String,
    pub icon: Option<String>,
}

impl LabelView {
    pub fn new(text: String, icon: Option<String>) -> Self {
        Self { text, icon }
    }

    pub fn with_elm(&self, p: &BPalette) -> Div {
        let mut el = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(p.gap_md))
            .px(px(p.pad_sm))
            .py(px(p.pad_xs))
            .text_color(p.text)
            .text_xs();

        if let Some(icon) = &self.icon {
            if !icon.is_empty() {
                el = el.child(IconView::new(icon).size(IconSize::Sm).into_element(p.text));
            }
        }

        el.child(SharedString::from(self.text.clone()))
    }

    pub fn into_elm(&self, p: &BPalette, _on_action: ActionHandler) -> AnyElement {
        self.with_elm(p).into_any_element()
    }

    /// Popup item — plain, no bg, lighter color on hover only.
    pub fn into_popup_item(&self, p: &BPalette) -> AnyElement {
        let mut el = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(p.gap_sm))
            .px(px(p.pad_sm))
            .py(px(p.pad_xs))
            .h(px(p.h_tab))
            .text_color(p.text)
            .text_xs()
            .rounded(px(p.radius_sm))
            .hover(|s| s.bg(p.fill_solid(p.bg_hover)))
            .id(format!("popup-label-{}", self.text));
        if let Some(ic) = &self.icon {
            if !ic.is_empty() {
                el = el.child(IconView::new(ic).size(IconSize::Sm).into_element(p.text));
            }
        }
        el.child(SharedString::from(self.text.clone()))
            .into_any_element()
    }
}
