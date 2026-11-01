//! # NSToolbarItem
//!
//! Represents an individual item within a toolbar.

use crate::view::NSView;

/// Individual toolbar item
/// Mirrors: NSToolbarItem
#[derive(Clone)]
pub struct NSToolbarItem {
    pub(crate) item_identifier: String,
    pub(crate) label: String,
    pub(crate) palette_label: String,
    pub(crate) tool_tip: Option<String>,
    pub(crate) view: Option<NSView>,
    pub(crate) enabled: bool,
}

impl NSToolbarItem {
    /// Create a new toolbar item with identifier
    /// Mirrors: NSToolbarItem.init(itemIdentifier:)
    pub fn new(item_identifier: &str) -> Self {
        Self {
            item_identifier: item_identifier.to_string(),
            label: String::new(),
            palette_label: String::new(),
            tool_tip: None,
            view: None,
            enabled: true,
        }
    }

    /// Create a button toolbar item
    pub fn button(item_identifier: &str, label: &str, view: NSView) -> Self {
        let mut item = Self::new(item_identifier);
        item.set_label(label);
        item.set_view(view);
        item
    }

    /// Create a segmented control toolbar item
    pub fn segmented_control(item_identifier: &str, view: NSView) -> Self {
        let mut item = Self::new(item_identifier);
        item.set_view(view);
        item
    }

    /// Create a search field toolbar item
    pub fn search_field(item_identifier: &str, view: NSView) -> Self {
        let mut item = Self::new(item_identifier);
        item.set_view(view);
        item
    }

    /// Set the label shown in the toolbar
    /// Mirrors: NSToolbarItem.label
    pub fn set_label(&mut self, label: &str) {
        self.label = label.to_string();
    }

    /// Get the label
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Set the palette label (shown in customization palette)
    /// Mirrors: NSToolbarItem.paletteLabel
    pub fn set_palette_label(&mut self, label: &str) {
        self.palette_label = label.to_string();
    }

    /// Set tooltip
    /// Mirrors: NSToolbarItem.toolTip
    pub fn set_tool_tip(&mut self, tip: &str) {
        self.tool_tip = Some(tip.to_string());
    }

    /// Set the view for this item
    /// Mirrors: NSToolbarItem.view
    pub fn set_view(&mut self, view: NSView) {
        self.view = Some(view);
    }

    /// Get the view
    pub fn view(&self) -> Option<&NSView> {
        self.view.as_ref()
    }

    /// Set enabled state
    /// Mirrors: NSToolbarItem.isEnabled
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get item identifier
    pub fn item_identifier(&self) -> &str {
        &self.item_identifier
    }
}
