//! # NSToolbar — Application Toolbar
//!
//! A bar that provides access to frequently used commands and features.
//!
//! ## Apple Documentation
//! - [NSToolbar](https://developer.apple.com/documentation/appkit/nstoolbar)
//! - [NSToolbarItem](https://developer.apple.com/documentation/appkit/nstoolbaritem)
//! - [NSToolbarDelegate](https://developer.apple.com/documentation/appkit/nstoolbardelegate)
//!
//! ## Usage
//!
//! ```rust,ignore
//! let mut toolbar = NSToolbar::new("MainToolbar");
//! toolbar.set_display_mode(DisplayMode::IconAndLabel);
//! toolbar.add_item(NSToolbarItem::new("save"));
//! toolbar.add_flexible_space();
//! toolbar.add_item(NSToolbarItem::new("search"));
//! ```
//!
//! ## See Also
//! - [NSSegmentedControl](https://developer.apple.com/documentation/appkit/nssegmentedcontrol)
//! - [NSButton](https://developer.apple.com/documentation/appkit/nsbutton)

pub mod delegate;
pub mod drawing;
pub mod item;

pub use delegate::NSToolbarDelegate;
pub use item::NSToolbarItem;

use crate::theme;

/// Identifier for a toolbar
pub type NSToolbarIdentifier = String;

/// Display mode for toolbar items
/// Mirrors: NSToolbar.DisplayMode
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayMode {
    IconOnly = 0,
    LabelOnly = 1,
    IconAndLabel = 2,
}

/// Size mode for toolbar items
/// Mirrors: NSToolbar.SizeMode
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SizeMode {
    Default = 0,
    Regular = 1,
    Small = 2,
}

/// Main toolbar class
/// Mirrors: NSToolbar
pub struct NSToolbar {
    pub(crate) identifier: NSToolbarIdentifier,
    pub(crate) display_mode: DisplayMode,
    pub(crate) size_mode: SizeMode,
    pub(crate) items: Vec<NSToolbarItem>,
    pub(crate) allows_user_customization: bool,
    pub(crate) centered_item_identifier: Option<String>,
    pub(crate) delegate: Option<Box<dyn NSToolbarDelegate>>,
    pub(crate) visible: bool,
    pub(crate) autosaves_configuration: bool,
}

impl NSToolbar {
    /// Initialize a new toolbar with identifier
    /// Mirrors: NSToolbar.init(identifier:)
    pub fn new(identifier: &str) -> Self {
        Self {
            identifier: identifier.to_string(),
            display_mode: DisplayMode::IconAndLabel,
            size_mode: SizeMode::Default,
            items: Vec::new(),
            allows_user_customization: true,
            centered_item_identifier: None,
            delegate: None,
            visible: true,
            autosaves_configuration: false,
        }
    }

    /// Add a standard item to the toolbar
    pub fn add_item(&mut self, item: NSToolbarItem) {
        self.items.push(item);
    }

    /// Add a flexible space divider to separate item groups
    pub fn add_flexible_space(&mut self) {
        let item = NSToolbarItem::new("NSToolbarFlexibleSpaceItem");
        self.add_item(item);
    }

    /// Get toolbar identifier
    pub fn identifier(&self) -> &NSToolbarIdentifier {
        &self.identifier
    }

    /// Set display mode
    pub fn set_display_mode(&mut self, mode: DisplayMode) {
        self.display_mode = mode;
    }

    /// Get display mode
    pub fn display_mode(&self) -> DisplayMode {
        self.display_mode
    }

    /// Set size mode
    pub fn set_size_mode(&mut self, mode: SizeMode) {
        self.size_mode = mode;
    }

    /// Get size mode
    pub fn size_mode(&self) -> SizeMode {
        self.size_mode
    }

    /// Insert item at index
    pub fn insert_item(&mut self, item: NSToolbarItem, index: usize) {
        let idx = index.min(self.items.len());
        self.items.insert(idx, item);
    }

    /// Remove item at index
    pub fn remove_item(&mut self, index: usize) {
        if index < self.items.len() {
            self.items.remove(index);
        }
    }

    /// Get all items
    pub fn items(&self) -> &[NSToolbarItem] {
        &self.items
    }

    /// Get height for toolbar
    pub fn height(&self) -> f32 {
        theme::f32_or(theme::keys::SIZE_TITLE_BAR_HEIGHT, 44.0)
    }

    /// Set visibility
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Check visibility
    pub fn is_visible(&self) -> bool {
        self.visible
    }
    
    /// Get view accessor (for EditorPanel trait)
    pub fn view(&self) -> &NSView {
        // NSToolbar needs to expose its underlying view
        // This is a placeholder - actual implementation depends on NSToolbar internals
        unimplemented!("NSToolbar needs view accessor")
    }
    
    pub fn view_mut(&mut self) -> &mut NSView {
        unimplemented!("NSToolbar needs view_mut accessor")
    }
}

/// Convenience factory methods
impl NSToolbar {
    /// Create a simple icon-only toolbar.
    pub fn icon_toolbar(identifier: &str) -> Self {
        let mut toolbar = Self::new(identifier);
        toolbar.set_display_mode(DisplayMode::IconOnly);
        toolbar
    }
}

use crate::graphics::NSRect;
use crate::view::NSView;
