//! Sidebar view — vertical icon tabs + expand toggle.

pub mod tabs;
pub use tabs::SidebarTabs;

use super::super::{BKey, BState, BView};
use crate::store::hosts::BHostType;
use crate::store::parts::BPartType;

pub struct SidebarView<'a> {
    pub host: BHostType,
    pub state: &'a BState,
}

impl<'a> SidebarView<'a> {
    pub fn new(host: BHostType, state: &'a BState) -> Self {
        Self { host, state }
    }

    pub fn build(self) -> BView {
        let host = self.host;
        let state = self.state;
        let exp = state.get(&BKey::Expanded {
            host,
            part: BPartType::Sidebar,
        }) == Some("true");
        let active = state.get(&BKey::Tab {
            host,
            part: BPartType::Sidebar,
        });

        let tabs = SidebarTabs::build(host, active, exp);
        BView::vtab_bar(tabs)
    }
}
