//! Menubar view — top-level and editor header menus.

pub mod header;
pub mod topbar;

pub use header::HeaderMenus;
pub use topbar::TopbarMenus;

use super::super::{BState, BView};
use crate::store::hosts::BHostType;
use crate::store::parts::BPartType;

pub struct MenubarView<'a> {
    pub host: BHostType,
    pub state: &'a BState,
}

impl<'a> MenubarView<'a> {
    pub fn new(host: BHostType, state: &'a BState) -> Self {
        Self { host, state }
    }

    pub fn build(self) -> BView {
        let host = self.host;
        let state = self.state;
        let part = BPartType::Menubar;

        let items = match host {
            BHostType::Topbar => TopbarMenus::build(state, host, part),
            _ => HeaderMenus::build(state, host, part),
        };

        BView::row(None, items)
    }
}
