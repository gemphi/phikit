//! The screen's own flags — Blender's `wm.*` / `screen.*` checkables the
//! top bar's menus flip: `ctx.screen.*`. None of them changes what is
//! drawn yet; each is kept so the menu shows what was chosen.

use super::BKey;
use crate::v2::state::BPath;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ScreenKey {
    /// `Render ▸ Lock Interface`.
    LockInterface,
    /// `Preferences ▸ Auto-Save Preferences`.
    AutoSavePrefs,
}

impl BKey for ScreenKey {
    fn path(&self) -> BPath {
        let tail: &'static str = match self {
            Self::LockInterface => "lock_interface",
            Self::AutoSavePrefs => "auto_save_prefs",
        };
        BPath::owned(vec!["screen", tail])
    }
}
