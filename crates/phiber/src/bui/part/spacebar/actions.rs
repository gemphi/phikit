//! Spacebar modal trigger buttons generator.

use super::super::super::bview::{BAction, BView};
use super::super::super::state::BKey;
use crate::store::modals::BModalType;

pub struct SpacebarActions;

impl SpacebarActions {
    pub fn build() -> Vec<BView> {
        BModalType::all()
            .iter()
            .map(|&m| {
                BView::button()
                    .with_icon(m.icon())
                    .with_action(BAction::Set(BKey::Modal, m.name().to_string()))
            })
            .collect()
    }
}
