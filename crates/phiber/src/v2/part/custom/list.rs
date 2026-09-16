//! `BCustomList` — one `BCustomItem` per host that has center content.

use super::{BCustomItem, set::hosts_with_center};
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

#[derive(Clone, Copy, Debug, Default)]
pub struct BCustomList;

impl BCustomList {
    pub fn new() -> Self {
        Self
    }
}

impl BViewList for BCustomList {
    fn items(&self, s: &BState) -> Vec<BView> {
        hosts_with_center()
            .iter()
            .map(|&h| BCustomItem::new(h).build(s))
            .collect()
    }
}
