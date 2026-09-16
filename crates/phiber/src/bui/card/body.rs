//! Card body component — wraps inner content with overflow handling.

use super::super::bview::{ActionHandler, BView};
use super::super::theme::BPalette;
use gpui::prelude::*;
use gpui::{AnyElement, IntoElement, div};

pub struct CardBodyView {
    pub content: BView,
}

impl CardBodyView {
    pub fn into_elm(self, p: &BPalette, on_action: ActionHandler) -> AnyElement {
        div()
            .flex_1()
            .w_full()
            .overflow_hidden()
            .child(self.content.into_elm(p, on_action))
            .into_any_element()
    }
}
