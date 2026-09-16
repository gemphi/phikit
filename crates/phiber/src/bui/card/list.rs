//! Card list component — vertical list of cards or items with configurable gap.

use super::super::bview::{ActionHandler, BView};
use super::super::theme::BPalette;
use gpui::prelude::*;
use gpui::{AnyElement, IntoElement, div, px};

pub struct CardListView {
    pub gap: Option<f32>,
    pub items: Vec<BView>,
}

impl CardListView {
    pub fn into_elm(self, p: &BPalette, on_action: ActionHandler) -> AnyElement {
        let gap_px = self.gap.unwrap_or(p.gap_xs);
        let mut list = div().flex().flex_col().gap(px(gap_px)).w_full();
        for item in self.items {
            list = list.child(item.into_elm(p, on_action.clone()));
        }
        list.into_any_element()
    }
}
