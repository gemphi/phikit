//! # NSToolbarDelegate
//!
//! Protocol defining the interface for a toolbar's delegate.

use super::item::NSToolbarItem;

/// Toolbar delegate protocol
/// Mirrors: NSToolbarDelegate
pub trait NSToolbarDelegate: Send + Sync {
    /// Return items for a given item identifier array
    fn toolbar_item_for_item_identifier(&self, item_identifier: &str) -> Option<NSToolbarItem>;

    /// Return default item identifiers
    fn toolbar_default_item_identifiers(&self) -> Vec<String>;

    /// Return allowed item identifiers (for customization)
    fn toolbar_allowed_item_identifiers(&self) -> Vec<String>;

    /// Return selectable item identifiers (optional)
    fn toolbar_selectable_item_identifiers(&self) -> Vec<String> {
        Vec::new()
    }
}
