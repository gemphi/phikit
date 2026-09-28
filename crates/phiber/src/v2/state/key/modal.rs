//! The dialogs' keys — which modal is up and how it is shown, and the
//! reader's document, under `ctx.modal.*` and `ctx.reader.*`.

use crate::v2::state::BPath;
use super::BKey;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ModalKey {
    /// The open modal's slug, or empty for none.
    Open,
    Maximized,
    Minimized,
    /// The document the reader shows (`modal::reader::store`).
    ReaderDoc,
    /// The modal the reader covered, restored when it closes.
    ReaderReturn,
    /// What the "Save changes before closing?" dialog goes on to do once
    /// answered — `quit`, `new_file:<spec>`, `open`, `open:<path>`
    /// (`wm_close_file_dialog`'s `post_action`).
    CloseThen,
}

impl BKey for ModalKey {
    fn path(&self) -> BPath {
        match self {
            Self::Open => BPath::owned(vec!["modal"]),
            Self::Maximized => BPath::owned(vec!["modal", "maximized"]),
            Self::Minimized => BPath::owned(vec!["modal", "minimized"]),
            Self::ReaderDoc => BPath::owned(vec!["reader", "doc"]),
            Self::ReaderReturn => BPath::owned(vec!["reader", "return"]),
            Self::CloseThen => BPath::owned(vec!["modal", "file_close", "then"]),
        }
    }
}
