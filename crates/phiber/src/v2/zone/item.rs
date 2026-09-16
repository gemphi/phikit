//! `BZoneItem` — one zone. Visibility gate + delegate to `BAreaList`.

use super::BZoneType;
use crate::v2::area::{BAreaList, BAreaSpec};
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

#[derive(Clone, Debug)]
pub struct BZoneItem<'a> {
    pub zone: BZoneType,
    pub areas: &'a [BAreaSpec],
}

impl<'a> BZoneItem<'a> {
    pub fn new(zone: BZoneType, areas: &'a [BAreaSpec]) -> Self {
        Self { zone, areas }
    }

    fn is_visible(&self, state: &BState) -> bool {
        super::visibility::is_visible(state, self.zone)
    }
}

impl<'a> BViewItem for BZoneItem<'a> {
    fn build(&self, state: &BState) -> BView {
        if self.areas.is_empty() || !self.is_visible(state) {
            return BView::empty();
        }
        BView::Zone {
            kind: self.zone,
            children: BAreaList::new(self.areas).items(state),
        }
    }
}
