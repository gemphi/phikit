//! `BToolPropsItem` — one tool-property field (label; edited via a control).

use super::set::BToolPropSpec;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem};

#[derive(Clone, Copy, Debug)]
pub struct BToolPropsItem {
    pub spec: BToolPropSpec,
}

impl BToolPropsItem {
    pub fn new(spec: BToolPropSpec) -> Self {
        Self { spec }
    }
}

impl BViewItem for BToolPropsItem {
    fn build(&self, _s: &BState) -> BView {
        BView::label(self.spec.label)
    }
}
