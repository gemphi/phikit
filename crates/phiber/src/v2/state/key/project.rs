//! The Project Browser's state, under `project.*`: the folder picked in the
//! navigator, the asset picked in the grid, and how the grid is shown —
//! cards or a list, their size, their order, the filter.

use super::BKey;
use crate::v2::state::BPath;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ProjectKey {
    /// The navigator's folder: a kind's slug, or `all` (All Files).
    Folder,
    /// The asset picked, by its spelling in the project database.
    Selected,
    /// `true` shows a list rather than cards.
    List,
    /// The cards' edge, px.
    Size,
    /// `true` orders Z to A.
    Descending,
    /// The filter field's text.
    Search,
    /// A footwear design, by slug, confirmed against its sketch.
    Confirmed(&'static str),
}

impl BKey for ProjectKey {
    fn path(&self) -> BPath {
        if let Self::Confirmed(slug) = self {
            return BPath::owned(vec!["project", "confirmed", slug]);
        }
        let tail: &'static str = match self {
            Self::Confirmed(_) => unreachable!("answered above"),
            Self::Folder => "folder",
            Self::Selected => "selected",
            Self::List => "list",
            Self::Size => "size",
            Self::Descending => "descending",
            Self::Search => "search",
        };
        BPath::owned(vec!["project", tail])
    }
}
