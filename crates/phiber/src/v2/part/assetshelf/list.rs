//! `BAssetShelfList` — brush tiles for the active host + mode.

use super::{BAssetShelfItem, set::tiles_for};
use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

#[derive(Clone, Copy, Debug)]
pub struct BAssetShelfList {
    pub host: BHostType,
}

impl BAssetShelfList {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewList for BAssetShelfList {
    fn items(&self, s: &BState) -> Vec<BView> {
        let active = s.host(self.host).get("brush");
        tiles_for(self.host)
            .iter()
            .map(|t| BAssetShelfItem::new(*t, Some(t.label) == active).build(s))
            .collect()
    }
}
