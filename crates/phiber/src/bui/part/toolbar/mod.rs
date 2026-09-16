//! Toolbar view — icon buttons + expand toggle for the active host.

pub mod item;
pub use item::ToolbarItem;

use super::super::{BKey, BState, BView};
use crate::store::hosts::BHostType;
use crate::store::parts::{BPartType, toolbar};

pub struct ToolbarView<'a> {
    pub host: BHostType,
    pub state: &'a BState,
}

impl<'a> ToolbarView<'a> {
    pub fn new(host: BHostType, state: &'a BState) -> Self {
        Self { host, state }
    }

    pub fn build(self) -> BView {
        let host = self.host;
        let exp = self.state.get(&BKey::Expanded {
            host,
            part: BPartType::Toolbar,
        }) == Some("true");
        let current_tool = self.state.get(&BKey::CtxTool);
        let tools = toolbar::set::with_idx(host);

        let btns = ToolbarItem::build(host, tools, current_tool, exp);
        BView::icon_list(btns)
    }
}
