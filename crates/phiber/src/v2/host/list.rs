//! `BHostList` — iterate a set of hosts (Switcher popover contents).

use super::{BHostItem, BHostType};
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

#[derive(Clone, Copy, Debug)]
pub struct BHostList<'a> {
    pub hosts: &'a [BHostType],
}

impl<'a> BHostList<'a> {
    pub fn new(hosts: &'a [BHostType]) -> Self {
        Self { hosts }
    }
}

impl<'a> BViewList for BHostList<'a> {
    fn items(&self, state: &BState) -> Vec<BView> {
        self.hosts
            .iter()
            .map(|&h| BHostItem::new(h).build(state))
            .collect()
    }
}
