//! # NSSplitViewController
//!
//! Apple's [`NSSplitViewController`](https://developer.apple.com/documentation/appkit/nssplitviewcontroller)
//! manages a split view with multiple items. Each item is an `NSSplitViewItem`
//! that wraps a view controller (or in our case, a view).
//!
//! The controller handles:
//! - Adding/removing split view items
//! - Collapsing/expanding sidebars
//! - Minimum/maximum widths per item
//! - Automatic gap/divider management (delegated to NSSplitView)
//!
//! ## Usage (from rust-kit-pro)
//! ```rust,ignore
//! let mut split = NSSplitViewController::new();
//! split.add_split_view_item(NSSplitViewItem::sidebar(hierarchy_view));
//! split.add_split_view_item(NSSplitViewItem::content(viewport_view));
//! split.add_split_view_item(NSSplitViewItem::inspector(inspector_view));
//! window.set_content_view_controller(split);
//! ```

use super::{NSSplitView, NSSplitViewItem, NSSplitViewItemBehavior};
use crate::panels::EditorPanel;
use crate::workspace::WorkspacePreset;
use crate::{NSAutoresizingMask, NSRect, NSView};

/// Manages a split view with multiple panes.
///
/// mirrors Apple's [`NSSplitViewController`](https://developer.apple.com/documentation/appkit/nssplitviewcontroller)
/// handling a collection of split view items and coordinating their layout.
/// it automatically manages dividers, collapse states, and resizing behavior.
pub struct NSSplitViewController {
    split_view: NSSplitView,
    items: Vec<NSSplitViewItem>,
    is_vertical: bool,
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

    // ── Private ─────────────────────────────────────────────────────

    /// Create a split view controller configured from a workspace preset.
    /// Builds the initial item list from the preset's panel slots.
    pub fn from_preset(preset: &WorkspacePreset) -> Self {
        let mut ctrl = Self::vertical();
        for slot in &preset.slots {
            if slot.visible {
                let behavior = match slot.dock {
                    crate::workspace::DockPosition::Left => NSSplitViewItemBehavior::Sidebar,
                    crate::workspace::DockPosition::Right => NSSplitViewItemBehavior::Inspector,
                    _ => NSSplitViewItemBehavior::Content,
                };
                let view = NSView::new(NSRect::sized(slot.preferred_size, 400.0))
                    .with_name(slot.id);
                let mut item = NSSplitViewItem::new(view);
                item.behavior = behavior;
                item.preferred_thickness = slot.preferred_size;
                item.minimum_thickness = slot.min_size;
                item.maximum_thickness = slot.max_size;
                item.can_collapse = slot.collapsible;
                ctrl.add_split_view_item(item);
            }
        }
        ctrl
    }

    /// Add an EditorPanel to the split view. Wraps the panel's view as a content item.
    pub fn add(&mut self, panel: &dyn EditorPanel) {
        let view = panel.view().clone();
        let item = NSSplitViewItem::content(view);
        self.add_split_view_item(item);
    }

    /// Apply a new preset, rebuilding the layout from scratch.
    pub fn apply_preset(&mut self, preset: &WorkspacePreset) {
        self.items.clear();
        self.split_view.view_mut().remove_all_subviews();
        for slot in &preset.slots {
            if slot.visible {
                let behavior = match slot.dock {
                    crate::workspace::DockPosition::Left => NSSplitViewItemBehavior::Sidebar,
                    crate::workspace::DockPosition::Right => NSSplitViewItemBehavior::Inspector,
                    _ => NSSplitViewItemBehavior::Content,
                };
                let view = NSView::new(NSRect::sized(slot.preferred_size, 400.0))
                    .with_name(slot.id);
                let mut item = NSSplitViewItem::new(view);
                item.behavior = behavior;
                item.preferred_thickness = slot.preferred_size;
                item.minimum_thickness = slot.min_size;
                item.maximum_thickness = slot.max_size;
                item.can_collapse = slot.collapsible;
                self.add_split_view_item(item);
            }
        }
    }

    /// Present the split view controller by making it visible.
    /// In a full implementation this would add the window to the application.
    pub fn present(&self) {
        println!("[NSSplitViewController] Presenting with {} items", self.items.len());
    }

    /// Rebuild the split view from items (after collapse/expand).
    fn rebuild_split_view(&mut self) {
        self.split_view.view_mut().remove_all_subviews();
        for item in &self.items {
            if !item.is_collapsed {
                self.split_view.add_arranged_subview(item.view.clone());
            }
        }
        self.update_dividers();
        self.split_view.view_mut().layout_subviews();
    }

    /// Compute divider positions from item preferred sizes.
    fn update_dividers(&mut self) {
        let frame = self.split_view.view().frame();
        let total = if self.is_vertical {
            frame.width()
        } else {
            frame.height()
        };
        if total <= 0.0 {
            return;
        }

        let visible: Vec<&NSSplitViewItem> =
            self.items.iter().filter(|i| !i.is_collapsed).collect();

        if visible.len() < 2 {
            return;
        }

        // Compute divider positions based on item behavior
        let mut positions = Vec::new();
        let mut consumed = 0.0_f32;

        for (i, item) in visible.iter().enumerate() {
            if i == visible.len() - 1 {
                break;
            } // last item fills remaining

            let size = match item.behavior {
                NSSplitViewItemBehavior::Sidebar => item.preferred_thickness.min(total * 0.3),
                NSSplitViewItemBehavior::Inspector => item.preferred_thickness.min(total * 0.3),
                NSSplitViewItemBehavior::Content => {
                    // Content gets remaining space — compute later
                    let sidebars_total: f32 = visible
                        .iter()
                        .filter(|v| {
                            matches!(
                                v.behavior,
                                NSSplitViewItemBehavior::Sidebar
                                    | NSSplitViewItemBehavior::Inspector
                            )
                        })
                        .map(|v| v.preferred_thickness)
                        .sum();
                    (total - sidebars_total).max(100.0)
                }
            };

            consumed += size;
            positions.push(consumed as f64);
        }

        for (i, pos) in positions.iter().enumerate() {
            self.split_view.set_divider_at_index(i, *pos);
        }
    }
}
