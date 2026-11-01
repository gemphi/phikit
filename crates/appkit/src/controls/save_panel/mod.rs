//! # NSSavePanel — File Save Dialog
//!
//! Native file save dialog for selecting where to save files.
//!
//! ## Apple Documentation
//! - [NSSavePanel](https://developer.apple.com/documentation/appkit/nssavepanel)
//!
//! ## Usage
//!
//! ```rust,ignore
//! let mut panel = NSSavePanel::new();
//! panel.set_name_field_string_value("Untitled.txt");
//! panel.set_allowed_file_types(vec!["txt".to_string(), "md".to_string()]);
//! 
//! if let Some(path) = panel.run_modal() {
//!     println!("User selected: {:?}", path);
//!     // Save file to path
//! }
//! ```
//!
//! ## See Also
//! - [NSSavePanel](https://developer.apple.com/documentation/appkit/nssavepanel)
//! - [NSOpenPanel](https://developer.apple.com/documentation/appkit/nsopenpanel)

use std::path::PathBuf;

/// **NSSavePanel** — A modal file save dialog
///
/// Apple: [NSSavePanel](https://developer.apple.com/documentation/appkit/nssavepanel)
pub struct NSSavePanel {
    name_field_string_value: String,
    allowed_file_types: Vec<String>,
}

impl NSSavePanel {
    /// Create a new save panel.
    ///
    /// Apple: `[[NSSavePanel alloc] init]`
    pub fn new() -> Self {
        Self {
            name_field_string_value: String::new(),
            allowed_file_types: Vec::new(),
        }
    }

    /// Set the default filename shown in the name field.
    ///
    /// Apple: `setNameFieldStringValue:`
    pub fn set_name_field_string_value(&mut self, name: &str) {
        self.name_field_string_value = name.to_string();
    }

    /// Set the allowed file types (extensions).
    ///
    /// Apple: `setAllowedFileTypes:`
    pub fn set_allowed_file_types(&mut self, types: Vec<String>) {
        self.allowed_file_types = types;
    }

    /// Display the save panel modally and return the selected path.
    ///
    /// Apple: `runModal` returns NSModalResponse, then use `url` property
    ///
    /// Returns `Some(PathBuf)` if the user selected a location, `None` if cancelled.
    pub fn run_modal(&self) -> Option<PathBuf> {
        // In a full implementation, this would show a native file dialog.
        // For now, this is a stub that logs the configuration.
        println!("[NSSavePanel] Running Modal Save Panel:");
        println!("  Default name: {}", self.name_field_string_value);
        if !self.allowed_file_types.is_empty() {
            println!("  Allowed types: {:?}", self.allowed_file_types);
        }

        // Stub: return None (user cancelled)
        // In a real implementation, this would use native Windows file dialog APIs
        None
    }
}

impl Default for NSSavePanel {
    fn default() -> Self {
        Self::new()
    }
}
