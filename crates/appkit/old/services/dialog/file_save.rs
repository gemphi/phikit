//! File save dialog.

use super::DialogResult;

/// Configuration for a file-save dialog.
pub struct FileSaveDialog {
    pub title: String,
    pub default_name: String,
    pub filters: Vec<(String, Vec<String>)>,
    pub initial_directory: Option<String>,
}

impl FileSaveDialog {
    /// Create a new file-save dialog with default settings.
    pub fn new() -> Self {
        Self {
            title: "Save File".to_string(),
            default_name: "Untitled".to_string(),
            filters: Vec::new(),
            initial_directory: None,
        }
    }

    /// Set the dialog title.
    pub fn with_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    /// Set the default file name.
    pub fn with_default_name(mut self, name: &str) -> Self {
        self.default_name = name.to_string();
        self
    }

    /// Add a file type filter.
    pub fn with_filter(mut self, name: &str, extensions: &[&str]) -> Self {
        self.filters.push((
            name.to_string(),
            extensions.iter().map(|s| s.to_string()).collect(),
        ));
        self
    }

    /// Set the initial directory.
    pub fn with_directory(mut self, path: &str) -> Self {
        self.initial_directory = Some(path.to_string());
        self
    }

    /// Show the dialog and return the result.
    #[cfg(target_os = "windows")]
    pub fn show(&self) -> DialogResult {
        println!(
            "[stub] FileSaveDialog::show() — title: \"{}\", default: \"{}\"",
            self.title, self.default_name
        );
        DialogResult::Ok(format!("C:\\Users\\user\\Documents\\{}", self.default_name))
    }

    /// Show the dialog and return the result (non-Windows stub).
    #[cfg(not(target_os = "windows"))]
    pub fn show(&self) -> DialogResult {
        println!(
            "[stub] FileSaveDialog::show() — title: \"{}\", default: \"{}\"",
            self.title, self.default_name
        );
        DialogResult::Ok(format!("/Users/user/Documents/{}", self.default_name))
    }
}

impl Default for FileSaveDialog {
    fn default() -> Self {
        Self::new()
    }
}
