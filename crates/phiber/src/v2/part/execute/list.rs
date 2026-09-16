//! `BExecuteList` — action buttons for the active host.

use super::{BExecuteItem, set::actions_for};
use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

#[derive(Clone, Copy, Debug)]
pub struct BExecuteList {
    pub host: BHostType,
}

impl BExecuteList {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewList for BExecuteList {
    fn items(&self, s: &BState) -> Vec<BView> {
        actions_for(self.host)
            .iter()
            .map(|a| BExecuteItem::new(*a).build(s))
            .collect()
    }
}
