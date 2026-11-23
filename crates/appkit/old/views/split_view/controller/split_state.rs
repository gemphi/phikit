//! Split view controller struct definition, constructors, and basic state accessors.

use crate::views::split_view::{NSSplitView, NSSplitViewItem};
use crate::{NSAutoresizingMask, NSRect, NSView};

/// Manages a split view with multiple panes.
///
/// mirrors Apple's [`NSSplitViewController`](https://developer.apple.com/documentation/appkit/nssplitviewcontroller)
/// handling a collection of split view items and coordinating their layout.
/// it automatically manages dividers, collapse states, and resizing behavior.
pub struct NSSplitViewController {
    pub(crate) split_view: NSSplitView,
    pub(crate) items: Vec<NSSplitViewItem>,
    pub(crate) is_vertical: bool,
}

impl NSSplitViewController {
    /// Create a vertical split controller (side-by-side panes).
    /// Arranges its items from left to right along the horizontal axis.
    pub fn vertical() -> Self {
        let mut split_view = NSSplitView::vertical();
        split_view.view_mut().set_autoresizing_mask(
            NSAutoresizingMask::WIDTH_SIZABLE | NSAutoresizingMask::HEIGHT_SIZABLE,
        );
        Self {
            split_view,
            items: Vec::new(),
            is_vertical: true,
        }
    }

    /// Create a horizontal split controller (stacked panes).
    /// Arranges its items from top to bottom along the vertical axis.
    pub fn horizontal() -> Self {
        let mut split_view = NSSplitView::horizontal();
        split_view.view_mut().set_autoresizing_mask(
            NSAutoresizingMask::WIDTH_SIZABLE | NSAutoresizingMask::HEIGHT_SIZABLE,
        );
        Self {
            split_view,
            items: Vec::new(),
            is_vertical: false,
        }
    }

    /// Add a split view item. The controller manages its position.
    /// New items are appended to the existing list and layout is refreshed.
    pub fn add_split_view_item(&mut self, item: NSSplitViewItem) {
        if !item.is_collapsed {
            self.split_view.add_arranged_subview(item.view.clone());
        }
        self.items.push(item);
        self.update_dividers();
    }

    /// Toggle collapse state of an item by index.
    /// If an item is collapsed, it is removed from the view hierarchy.
    pub fn toggle_item(&mut self, index: usize) {
        if let Some(item) = self.items.get_mut(index) {
            item.is_collapsed = !item.is_collapsed;
            self.rebuild_split_view();
        }
    }

    /// Collapse an item by index.
    /// The item's space is reclaimed by its neighbors in the split view.
    pub fn collapse_item(&mut self, index: usize) {
        if let Some(item) = self.items.get_mut(index) {
            item.is_collapsed = true;
            self.rebuild_split_view();
        }
    }

    /// Expand an item by index.
    /// Restores the item to its preferred or minimum size in the layout.
    pub fn expand_item(&mut self, index: usize) {
        if let Some(item) = self.items.get_mut(index) {
            item.is_collapsed = false;
            self.rebuild_split_view();
        }
    }

    /// Set the frame — typically called by the window's layout_chrome.
    /// Triggers a full layout pass and recalculates divider positions.
    pub fn set_frame(&mut self, frame: NSRect) {
        self.split_view.view_mut().set_frame(frame);
        self.update_dividers();
    }

    /// Get the underlying view for adding to a window.
    /// Provides access to the NSSplitView instance managed by this controller.
    pub fn view(&self) -> &NSView {
        self.split_view.view()
    }

    /// Get mutable view.
    /// Allows direct modification of the underlying split view instance.
    pub fn view_mut(&mut self) -> &mut NSView {
        self.split_view.view_mut()
    }

    /// Get a view clone for adding to parent.
    /// Returns a thread-safe clone of the underlying view's handle.
    pub fn view_clone(&self) -> NSView {
        self.split_view.view().clone()
    }

    /// Force layout pass.
    /// Triggers an immediate recalculation of all subview frames.
    pub fn layout(&mut self) {
        self.split_view.view_mut().layout_subviews();
    }

    /// Number of items.
    /// Returns the total count of managed split view items (collapsed or not).
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    /// Get item at index.
    /// Returns a reference to the item if the index is valid.
    pub fn item_at(&self, index: usize) -> Option<&NSSplitViewItem> {
        self.items.get(index)
    }

    /// Get mutable item at index.
    /// Allows modification of an item's constraints or collapse state.
    pub fn item_at_mut(&mut self, index: usize) -> Option<&mut NSSplitViewItem> {
        self.items.get_mut(index)
    }

    /// Present the split view controller by making it visible.
    /// In a full implementation this would add the window to the application.
    pub fn present(&self) {
        println!("[NSSplitViewController] Presenting with {} items", self.items.len());
    }
}
