//! Collection View Traits and Protocol
//!
//! this file defines the standard protocols for providing data and responding
//! to interaction events within a grid-based collection view.
//! mirrors Apple's modular pattern for data-driven UI components.

use crate::graphics::NSPoint;
use crate::view::NSView;

/// Protocol for providing items to a collection view.
///
/// handles item count and view creation for individual grid elements.
/// coordinates selection state updates directly on the provider.
pub trait NSCollectionViewDataSource: Send + Sync {
    /// Returns the total number of items to display in the grid.
    fn number_of_items(&self) -> usize;

    /// Creates or returns a configured view for the item at a given index.
    fn view_for_item(&self, index: usize) -> NSView;

    /// Notifies the source that an item has been selected by the user.
    fn collection_view_did_select_item(&mut self, _index: usize) {}
}

/// Protocol for responding to collection view interaction events.
///
/// handles high-level events like selection changes and double clicks.
/// provides support for starting drag-and-drop operations from grid items.
pub trait NSCollectionViewDelegate: Send + Sync {
    /// Notifies the delegate when the grid selection state changes.
    fn collection_view_selection_did_change(&mut self, selected_index: Option<usize>);

    /// Notifies the delegate that an item was double-clicked by the user.
    fn collection_view_item_did_double_click(&mut self, index: usize);

    /// Notifies the delegate that a drag operation should initiate.
    fn collection_view_did_start_drag(&mut self, _index: usize, _location: NSPoint) {}
}
