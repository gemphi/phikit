//! `BMenubarList` — reads `menus_for(host)` and delegates to `BMenubarItem`.

use super::{BMenubarItem, set::menus_for};
use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

#[derive(Clone, Copy, Debug)]
pub struct BMenubarList {
    pub host: BHostType,
}

impl BMenubarList {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewList for BMenubarList {
    fn items(&self, s: &BState) -> Vec<BView> {
        menus_for(self.host)
            .iter()
            .map(|m| BMenubarItem::new(self.host, *m).build(s))
            .collect()
    }
}
