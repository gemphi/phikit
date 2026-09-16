//! Default settings modal state.

use super::SettingsState;
use crate::state::modal::BModalState;
use crate::store::BModalType;
use crate::store::BSettingsType;

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            active_category: BSettingsType::Interface,
            search: String::new(),
        }
    }
}

impl BModalState for SettingsState {
    fn idx(&self) -> BModalType {
        BModalType::Settings
    }
    fn title(&self) -> &'static str {
        "Settings"
    }
}
