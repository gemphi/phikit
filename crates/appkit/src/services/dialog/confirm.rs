//! Confirmation dialog for yes/no/cancel prompts.

/// Result of a confirmation dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmResult {
    Yes,
    No,
    Cancel,
}

/// A confirmation dialog with customizable buttons.
pub struct ConfirmDialog {
    pub title: String,
    pub message: String,
    pub buttons: Vec<String>,
}

impl ConfirmDialog {
    /// Create a new confirmation dialog.
    pub fn new(title: &str, message: &str) -> Self {
        Self {
            title: title.to_string(),
            message: message.to_string(),
            buttons: vec!["Yes".to_string(), "No".to_string(), "Cancel".to_string()],
        }
    }

    /// Set custom button labels (e.g. ["Save", "Don't Save", "Cancel"]).
    pub fn with_buttons(mut self, buttons: &[&str]) -> Self {
        self.buttons = buttons.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Show the dialog and return the user's choice.
    pub fn show(&self) -> ConfirmResult {
        println!(
            "[stub] ConfirmDialog::show() — title: \"{}\", message: \"{}\", buttons: {:?}",
            self.title, self.message, self.buttons
        );
        // Stub: default to Yes
        ConfirmResult::Yes
    }
}
