//! Statusbar view — status items from store/parts/statusbar.

pub mod stats;
pub use stats::StatusbarStats;

use super::super::{BState, BView};
use crate::store::hosts::BHostType;

pub struct StatusbarView<'a> {
    pub host: BHostType,
    pub state: &'a BState,
}

impl<'a> StatusbarView<'a> {
    pub fn new(host: BHostType, state: &'a BState) -> Self {
        Self { host, state }
    }

    pub fn build(self) -> BView {
        let _ = self.state;
        let items = StatusbarStats::build(self.host);
        BView::row(None, items)
    }
}
