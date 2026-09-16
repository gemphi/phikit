//! RowView — horizontal flex container.
//! Pure layout wrapper with no visual chrome.

use crate::bui::bview::{ActionHandler, BView};
use crate::bui::theme::BPalette;
use gpui::prelude::*;
use gpui::{AnyElement, Div, div, px};

pub struct RowView {
    pub gap: Option<f32>,
    pub children: Vec<BView>,
}

impl RowView {
    pub fn new(gap: Option<f32>, children: Vec<BView>) -> Self {
        Self { gap, children }
    }

    pub fn with_div(&self, _p: &BPalette) -> Div {
        let mut el = div().flex().flex_row().items_stretch().min_w_0();
        if let Some(g) = self.gap {
            el = el.gap(px(g));
        }
        el
    }

    pub fn into_elm(&self, p: &BPalette, on_action: ActionHandler) -> AnyElement {
        let mut el = self.with_div(p);
        for c in &self.children {
            el = el.child(c.into_elm(p, on_action.clone()));
        }
        el.into_any_element()
    }
}
