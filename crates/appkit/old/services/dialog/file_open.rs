//! NSOpenPanel — file open dialog.
//!
//! Apple equivalent: `NSOpenPanel`
//! Reference: https://developer.apple.com/documentation/appkit/nsopenpanel
//!
//! NSOpenPanel is a subclass of NSSavePanel that lets the user select
//! existing files and directories.

use super::DialogResult;

/// NSOpenPanel — presents a file-open dialog to the user.
///
/// Mirrors Apple's `NSOpenPanel` API.
pub struct NSOpenPanel {
    pub title: String,
    pub filters: Vec<(String, Vec<String>)>,
    pub initial_directory: Option<String>,
    pub allow_multiple: bool,
}

impl NSOpenPanel {
    /// Create a new file-open dialog with default settings.
    pub fn new() -> Self {
        Self {
            title: "Open File".to_string(),
            filters: Vec::new(),
            initial_directory: None,
            allow_multiple: false,
        }
    }

    /// Set the dialog title.
    pub fn with_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    /// Add a file type filter (e.g. "3D Files", &["usdz", "obj", "gltf"]).
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

    /// Allow selecting multiple files.
    pub fn with_multiple(mut self, allow: bool) -> Self {
        self.allow_multiple = allow;
        self
    }

    /// Show the dialog and return the result.
    #[cfg(target_os = "windows")]
    pub fn show(&self) -> DialogResult {
        println!("[stub] NSOpenPanel::show() — title: \"{}\"", self.title);
        // Stub: return a hardcoded path for now.
        DialogResult::Ok("C:\\Users\\user\\Documents\\scene.usdz".to_string())
    }

    /// Show the dialog and return the result (non-Windows stub).
    #[cfg(not(target_os = "windows"))]
    pub fn show(&self) -> DialogResult {
        println!("[stub] NSOpenPanel::show() — title: \"{}\"", self.title);
        DialogResult::Ok("/Users/user/Documents/scene.usdz".to_string())
    }
}

impl Default for NSOpenPanel {
    fn default() -> Self {
        Self::new()
    }
}
