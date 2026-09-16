//! Navigation Bar view builder.

use crate::bui::bview::{BView, CardShade};
use crate::bui::state::BState;
use crate::store::hosts::BHostType;

pub struct NavBarView<'a> {
    pub host: BHostType,
    pub state: &'a BState,
}

impl<'a> NavBarView<'a> {
    pub fn new(host: BHostType, state: &'a BState) -> Self {
        Self { host, state }
    }

    pub fn build(self) -> BView {
        let crumbs = crate::store::parts::navbar::set::with_idx(self.host);
        if crumbs.is_empty() {
            return BView::empty();
        }
        // Pure loop over the store's static breadcrumb trail.
        let mut items: Vec<BView> = Vec::new();
        for (i, c) in crumbs.iter().enumerate() {
            if i > 0 {
                items.push(BView::label(">"));
            }
            items.push(BView::label(*c));
        }
        BView::card()
            .with_shade(CardShade::Header)
            .with_body(BView::row(Some(4.0), items))
    }
}
