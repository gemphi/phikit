//! File Browser modal — the Open / Save As / Import / Export overlay.
//! Content is shared with the `file` host: sidebar categories, columns,
//! filters, and nav actions all live in `part::custom::file`.
//!
//! Region layout mirrors Blender's `space_file` (`space_file.cc:61`):
//!   ┌─ header   (View / Select menus) ──────────────────────────┐
//!   ├─ ui       (nav / path / search / display / filter) ───────┤
//!   ├─ tools    (bookmarks sidebar) │ window (file list) ───────┤
//!   └─ execute  (filename / Cancel / Open) ────────────────────-┘

pub mod execute;
pub mod files;
pub mod header;
pub mod path_bar;
pub mod sidebar;

pub use crate::v2::part::custom::file::set::{
    BFileCategory, BFileColumn, BFileDisplayType, BFileEntry, CATEGORIES, COLUMNS, DISPLAY_MODES,
    FILTERS, NAV, SYSTEM,
};
use crate::v2::state::BState;
use crate::v2::view::prim::BPanelRole;
use crate::v2::view::{BView, BViewItem};

#[derive(Clone, Copy, Debug, Default)]
pub struct BFileBrowserModal;

impl BViewItem for BFileBrowserModal {
    fn build(&self, s: &BState) -> BView {
        BView::column(vec![
            header::view(s),
            path_bar::view(s),
            // The middle band (bookmarks sidebar | file list) is wrapped in a
            // `Body` panel so it takes `flex_1 min_h_0` inside the modal card:
            // this gives the sidebar and file list a bounded height to scroll
            // within instead of clipping or overflowing the card.
            BView::panel(
                BPanelRole::Body,
                vec![BView::row(vec![sidebar::view(s), files::view(s)])],
            ),
            execute::view(s),
        ])
    }
}
