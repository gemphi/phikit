//! Default help modal state.

use super::HelpState;
use crate::state::modal::BModalState;
use crate::store::BModalType;

impl Default for HelpState {
    fn default() -> Self {
        Self {
            selected_link: None,
        }
    }
}

impl BModalState for HelpState {
    fn idx(&self) -> BModalType {
        BModalType::Help
    }
    fn title(&self) -> &'static str {
        "Help"
    }
}
