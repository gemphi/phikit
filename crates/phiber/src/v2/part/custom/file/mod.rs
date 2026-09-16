//! File browser widget — shared by the `File` host and the modal
//! Open / Save As / Import / Export dialog.
//!
//! Split along Blender's own seams:
//!   `set`      — static tables (categories, columns, display modes, filters)
//!   `keys`     — every `ctx.file.*` state key, one function each
//!   `kind`     — `FILE_TYPE_*` classification by extension
//!   `listing`  — real `read_dir` + sort + filter + formatting
//!   `fsmenu`   — runtime volumes and system bookmarks
//!   `history`  — the back/forward folder stacks
//!   `purpose`  — what the browser was opened for (execute-button label)

pub mod fsmenu;
pub mod history;
pub mod item;
pub mod keys;
pub mod kind;
pub mod list;
pub mod listing;
pub mod purpose;
pub mod set;
pub mod table;

pub use item::BFileItem;
pub use kind::BFileKind;
pub use list::BFileList;
pub use listing::{BFileRow, BFileSort};
pub use purpose::BFilePurpose;
