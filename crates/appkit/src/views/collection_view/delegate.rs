//! Internal Collection View Delegate
//!
//! this file implements the private view delegate that drives collection view layout,
//! rendering, and event handling. it translates low-level mouse events into
//! high-level data source and protocol notifications.

use super::traits::{NSCollectionViewDataSource, NSCollectionViewDelegate};
use crate::event::NSEvent;
use crate::render::RenderContext;
use crate::{view::NSViewDelegate, NSColor, NSPoint, NSRect, NSView};
use std::sync::{Arc, Mutex};

/// Internal delegate coordinating collection view logic.
///
/// handles the calculation of grid coordinates and item hit testing.
/// manages background drawing and distributes events to the appropriate providers.
pub(crate) struct CollectionViewDelegate {
    pub(crate) item_size: NSRect,
    pub(crate) data_source: Arc<Mutex<dyn NSCollectionViewDataSource>>,
    pub(crate) collection_delegate: Option<Arc<Mutex<dyn NSCollectionViewDelegate>>>,
    pub(crate) item_views: Mutex<Vec<NSView>>,
}

impl CollectionViewDelegate {
    /// Rebuilds the item views from the data source.
    /// Call this when data source changes or on initial layout.
    fn rebuild_item_views(&self, parent_view: &mut NSView, ds_guard: &std::sync::MutexGuard<'_, dyn NSCollectionViewDataSource>) {
        let subviews = parent_view.subviews_mut();
        subviews.retain(|v| !v.name.starts_with("FileItem_"));
        
        let items_count = ds_guard.number_of_items();
        let bounds = parent_view.bounds();
        let pad = 8.0_f32;
        let gap = 8.0_f32;
        let available_w = bounds.width() - pad * 2.0;
        let item_w = self.item_size.width() + gap;
        let items_per_row = (available_w / item_w).floor().max(1.0) as usize;
        
        let mut x = pad;
        let mut y = pad;
        let mut new_views = Vec::new();
        for i in 0..items_count {
            if i > 0 && i % items_per_row == 0 {
                x = pad;
                y += self.item_size.height() + gap;
            }
            let mut item_view = ds_guard.view_for_item(i);
            item_view.set_frame(NSRect::new(
                x, y, self.item_size.width(), self.item_size.height(),
            ));
            new_views.push(item_view);
            x += self.item_size.width() + gap;
        }
        
        for view in new_views.iter().cloned() {
            parent_view.add_subview(view);
        }
        *self.item_views.lock().unwrap() = new_views;
    }
}

impl NSViewDelegate for CollectionViewDelegate {
    /// Performs grid layout and adjusts the view's content size.
    /// calculates required height based on item count and available width.
    fn layout(&mut self, view: &mut NSView) {
        // First, collect all item views we need to create
        let items_to_create: Vec<NSView> = {
            let ds_guard = match self.data_source.lock() {
                Ok(g) => g,
                Err(_) => return,
            };
            let items_count = ds_guard.number_of_items();
            let frame = view.frame();
            let bounds = view.bounds();
            
            let pad = 8.0_f32; // Padding from edges
            let gap = 8.0_f32; // Gap between items
            let available_w = frame.width() - pad * 2.0;
            let item_w = self.item_size.width() + gap;
            let items_per_row = (available_w / item_w).floor().max(1.0) as usize;
            let rows = (items_count + items_per_row - 1) / items_per_row;
            let required_height = (rows as f32) * (self.item_size.height() + gap) + pad * 2.0;
            if (frame.size.height - required_height).abs() > 0.1 {
                view.frame.size.height = required_height.max(frame.size.height);
                view.bounds.size.height = required_height.max(bounds.size.height);
            }
            
            // Build all item views — start from TOP-LEFT
            let mut x = pad;
            let mut y = pad; // Start from top
            let mut new_views = Vec::new();
            for i in 0..items_count {
                if i > 0 && i % items_per_row == 0 {
                    x = pad;
                    y += self.item_size.height() + gap;
                }
                let mut item_view = ds_guard.view_for_item(i);
                item_view.set_frame(NSRect::new(
                    x, y, self.item_size.width(), self.item_size.height(),
                ));
                new_views.push(item_view);
                x += self.item_size.width() + gap;
            }
            new_views
        }; // ds_guard dropped here
        
        // Now clear old items and add new ones
        {
            let subviews = view.subviews_mut();
            subviews.retain(|v| !v.name.starts_with("FileItem_"));
        }
        
        // Add all new views
        for item_view in items_to_create {
            view.add_subview(item_view);
        }
    }

    /// Renders the collection view background.
    /// Item views are rendered as subviews.
    fn draw(&self, _view: &NSView, _ctx: &mut RenderContext) {
        // Background is handled by the parent FileView
        // Item subviews render themselves
    }

    /// Handles mouse clicks to identify and select grid items.
    /// translates window coordinates to grid-relative row and column indices.
    fn mouse_down(&mut self, view: &mut NSView, event: &NSEvent) {
        let frame = view.frame();
        let mouse_pos = event.location_in_window();
        let local_pos = NSPoint::new(mouse_pos.x - frame.origin.x, mouse_pos.y - frame.origin.y);
        if frame.contains_point(&local_pos) {
            let item_w = self.item_size.width() + 20.0;
            let items_per_row = ((frame.width() - 40.0) / item_w).floor().max(1.0) as usize;
            let col = ((local_pos.x - 20.0) / item_w).floor() as usize;
            let row = ((local_pos.y - 20.0) / (self.item_size.height() + 20.0)).floor() as usize;
            let index = row * items_per_row + col;
            if let Ok(mut ds_guard) = self.data_source.lock() {
                if index < ds_guard.number_of_items() {
                    ds_guard.collection_view_did_select_item(index);
                    if let Some(delegate_arc) = &self.collection_delegate {
                        if let Ok(mut delegate) = delegate_arc.lock() {
                            delegate.collection_view_selection_did_change(Some(index));
                            delegate.collection_view_item_did_double_click(index);
                        }
                    }
                }
            }
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
