//! Channels view — animation channels from store/parts/channels.

pub mod item;
pub use item::ChannelItem;

use super::super::{BState, BView};
use crate::store::hosts::BHostType;

pub struct ChannelsView<'a> {
    pub host: BHostType,
    pub state: &'a BState,
}

impl<'a> ChannelsView<'a> {
    pub fn new(host: BHostType, state: &'a BState) -> Self {
        Self { host, state }
    }

    pub fn build(self) -> BView {
        let _ = self.state;
        let items = ChannelItem::build(self.host);
        BView::column(None, items)
    }
}
