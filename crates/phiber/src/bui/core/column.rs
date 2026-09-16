//! ColumnView — vertical flex container.
//! Pure layout wrapper with no visual chrome.

use crate::bui::bview::{ActionHandler, BView};
use crate::bui::theme::BPalette;
use gpui::prelude::*;
use gpui::{AnyElement, Div, div, px};

pub struct ColumnView {
    pub gap: Option<f32>,
    pub children: Vec<BView>,
    pub center: bool,
}

impl ColumnView {
    pub fn new(gap: Option<f32>, children: Vec<BView>) -> Self {
        Self {
            gap,
            children,
            center: false,
        }
    }

    pub fn centered(gap: Option<f32>, children: Vec<BView>) -> Self {
        Self {
            gap,
            children,
            center: true,
        }
    }

    pub fn with_div(&self, _p: &BPalette) -> Div {
        let base = div().flex().flex_col().min_h_0();
        let mut el = if self.center {
            base.items_center()
        } else {
            base.items_stretch()
        };
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
