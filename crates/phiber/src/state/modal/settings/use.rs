//! Settings modal set methods - pure state mutations.

use super::SettingsState;
use crate::store::BSettingsType;

impl SettingsState {
    pub fn set_category(&mut self, category: BSettingsType) {
        self.active_category = category;
    }

    pub fn set_search(&mut self, query: String) {
        self.search = query;
    }
}
