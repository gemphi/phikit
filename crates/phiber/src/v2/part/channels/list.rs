//! `BChannelsList` — reads `channels_for(host)`.

use super::{BChannelsItem, set::channels_for};
use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

#[derive(Clone, Copy, Debug)]
pub struct BChannelsList {
    pub host: BHostType,
}

impl BChannelsList {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewList for BChannelsList {
    fn items(&self, s: &BState) -> Vec<BView> {
        channels_for(self.host)
            .iter()
            .map(|c| BChannelsItem::new(*c).build(s))
            .collect()
    }
}
