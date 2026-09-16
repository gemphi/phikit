//! Profile modal — Phiber user identity + session actions.

pub mod set;

use super::BModalType;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem};

pub use set::{ACTIONS, BProfileField, FIELDS};

#[derive(Clone, Copy, Debug, Default)]
pub struct BProfileModal;

impl BViewItem for BProfileModal {
    fn build(&self, _s: &BState) -> BView {
        BView::label(BModalType::Profile.label())
    }
}
