//! HTabBarView / VTabBarView — horizontal and vertical tab bar containers.

use super::super::bview::{ActionHandler, BView};
use super::super::theme::BPalette;
use gpui::prelude::*;
use gpui::{AnyElement, Div, div, px};

pub struct HTabBarView {
    pub children: Vec<BView>,
}

impl HTabBarView {
    pub fn new(children: Vec<BView>) -> Self {
        Self { children }
    }

    pub fn with_div(&self, p: &BPalette) -> Div {
        div()
            .flex()
            .flex_row()
            .items_end()
            .gap(px(p.gap_sm))
            .overflow_hidden()
    }

    pub fn into_elm(&self, p: &BPalette, on_action: ActionHandler) -> AnyElement {
        let mut el = self.with_div(p);
        for c in &self.children {
            el = el.child(c.into_elm(p, on_action.clone()));
        }
        el.into_any_element()
    }
}

pub struct VTabBarView {
    pub width: Option<f32>,
    pub children: Vec<BView>,
}

impl VTabBarView {
    pub fn new(width: Option<f32>, children: Vec<BView>) -> Self {
        Self { width, children }
    }

    pub fn with_div(&self, p: &BPalette) -> Div {
        let w = self.width.unwrap_or(p.w_sidebar);
        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(p.gap_xs))
            .w(px(w))
            .p(px(p.pad_xs))
            .flex_shrink_0()
            .overflow_hidden()
    }

    pub fn into_elm(&self, p: &BPalette, on_action: ActionHandler) -> AnyElement {
        let mut el = self.with_div(p);
        for c in &self.children {
            el = el.child(c.into_elm(p, on_action.clone()));
        }
        el.into_any_element()
    }
}
