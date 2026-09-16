//! Channels — vertical list of animation channels.

pub mod item;
pub mod list;
pub mod set;

use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

pub use item::BChannelsItem;
pub use list::BChannelsList;
pub use set::BChannelSpec;

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
