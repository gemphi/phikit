//! ScrollView — overflow container (vertical scroll, flex-1).
//! Pure layout wrapper with no visual chrome.

use crate::bui::bview::{ActionHandler, BView};
use crate::bui::theme::BPalette;
use gpui::prelude::*;
use gpui::{AnyElement, div, px};

pub struct ScrollView {
    pub children: Vec<BView>,
}

impl ScrollView {
    pub fn new(children: Vec<BView>) -> Self {
        Self { children }
    }

    pub fn into_elm(&self, p: &BPalette, on_action: ActionHandler) -> AnyElement {
        let mut el = div()
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .w_full()
            .gap(px(p.gap_sm))
            .id(format!("scroll-{}", self.children.len()))
            .overflow_y_scroll();
        for c in &self.children {
            el = el.child(c.into_elm(p, on_action.clone()));
        }
        el.into_any_element()
    }
}
