//! The file browser's keys — Blender's `FileSelectParams`, one variant per
//! field, all under `ctx.file.*`.

use super::BKey;
use crate::v2::state::BPath;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FileKey {
    /// Current directory (`FileSelectParams::dir`).
    Path,
    /// Absolute path of the highlighted row (`FileSelectParams::file`,
    /// resolved). Always a full path — the importer opens exactly this.
    Selected,
    /// Text in the file name field.
    Filename,
    /// Search glob (`FileSelectParams::filter_search`).
    Search,
    /// Display type (`eFileDisplayType`).
    Display,
    /// Sort column (`eFileSortType`).
    Sort,
    /// `FILE_SORT_INVERT`.
    SortInvert,
    /// `FILE_HIDE_DOT`, inverted — this key is "show hidden".
    ShowHidden,
    /// Master filter toggle (`FILE_FILTER`).
    UseFilter,
    /// One per-type filter flag, e.g. `ctx.file.filter_blender`.
    Filter(&'static str),
    /// Pending navigation operator (`file.previous` / `next` / `parent` /
    /// `refresh`), consumed by the reducer.
    Nav,
    /// Back stack, newline-separated, most recent last.
    HistBack,
    /// Forward stack, newline-separated, most recent last.
    HistFwd,
    /// What the browser was opened for — drives the execute button's label.
    Purpose,
    /// Last enumeration error, if any.
    Error,
    /// Whether a source-list section is unfolded: `ctx.file.sidebar.<label>.expanded`.
    SidebarExpanded(&'static str),
    /// One bookmark slot: `ctx.file.bookmark.<n>`.
    Bookmark(&'static str),
    /// The `+` beside the bookmarks: an intent the reducer consumes.
    BookmarkAdd,
    /// Open/closed state of the type-filter popover (`FILEBROWSER_PT_filter`).
    FilterOpen,
    /// `WM_OT_open_mainfile.load_ui` — default on, stored as `"false"` when off.
    LoadUi,
    /// The welcome window's Recover Last Session: an intent the app consumes.
    RecoverLastSession,
}

impl BKey for FileKey {
    fn path(&self) -> BPath {
        let tail: &'static str = match self {
            Self::Path => "path",
            Self::Selected => "selected",
            Self::Filename => "filename",
            Self::Search => "search",
            Self::Display => "display",
            Self::Sort => "sort",
            Self::SortInvert => "sort_invert",
            Self::ShowHidden => "show_hidden",
            Self::UseFilter => "use_filter",
            Self::Filter(flag) => flag,
            Self::Nav => "nav",
            Self::HistBack => "hist_back",
            Self::HistFwd => "hist_fwd",
            Self::Purpose => "purpose",
            Self::Error => "error",
            Self::SidebarExpanded(label) => {
                return BPath::owned(vec!["file", "sidebar", label, "expanded"]);
            }
            Self::Bookmark(slot) => return BPath::owned(vec!["file", "bookmark", slot]),
            Self::BookmarkAdd => "bookmark_add",
            Self::FilterOpen => "filter_open",
            Self::LoadUi => "load_ui",
            Self::RecoverLastSession => "recover_last_session",
        };
        BPath::owned(vec!["file", tail])
    }
}
