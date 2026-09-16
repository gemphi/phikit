//! Channel item mapper.

use super::super::super::bview::BView;
use crate::store::hosts::BHostType;
use crate::store::parts::channels;

pub struct ChannelItem;

impl ChannelItem {
    pub fn build(host: BHostType) -> Vec<BView> {
        channels::set::with_idx(host)
            .iter()
            .map(|c| BView::label_icon(c.idx, c.icon))
            .collect()
    }
}
