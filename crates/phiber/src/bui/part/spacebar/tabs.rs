//! Spacebar workspace tabs generator using fluent .with_* methods.

use super::super::super::bview::{BAction, BView};
use super::super::super::state::BKey;
use crate::store::spaces::BSpaceType;

pub struct SpacebarTabs;

impl SpacebarTabs {
    pub fn build(active: BSpaceType) -> Vec<BView> {
        BSpaceType::all()
            .iter()
            .map(|&st| {
                let is_active = st == active;
                let action = BAction::Set(BKey::CtxSpace, format!("{:?}", st));
                BView::tab(st.label())
                    .with_icon(st.icon())
                    .with_active(is_active)
                    .with_action(action)
            })
            .collect()
    }
}
