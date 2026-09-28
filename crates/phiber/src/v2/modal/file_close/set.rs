//! The dialog's content: its title, the file's name, and the three answers
//! in the order Blender lays them out — Save, Don't Save, Cancel on
//! Windows; Don't Save, then Cancel and Save, elsewhere
//! (`block_create__close_file_dialog`'s `windows_layout`).

use crate::v2::state::{BKey as _, BState, OpKey};
use crate::v2::view::prim::{BAction, BControl};
use crate::v2::view::BView;

/// The dialog's question.
pub const TITLE: &str = "Save changes before closing?";

/// The three answers, as `ctx.op.file_close` takes them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BCloseAnswer {
    /// `wm_block_file_close_save`: write the file (asking where when it has
    /// no path), then carry on.
    Save,
    /// `wm_block_file_close_discard`: carry on without writing.
    Discard,
    /// `wm_block_file_close_cancel`: stay.
    Cancel,
}

impl BCloseAnswer {
    pub fn slug(self) -> &'static str {
        match self {
            Self::Save => "save",
            Self::Discard => "discard",
            Self::Cancel => "cancel",
        }
    }

    pub fn from_slug(slug: &str) -> Option<Self> {
        [Self::Save, Self::Discard, Self::Cancel].into_iter().find(|a| a.slug() == slug)
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Save => "Save",
            Self::Discard => "Don't Save",
            Self::Cancel => "Cancel",
        }
    }

    pub fn button(self) -> BView {
        BView::Button(BControl::new().with_label(self.label()).with_action(BAction::Set(OpKey::FileClose.path(), self.slug().to_string())))
    }
}

/// The answers in the platform's order.
pub fn buttons() -> [BCloseAnswer; 3] {
    if cfg!(windows) {
        [BCloseAnswer::Save, BCloseAnswer::Discard, BCloseAnswer::Cancel]
    } else {
        [BCloseAnswer::Discard, BCloseAnswer::Cancel, BCloseAnswer::Save]
    }
}

/// The document's file name, or `Untitled.blend` for one never saved.
pub fn file_name(s: &BState) -> String {
    s.get(&crate::v2::scene::loaded_path_key())
        .filter(|p| !p.is_empty())
        .and_then(|p| p.rsplit(['/', '\\']).next())
        .map_or_else(|| "Untitled.blend".to_string(), str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_dialog_names_the_file_and_offers_the_three_answers() {
        let mut s = BState::new();
        assert_eq!(file_name(&s), "Untitled.blend");
        s.set(&crate::v2::scene::loaded_path_key(), "C:\\work\\shot.blend");
        assert_eq!(file_name(&s), "shot.blend");
        let labels: Vec<&str> = buttons().iter().map(|a| a.label()).collect();
        if cfg!(windows) {
            assert_eq!(labels, ["Save", "Don't Save", "Cancel"]);
        } else {
            assert_eq!(labels, ["Don't Save", "Cancel", "Save"]);
        }
        assert_eq!(BCloseAnswer::from_slug("discard"), Some(BCloseAnswer::Discard));
        assert_eq!(BCloseAnswer::from_slug("later"), None);
    }
}
