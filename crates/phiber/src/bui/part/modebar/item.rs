//! Mode item mapper for modebar dropdown.

use super::super::super::bview::{BAction, BView};
use super::super::super::state::BKey;
use crate::store::hosts::BHostType;
use crate::store::parts::modebar::ModeTab;

pub struct ModebarItem;

impl ModebarItem {
    pub fn build_children(_host: BHostType, modes: &[ModeTab]) -> Vec<BView> {
        modes
            .iter()
            .map(|t| {
                BView::dropdown_action(
                    t.label,
                    Some(t.icon.to_string()),
                    vec![],
                    false,
                    BAction::Set(BKey::CtxMode, t.label.to_string()),
                )
            })
            .collect()
    }
}
