//! The Play host's state, under `play.*`: the match being picked, written
//! as one line (`phiplay::pick::encode`), and the picker row on show.

use super::BKey;
use crate::v2::state::BPath;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlayKey {
    /// The choice: each row's slug and its card, `sport=0;home=4;…`.
    Choice,
    /// The row whose cards are shown, by slug.
    Row,
}

impl BKey for PlayKey {
    fn path(&self) -> BPath {
        let tail: &'static str = match self {
            Self::Choice => "choice",
            Self::Row => "row",
        };
        BPath::owned(vec!["play", tail])
    }
}
