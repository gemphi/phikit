//! Spacebar view — workspace tabs + modal buttons in the topbar.

pub mod actions;
pub mod tabs;

pub use actions::SpacebarActions;
pub use tabs::SpacebarTabs;

use super::super::theme::BPalette;
use super::super::{BKey, BState, BView};
use crate::store::hosts::BHostType;
use crate::store::spaces::BSpaceType;

pub struct SpacebarView<'a> {
    pub host: BHostType,
    pub state: &'a BState,
}

impl<'a> SpacebarView<'a> {
    pub fn new(host: BHostType, state: &'a BState) -> Self {
        Self { host, state }
    }

    pub fn build(self) -> BView {
        let active = self
            .state
            .get(&BKey::CtxSpace)
            .and_then(BSpaceType::from_name)
            .unwrap_or(BSpaceType::Layout);

        let tabs = SpacebarTabs::build(active);
        let actions = SpacebarActions::build();
        let p = BPalette::from_state(self.state);

        BView::row(
            Some(p.gap_md),
            vec![BView::htab_bar(tabs), BView::row(Some(p.gap_md), actions)],
        )
    }
}
