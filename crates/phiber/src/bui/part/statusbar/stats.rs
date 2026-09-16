//! Statusbar stats and info labels mapper.

use super::super::super::bview::BView;
use crate::store::hosts::BHostType;
use crate::store::parts::statusbar;

pub struct StatusbarStats;

impl StatusbarStats {
    pub fn build(host: BHostType) -> Vec<BView> {
        statusbar::set::with_idx(host)
            .iter()
            .map(|i| BView::label(i.label))
            .collect()
    }
}
