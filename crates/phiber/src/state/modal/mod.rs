//! Modal state - ModalState enum, per-modal state, and body dispatch.
//!
//! Modals follow the same store/state/view pattern as spaces.
//! No handler trait, no registry - just state structs + view functions.
//!
//! Structure (mod.rs/def.rs/ctx.rs/use.rs per modal):
//!   mod.rs     - state struct + view function
//!   def.rs     - Default impl
//!   ctx.rs     - context sync (modals bypass BContext)
//!   use.rs     - set methods (pure state mutations)
//!
//! Note: AreaSwitcher is no longer a modal - it's a `BPartType::Switcher`
//! menu part in the header region.

pub mod ctx;
pub mod def;
pub mod help;
pub mod profile;
pub mod settings;
pub mod r#use;

pub use help::HelpState;
pub use profile::ProfileState;
pub use settings::SettingsState;

use crate::store::BModalType;

/// Trait every modal state must implement.
pub trait BModalState {
    fn idx(&self) -> BModalType;
    fn title(&self) -> &'static str;
    fn is_open(&self) -> bool {
        true
    }
}

use crate::bui::BView;
use gpui::App;

/// Tracks which modal is open and its internal state.
#[derive(Clone, Debug)]
pub enum ModalState {
    Closed,
    Settings(SettingsState),
    Help(HelpState),
    Profile(ProfileState),
    FileBrowser,
}

/// Dispatch: produce the modal body BView tree from state.
/// The app's view layer calls this, wraps the result in b_modal(), and renders.
pub fn modal_body(state: &ModalState, cx: &App) -> Vec<BView> {
    match state {
        ModalState::Settings(s) => settings::settings_modal_view(s, cx),
        ModalState::Help(s) => help::help_view(s, cx),
        ModalState::Profile(s) => profile::profile_modal_view(s, cx),
        ModalState::FileBrowser => Vec::new(),
        ModalState::Closed => Vec::new(),
    }
}
