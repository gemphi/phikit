//! Settings modal — Preferences dialog opened via `ctx.modal = "settings"`.

pub mod set;

use super::BModalType;
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem};

pub use set::{BSettingsCategory, BSettingsCategorySpec, CATEGORIES};

#[derive(Clone, Copy, Debug, Default)]
pub struct BSettingsModal;

impl BViewItem for BSettingsModal {
    fn build(&self, _s: &BState) -> BView {
        BView::label(BModalType::Settings.label())
    }
}
