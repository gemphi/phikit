//! Command Palette — searchable operator/menu launcher.
//! Blender ref: F3 "Menu Search" and "Operator Search" (Developer Extras).

pub mod set;

use super::BModalType;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem};

pub use set::{BCommandField, BCommandKind, SOURCES};

#[derive(Clone, Copy, Debug, Default)]
pub struct BCommandModal;

impl BViewItem for BCommandModal {
    fn build(&self, _s: &BState) -> BView {
        BView::label(BModalType::Command.label())
    }
}
