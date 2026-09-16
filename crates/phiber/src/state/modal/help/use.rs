//! Help modal set methods - pure state mutations.

use super::HelpState;
use crate::store::BHelpType;

impl HelpState {
    pub fn set_selected_link(&mut self, link: BHelpType) {
        self.selected_link = Some(link);
    }
}
