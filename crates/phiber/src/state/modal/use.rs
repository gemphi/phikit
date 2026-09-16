//! Modal state set methods.

use super::{HelpState, ModalState, ProfileState, SettingsState};
use crate::store::BModalType;

impl ModalState {
    pub fn is_open(&self) -> bool {
        !matches!(self, ModalState::Closed)
    }

    pub fn idx(&self) -> Option<BModalType> {
        match self {
            ModalState::Closed => None,
            ModalState::Settings(_) => Some(BModalType::Settings),
            ModalState::Help(_) => Some(BModalType::Help),
            ModalState::Profile(_) => Some(BModalType::Profile),
            ModalState::FileBrowser => Some(BModalType::FileBrowser),
        }
    }

    pub fn set_open(&mut self, modal_type: BModalType) {
        *self = match modal_type {
            BModalType::Settings => ModalState::Settings(SettingsState::default()),
            BModalType::Help => ModalState::Help(HelpState::default()),
            BModalType::Profile => ModalState::Profile(ProfileState::default()),
            BModalType::FileBrowser => ModalState::FileBrowser,
        };
    }

    pub fn set_close(&mut self) {
        *self = ModalState::Closed;
    }

    pub fn set_toggle(&mut self, modal_type: BModalType) {
        if self.idx() == Some(modal_type) {
            self.set_close();
        } else {
            self.set_open(modal_type);
        }
    }
}
