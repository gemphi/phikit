//! # NSPathControl — File Path Selection Control
//!
//! Path control for displaying and selecting file system paths with breadcrumb navigation.
//!
//! ## Apple Documentation
//! - [NSPathControl](https://developer.apple.com/documentation/appkit/nspathcontrol)
//! - [NSPathCell](https://developer.apple.com/documentation/appkit/nspathcell)
//!
//! ## Usage
//!
//! ```rust,ignore
//! use std::path::PathBuf;
//!
//! let mut path_control = NSPathControl::new(frame);
//! path_control.set_path(PathBuf::from("/Users/username/Documents"));
//! path_control.set_path_style(NSPathStyle::Standard);
//! path_control.set_delegate(Box::new(MyPathControlDelegate));
//! ```
//!
//! ## Delegate Pattern
//!
//! ```rust,ignore
//! struct MyDelegate;
//!
//! impl NSPathControlDelegate for MyDelegate {
//!     fn path_did_change(&mut self, path: &Path) {
//!         println!("Path changed to: {}", path.display());
//!     }
//!     
//!     fn will_display_open_panel(&mut self, open_panel: &mut NSOpenPanel) {
//!         open_panel.set_can_choose_directories(true);
//!         open_panel.set_can_choose_files(false);
//!     }
//! }
//! ```
//!
//! ## See Also
//! - [NSPathControl](https://developer.apple.com/documentation/appkit/nspathcontrol)
//! - [NSControl](https://developer.apple.com/documentation/appkit/nscontrol)

pub mod delegate;

pub use delegate::NSPathControlDelegate;

use crate::control::NSControl;
use crate::graphics::NSRect;
use crate::view::NSView;
use delegate::{NSPathControlViewDelegate, PathControlState};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Path control visual style.
///
/// Apple: [NSPathStyle](https://developer.apple.com/documentation/appkit/nspathcontrol/style)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NSPathStyle {
    /// Standard path display with breadcrumb navigation.
    ///
    /// Apple: `standard`
    Standard,
    
    /// Navigation bar style (similar to Finder).
    ///
    /// Apple: `navigationBar`
    NavigationBar,
    
    /// Pop-up button style showing only the last component.
    ///
    /// Apple: `popUp`
    PopUp,
}

/// Path control component type.
///
/// Apple: [NSPathComponentCell](https://developer.apple.com/documentation/appkit/nspathcomponentcell)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NSPathComponent {
    /// Display name of the component.
    pub title: String,
    
    /// Full path up to and including this component.
    pub path: PathBuf,
    
    /// Whether this component represents a directory.
    pub is_directory: bool,
}

impl NSPathComponent {
    /// Create a new path component.
    pub fn new(title: String, path: PathBuf, is_directory: bool) -> Self {
        Self {
            title,
            path,
            is_directory,
        }
    }
}

/// Path control for file path selection.
///
/// Apple: [NSPathControl](https://developer.apple.com/documentation/appkit/nspathcontrol)
#[derive(Clone)]
pub struct NSPathControl {
    control: NSControl,
    state: Arc<Mutex<PathControlState>>,
    delegate: Arc<Mutex<Option<Box<dyn NSPathControlDelegate>>>>,
}

impl NSPathControl {
    /// Create a new path control.
    ///
    /// Apple: `[[NSPathControl alloc] initWithFrame:]`
    pub fn new(frame: NSRect) -> Self {
        let state = Arc::new(Mutex::new(PathControlState::default()));
        let delegate = Arc::new(Mutex::new(None));
        
        let mut control = NSControl::new(frame);
        control.view_mut().set_delegate(NSPathControlViewDelegate {
            state: state.clone(),
            delegate: delegate.clone(),
        });
        
        Self {
            control,
            state,
            delegate,
        }
    }

    /// Create a path control with default styling.
    pub fn with_path(path: PathBuf) -> Self {
        let mut control = Self::new(NSRect::sized(300.0, 28.0));
        control.set_path(path);
        control
    }

    // ── Path Management ──────────────────────────────────────────────

    /// Set the current path.
    ///
    /// Apple: `setURL:`
    pub fn set_path(&mut self, path: PathBuf) {
        let mut state = self.state.lock().unwrap();
        state.path = path.clone();
        state.components = Self::path_to_components(&path);
    }

    /// Get the current path.
    ///
    /// Apple: `URL`
    pub fn path(&self) -> PathBuf {
        self.state.lock().unwrap().path.clone()
    }

    /// Get the path components for breadcrumb display.
    ///
    /// Apple: `pathComponentCells`
    pub fn path_components(&self) -> Vec<NSPathComponent> {
        self.state.lock().unwrap().components.clone()
    }

    // ── Configuration ────────────────────────────────────────────────

    /// Set the path control style.
    ///
    /// Apple: `setPathStyle:`
    pub fn set_path_style(&mut self, style: NSPathStyle) {
        self.state.lock().unwrap().style = style;
    }

    /// Get the path control style.
    ///
    /// Apple: `pathStyle`
    pub fn path_style(&self) -> NSPathStyle {
        self.state.lock().unwrap().style
    }

    /// Set whether the control is editable (allows clicking to change path).
    ///
    /// Apple: `setEditable:`
    pub fn set_editable(&mut self, editable: bool) {
        self.state.lock().unwrap().is_editable = editable;
    }

    /// Get whether the control is editable.
    ///
    /// Apple: `isEditable`
    pub fn is_editable(&self) -> bool {
        self.state.lock().unwrap().is_editable
    }

    /// Set the background color.
    ///
    /// Apple: `setBackgroundColor:`
    pub fn set_background_color(&mut self, color: crate::graphics::NSColor) {
        self.state.lock().unwrap().background_color = Some(color);
    }

    /// Get the background color.
    ///
    /// Apple: `backgroundColor`
    pub fn background_color(&self) -> Option<crate::graphics::NSColor> {
        self.state.lock().unwrap().background_color
    }

    /// Set placeholder text shown when path is empty.
    ///
    /// Apple: `setPlaceholderString:`
    pub fn set_placeholder(&mut self, placeholder: String) {
        self.state.lock().unwrap().placeholder = Some(placeholder);
    }

    /// Get placeholder text.
    ///
    /// Apple: `placeholderString`
    pub fn placeholder(&self) -> Option<String> {
        self.state.lock().unwrap().placeholder.clone()
    }

    // ── Delegate ─────────────────────────────────────────────────────

    /// Set the delegate for path control events.
    ///
    /// Apple: `setDelegate:`
    pub fn set_delegate(&mut self, delegate: Box<dyn NSPathControlDelegate>) {
        *self.delegate.lock().unwrap() = Some(delegate);
    }

    // ── View Access ──────────────────────────────────────────────────

    pub fn view(&self) -> NSView {
        self.control.view().clone()
    }

    pub fn view_mut(&mut self) -> &mut NSView {
        self.control.view_mut()
    }

    pub fn view_clone(&self) -> NSView {
        self.control.view().clone()
    }

    // ── Internal Helpers ─────────────────────────────────────────────

    /// Convert a path to path components for breadcrumb display.
    fn path_to_components(path: &Path) -> Vec<NSPathComponent> {
        let mut components = Vec::new();
        let mut current_path = PathBuf::new();
        
        for component in path.components() {
            use std::path::Component;
            
            match component {
                Component::RootDir => {
                    current_path.push("/");
                    components.push(NSPathComponent::new(
                        "/".to_string(),
                        current_path.clone(),
                        true,
                    ));
                }
                Component::Normal(name) => {
                    current_path.push(name);
                    let title = name.to_string_lossy().to_string();
                    let is_directory = current_path.is_dir();
                    components.push(NSPathComponent::new(
                        title,
                        current_path.clone(),
                        is_directory,
                    ));
                }
                Component::Prefix(prefix) => {
                    // Windows drive letters (C:, D:, etc.)
                    let prefix_str = prefix.as_os_str().to_string_lossy().to_string();
                    current_path.push(prefix_str.clone());
                    components.push(NSPathComponent::new(
                        prefix_str,
                        current_path.clone(),
                        true,
                    ));
                }
                Component::CurDir | Component::ParentDir => {
                    // Skip . and .. in display
                }
            }
        }
        
        components
    }
}
