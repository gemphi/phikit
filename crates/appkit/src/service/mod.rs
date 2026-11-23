//! What a part asks for that is not drawn: a dialog, a project, a command, a
//! key map, a session. So far: a table of names read from a file, an
//! app's assets as folders on disk, the shared assets built into the
//! program, and the folder an app keeps a person's own files in.

mod bundle;
mod call;
mod folder;
mod store;
mod table;

pub use bundle::PBundle;
pub use call::{PCall, PStream, STOPPED};
pub use folder::PFolder;
pub use store::PStore;
pub use table::PTable;
