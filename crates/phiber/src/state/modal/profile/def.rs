//! Default profile modal state.

use super::ProfileState;
use crate::state::modal::BModalState;
use crate::store::BModalType;

impl Default for ProfileState {
    fn default() -> Self {
        Self {
            user: None,
            selected_action: None,
        }
    }
}

impl BModalState for ProfileState {
    fn idx(&self) -> BModalType {
        BModalType::Profile
    }
    fn title(&self) -> &'static str {
        "Profile"
    }
}
