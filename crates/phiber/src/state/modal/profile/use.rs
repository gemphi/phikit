//! Profile modal set methods - pure state mutations.

use super::ProfileState;
use crate::store::BProfileType;

impl ProfileState {
    pub fn set_action(&mut self, action: BProfileType) {
        self.selected_action = Some(action);
    }

    pub fn set_user(&mut self, user: Option<String>) {
        self.user = user;
    }
}
