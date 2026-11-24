//! What passes: what is so of the open room just now, which nothing
//! keeps, and a picture as the viewer is handed it.

use std::cell::{Cell, RefCell};

/// A picture that was posted: its key, who posted it, by their id and
/// by name, and when; and of one of the person's own files (the
/// library's), where it is on this disk.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Shot {
    pub key: String,
    pub who: String,
    pub by: String,
    pub when: String,
    pub file: Option<std::path::PathBuf>,
}

/// What is so of the open room just now. Another room begins without
/// any of it.
#[derive(Debug, Default)]
pub struct Now {
    /// The message the person is answering, by its id.
    pub answer: RefCell<String>,
    /// Whether the room is being searched, and whether its pinned
    /// messages are listed.
    pub finding: Cell<bool>,
    pub pins: Cell<bool>,
    /// The person whose profile stands beside the room, by their id.
    pub who: RefCell<Option<String>>,
    /// A message the stream is to bring into sight, by its id.
    pub seek: RefCell<Option<String>>,
    /// The pictures the viewer steps through.
    pub album: RefCell<Vec<Shot>>,
    /// The thing of a shop that has the page, by its id.
    pub product: RefCell<Option<String>>,
    /// The record of Phiegg's whose page is open (a world, a club, a show).
    pub record: RefCell<Option<crate::part::search::Found>>,
}
