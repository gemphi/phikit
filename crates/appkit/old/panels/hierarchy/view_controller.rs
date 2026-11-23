//! Hierarchy view controller.
//! 
//! Follows Apple's [`NSOutlineViewController`](https://developer.apple.com/documentation/appkit/nsoutlineview) 
//! pattern for hierarchical data display.
//!
//! ## Apple Pattern
//! Mirrors Apple's outline view pattern:
//! - [`NSOutlineView`](https://developer.apple.com/documentation/appkit/nsoutlineview) - Tree/hierarchy display
//! - [`NSOutlineViewDataSource`](https://developer.apple.com/documentation/appkit/nsoutlineviewdatasource) - Data source protocol
//! - [`NSOutlineViewDelegate`](https://developer.apple.com/documentation/appkit/nsoutlineviewdelegate) - Delegate protocol
//!
//! ## Features
//! - Data source binding via HierarchyDataSource trait
//! - Selection management
//! - Expand/collapse
//! - Drag and drop
//!
//! ## Architecture
//! ```
//! NSHierarchyViewController (thin controller)
//!   └── NSOutlineView (view component - handles all UI)
//!       └── HierarchyDataSource (provides tree data)
//! ```
//!
//! ## See Also
//! - [NSOutlineView](https://developer.apple.com/documentation/appkit/nsoutlineview)
//! - [NSOutlineViewDataSource](https://developer.apple.com/documentation/appkit/nsoutlineviewdatasource)
//! - [Outline View Programming Guide](https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/OutlineView/OutlineView.html)

use super::model::*;
use crate::outline_view::NSOutlineView;
use crate::prelude::*;
use crate::theme::{self, keys};
use crate::layout::{spacing, hierarchy};
use std::sync::{Arc, Mutex};

/// Hierarchy view controller following Apple's NSOutlineViewController pattern.
/// 
/// Manages a tree view of entities with support for:
/// - Data source binding via HierarchyDataSource trait
/// - Selection management
/// - Expand/collapse
/// - Drag and drop
pub struct NSHierarchyViewController {
    pub view: NSView,
    pub outline_view: NSOutlineView,
    data_source: Option<Arc<Mutex<dyn HierarchyDataSource>>>,
    selected_id: Option<u64>,
    on_selection_changed: Option<SelectionCallback>,
    on_drag_drop: Option<DragDropCallback>,
    on_expand_collapse: Option<ExpandCollapseCallback>,
}

impl NSHierarchyViewController {
    /// Create a new hierarchy view controller.
    pub fn new(rect: NSRect) -> Self {
        let mut view = NSView::new(rect).with_name("HierarchyPanel");
        view.set_autoresizing_mask(
            NSAutoresizingMask::WIDTH_SIZABLE | NSAutoresizingMask::HEIGHT_SIZABLE,
        );
        view.set_background_color(Some(hierarchy::bg_color("hierarchy")));
        view.set_corner_radius(crate::theme::ThemeToken::RADIUS_PANEL());
        view.set_glass(true);
        view.set_clips_to_bounds(true);
        
        let outline_view = NSOutlineView::new(rect);
        
        Self {
            view,
            outline_view,
            data_source: None,
            selected_id: None,
            on_selection_changed: None,
            on_drag_drop: None,
            on_expand_collapse: None,
        }
    }

    // ─── Data Source ─────────────────────────────────────────────────

    /// Set the data source for the hierarchy (Apple pattern: setDataSource:).
    pub fn set_data_source(&mut self, source: Arc<Mutex<dyn HierarchyDataSource>>) {
        self.data_source = Some(source);
        self.reload_data();
        self.sync_outline_subview();
    }

    /// Reload data from the data source (Apple pattern: reloadData).
    pub fn reload_data(&mut self) {
        if let Some(ref source) = self.data_source {
            if let Ok(ds) = source.lock() {
                let roots = ds.root_items();
                // Update outline view with new data
                println!("[Hierarchy] Reloaded {} root items", roots.len());
            }
        }
    }

    // ─── Selection ───────────────────────────────────────────────────

    /// Select an entity by ID (Apple pattern: selectRowIndexes:byExtendingSelection:).
    pub fn select_item(&mut self, entity_id: u64) {
        self.selected_id = Some(entity_id);
        if let Some(ref callback) = self.on_selection_changed {
            callback(Some(entity_id));
        }
    }

    /// Clear selection (Apple pattern: deselectAll:).
    pub fn clear_selection(&mut self) {
        self.selected_id = None;
        if let Some(ref callback) = self.on_selection_changed {
            callback(None);
        }
    }

    /// Get the currently selected entity ID.
    pub fn selected_item(&self) -> Option<u64> {
        self.selected_id
    }

    // ─── Expand/Collapse ─────────────────────────────────────────────

    /// Expand an entity to show its children (Apple pattern: expandItem:).
    pub fn expand_item(&mut self, entity_id: u64) {
        if let Some(ref callback) = self.on_expand_collapse {
            callback(entity_id, true);
        }
    }

    /// Collapse an entity to hide its children (Apple pattern: collapseItem:).
    pub fn collapse_item(&mut self, entity_id: u64) {
        if let Some(ref callback) = self.on_expand_collapse {
            callback(entity_id, false);
        }
    }

    // ─── Callbacks (Delegate Pattern) ───────────────────────────────

    /// Set callback for selection changes (Apple pattern: outlineViewSelectionDidChange:).
    pub fn set_on_selection_changed<F>(&mut self, callback: F)
    where
        F: Fn(Option<u64>) + Send + Sync + 'static,
    {
        self.on_selection_changed = Some(Box::new(callback));
    }

    /// Set callback for drag/drop operations (Apple pattern: outlineView:acceptDrop:item:childIndex:).
    pub fn set_on_drag_drop<F>(&mut self, callback: F)
    where
        F: Fn(u64, Option<u64>) -> bool + Send + Sync + 'static,
    {
        self.on_drag_drop = Some(Box::new(callback));
    }

    /// Set callback for expand/collapse events (Apple pattern: outlineViewItemDidExpand:).
    pub fn set_on_expand_collapse<F>(&mut self, callback: F)
    where
        F: Fn(u64, bool) + Send + Sync + 'static,
    {
        self.on_expand_collapse = Some(Box::new(callback));
    }

    // ─── Internal ────────────────────────────────────────────────────

    /// Re-sync the outline view's NSView into the parent view hierarchy.
    /// Must be called after set_data_source so the delegate is attached.
    pub fn sync_outline_subview(&mut self) {
        self.view.remove_all_subviews();
        self.view.add_subview(self.outline_view.view().clone());
    }

    /// Handle drag operation (called by NSOutlineView delegate).
    pub(crate) fn handle_drag(&mut self, dragged_id: u64, target_id: Option<u64>) -> bool {
        if let Some(ref callback) = self.on_drag_drop {
            callback(dragged_id, target_id)
        } else {
            false
        }
    }

    // ─── Legacy Methods (Deprecated) ─────────────────────────────────

    /// @deprecated Use select_item() instead.
    pub fn set_selected_entity(&mut self, entity_id: Option<u64>) {
        if let Some(id) = entity_id {
            self.select_item(id);
        } else {
            self.clear_selection();
        }
    }
}

impl EditorPanel for NSHierarchyViewController {
    fn view(&self) -> &NSView { &self.view }
    fn view_mut(&mut self) -> &mut NSView { &mut self.view }
    
    fn on_selection_changed(&mut self, entity_id: Option<u64>) {
        if let Some(id) = entity_id {
            self.select_item(id);
        } else {
            self.clear_selection();
        }
    }
    
    fn reload_data(&mut self) {
        self.reload_data();
    }
    
    fn name(&self) -> &str { "Hierarchy" }
    fn min_size(&self) -> f32 { 200.0 }
    fn preferred_size(&self) -> f32 { 260.0 }
    
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

use crate::panels::EditorPanel;
