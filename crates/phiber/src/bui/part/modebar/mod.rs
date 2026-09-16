//! Modebar view — mode selector dropdown for the active host.

pub mod item;
pub use item::ModebarItem;

use super::super::bview::BAction;
use super::super::{BKey, BState, BView};
use crate::store::hosts::BHostType;
use crate::store::parts::{BPartType, modebar};

pub struct ModebarView<'a> {
    pub host: BHostType,
    pub state: &'a BState,
}

impl<'a> ModebarView<'a> {
    pub fn new(host: BHostType, state: &'a BState) -> Self {
        Self { host, state }
    }

    pub fn build(self) -> BView {
        let host = self.host;
        let modes = modebar::set::with_idx(host);
        let dk = BKey::Dropdown {
            host,
            part: BPartType::Modebar,
            key: "mode".to_string(),
        };
        // Mode is linked via the shared CtxMode key — switcher and modebar both read/write it.
        let active = self.state.get(&BKey::CtxMode);
        let is_open = self.state.get(&dk) == Some("true");

        let ch = ModebarItem::build_children(host, modes);

        let pick = modes
            .iter()
            .find(|t| active == Some(t.label) || (active.is_none() && t.active))
            .or_else(|| modes.first());
        let label = pick.map(|t| t.label).unwrap_or("View");
        let icon = pick.map(|t| t.icon.to_string());

        BView::dropdown_action(label, icon, ch, is_open, BAction::Toggle(dk))
    }
}
