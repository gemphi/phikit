//! Settings modal — Preferences dialog opened via `ctx.modal = "settings"`.

pub mod set;

use crate::v2::host::BHostItem;
use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
pub use set::BSettingsCategory;
pub use set::BSettingsCategorySpec;
pub use set::CATEGORIES;
pub use set::NAV_TABS;
pub use set::category_for;

#[derive(Clone, Copy, Debug, Default)]
pub struct BSettingsModal;

impl BViewItem for BSettingsModal {
    /// The Preferences editor itself, elevated — one implementation for the
    /// workspace host and the dialog (Blender opens `SPACE_USERPREF` in a
    /// new window the same way).
    fn build(&self, s: &BState) -> BView {
        BHostItem::new(BHostType::Prefs).build(s)
    }
}
