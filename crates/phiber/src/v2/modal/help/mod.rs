//! Help modal — Blender's Help menu items as an overlay dialog.

pub mod set;

use super::BModalType;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem};

pub use set::{ACTIONS, BHelpEntry, WEB_LINKS};

#[derive(Clone, Copy, Debug, Default)]
pub struct BHelpModal;

impl BViewItem for BHelpModal {
    fn build(&self, _s: &BState) -> BView {
        BView::label(BModalType::Help.label())
    }
}
