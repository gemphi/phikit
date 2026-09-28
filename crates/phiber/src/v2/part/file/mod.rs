//! File browser widget — shared by the `File` host and the modal
//! Open / Save As / Import / Export dialog.
//!
//! Split along Blender's own seams:
//!   `set`      — static tables (categories, columns, display modes, filters)
//!   keys — `state::key::FileKey`: every `ctx.file.*` key, one variant each
//!   `kind`     — `FILE_TYPE_*` classification by extension
//!   `listing`  — real `read_dir` + sort + filter + formatting
//!   `fsmenu`   — runtime volumes and system bookmarks
//!   `history`  — the back/forward folder stacks
//!   `purpose`  — what the browser was opened for (execute-button label)
//!   `header`   — View / Select (+ Library / Catalog) menus   ┐ shared by the
//!   `path_bar` — nav · directory · search · display · filter │ modal dialog
//!   `sidebar`  — Volumes / System / Bookmarks / Recent panels│ and the hosts
//!   `execute`  — filename + Cancel / <Execute>               ┘

pub mod asset_sidebar;
pub mod execute;
pub mod fsmenu;
pub mod header;
pub mod history;
pub mod kind;
pub mod listing;
pub mod nav;
pub mod op;
pub mod option;
pub mod path_bar;
pub mod preview;
pub mod purpose;
pub mod set;
pub mod sidebar;
pub mod table;
pub mod thumb;

use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
use crate::v2::view::BViewList;
pub use self::kind::BFileKind;
pub use self::listing::BFileRow;
pub use self::listing::BFileSort;
pub use self::purpose::BFilePurpose;

// `BFileItem` — the File / Asset host's window region: the file table (or
// the thumbnail grid), and in the Asset host the asset grid.
// 
// Everything else the browser shows is a region of the host, declared in
// `host::file` / `host::asset` the way Blender's `space_file` declares
// them: the header's menus (`part::menubar`), the path strip
// (`BPartType::PathBar`), the source list (`BPartType::SourceList`), the
// options sidebar and the execute bar (`BPartType::Execute`). One
// declaration serves the editor in a workspace and the Open / Save As
// dialog, which raises the same host in a card.
#[derive(Clone, Copy, Debug)]
pub struct BFileItem {
    pub host: BHostType,
}

impl BFileItem {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BFileItem {
    fn build(&self, s: &BState) -> BView {
        self::table::view(s)
    }
}

// `BFileList` — the File host's single body item.

#[derive(Clone, Copy, Debug)]
pub struct BFileList {
    pub host: BHostType,
}

impl BFileList {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewList for BFileList {
    fn items(&self, s: &BState) -> Vec<BView> {
        vec![BFileItem::new(self.host).build(s)]
    }
}
