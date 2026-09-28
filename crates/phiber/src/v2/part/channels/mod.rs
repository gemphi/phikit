//! Channels — vertical list of animation channels.

pub mod set;

use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
use crate::v2::view::BViewList;
use self::set::channels_for;
pub use set::BChannelSpec;

// `BChannelsItem` — one channel row; carries a `BChannelSpec`.
#[derive(Clone, Copy, Debug)]
pub struct BChannelsItem {
    pub spec: BChannelSpec,
}

impl BChannelsItem {
    pub fn new(spec: BChannelSpec) -> Self {
        Self { spec }
    }
}

impl BViewItem for BChannelsItem {
    fn build(&self, _s: &BState) -> BView {
        BView::label(self.spec.label)
    }
}

// `BChannelsList` — reads `channels_for(host)`.

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

#[derive(Clone, Copy, Debug)]
pub struct BChannels {
    pub host: BHostType,
}

impl BChannels {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BChannels {
    fn build(&self, s: &BState) -> BView {
        BView::column(BChannelsList::new(self.host).items(s))
    }
}
