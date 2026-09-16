//! `BModalItem` — one modal. Emits `BView::Modal` populated by the
//! per-modal content module.

use super::{BModalType, command, file_browser, help, profile, settings, splash};
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem};

#[derive(Clone, Copy, Debug)]
pub struct BModalItem {
    pub modal: BModalType,
}

impl BModalItem {
    pub fn new(modal: BModalType) -> Self {
        Self { modal }
    }
}

impl BViewItem for BModalItem {
    fn build(&self, s: &BState) -> BView {
        let body = match self.modal {
            BModalType::Settings => settings::BSettingsModal.build(s),
            BModalType::Help => help::BHelpModal.build(s),
            BModalType::Profile => profile::BProfileModal.build(s),
            BModalType::Command => command::BCommandModal.build(s),
            BModalType::FileBrowser => file_browser::BFileBrowserModal.build(s),
            BModalType::Splash => splash::BSplashModal.build(s),
            _ => BView::empty(),
        };
        BView::Modal {
            kind: self.modal,
            body: Box::new(body),
        }
    }
}
