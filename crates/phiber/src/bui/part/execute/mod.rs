//! Execute bar view builder (OK / Cancel / Apply).

use crate::bui::bview::{BView, CardShade};
use crate::bui::state::BState;
use crate::store::hosts::BHostType;

pub struct ExecuteView<'a> {
    pub host: BHostType,
    pub state: &'a BState,
}

impl<'a> ExecuteView<'a> {
    pub fn new(host: BHostType, state: &'a BState) -> Self {
        Self { host, state }
    }

    pub fn build(self) -> BView {
        let spec = crate::store::parts::execute::set::with_idx(self.host);
        let mut items: Vec<BView> = Vec::new();
        if !spec.field.is_empty() {
            items.push(BView::label(spec.field));
        }
        items.push(BView::button().with_label(spec.cancel));
        items.push(BView::button().with_label(spec.ok).with_active(true));
        BView::card()
            .with_shade(CardShade::Panel)
            .with_body(BView::row(Some(8.0), items))
    }
}
