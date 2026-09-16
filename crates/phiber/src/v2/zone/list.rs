//! `BZoneList` — iterate the zones of a space.

use super::{BZoneItem, BZoneType};
use crate::v2::space::BSpaceSpec;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

#[derive(Clone, Copy, Debug)]
pub struct BZoneList<'a> {
    pub space: &'a BSpaceSpec,
}

impl<'a> BZoneList<'a> {
    pub fn new(space: &'a BSpaceSpec) -> Self {
        Self { space }
    }
}

impl<'a> BViewList for BZoneList<'a> {
    fn items(&self, state: &BState) -> Vec<BView> {
        BZoneType::ALL
            .iter()
            .map(|&z| BZoneItem::new(z, self.space.areas_in(z)).build(state))
            .filter(|v| !v.is_empty())
            .collect()
    }
}
