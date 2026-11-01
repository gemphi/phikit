//! Dialog primitives for file open/save, confirmation, and progress.

mod confirm;
mod file_open;
mod file_save;
mod progress;

pub use confirm::{ConfirmDialog, ConfirmResult};
pub use file_open::NSOpenPanel;
pub use file_save::FileSaveDialog;
pub use progress::ProgressDialog;

/// Result of a dialog interaction.
#[derive(Debug, Clone, PartialEq)]
pub enum DialogResult {
    /// Successful outcome — contains a file path or confirmed value.
    Ok(String),
    /// User cancelled the dialog.
    Cancel,
    /// An error occurred.
    Error(String),
}
