//! # NSBrowser Delegate
//!
//! Delegate protocol for browser data source and event handling.
//!
//! ## Apple Documentation
//! - [NSBrowserDelegate](https://developer.apple.com/documentation/appkit/nsbrowserdelegate)
//! - [NSBrowser](https://developer.apple.com/documentation/appkit/nsbrowser)

use crate::prelude::*;
use std::sync::{Arc, Mutex};

/// Delegate protocol for NSBrowser data source and events.
///
/// Apple: [NSBrowserDelegate](https://developer.apple.com/documentation/appkit/nsbrowserdelegate)
pub trait NSBrowserDelegate: Send + Sync {
    /// Return the number of children at the given path.
    ///
    /// The path is a vector of indices representing the navigation from root to the current level.
    /// An empty path represents the root level.
    ///
    /// Apple: `browser:numberOfChildrenOfItem:`
    fn number_of_children(&self, path: &[usize]) -> usize;

    /// Return the label for the child at the given index within the specified path.
    ///
    /// Apple: `browser:child:ofItem:`
    fn child_at_index(&self, path: &[usize], index: usize) -> String;

    /// Return whether the item at the given path is a leaf node (has no children).
    ///
    /// Apple: `browser:isLeafItem:`
    fn is_leaf(&self, path: &[usize]) -> bool;

    /// Called when the user selects an item in the browser.
    ///
    /// Apple: `browser:didChangeLastColumn:`
    fn did_select_item(&mut self, path: &[usize]) {
        let _ = path;
    }

    /// Called when the user double-clicks an item in the browser.
    ///
    /// Apple: `browser:shouldEditItem:`
    fn did_double_click_item(&mut self, path: &[usize]) {
        let _ = path;
    }
}

/// Internal state for browser control
pub(crate) struct BrowserState {
    /// Currently selected path (indices from root to selected item)
    pub(crate) selected_path: Vec<usize>,
    /// Number of visible columns
    pub(crate) visible_columns: usize,
    /// Maximum number of visible columns
    pub(crate) max_visible_columns: usize,
    /// Width of each column
    pub(crate) column_width: f32,
    /// Whether empty selection is allowed
    pub(crate) allows_empty_selection: bool,
    /// Whether the browser needs to reload data
    pub(crate) needs_reload: bool,
    /// Cached column data for rendering
    pub(crate) columns: Vec<ColumnData>,
    /// Index of the item being hovered in each column
    pub(crate) hover_indices: Vec<Option<usize>>,
    /// Scroll offset for horizontal scrolling
    pub(crate) scroll_offset: f32,
}

impl Default for BrowserState {
    fn default() -> Self {
        Self {
            selected_path: Vec::new(),
            visible_columns: 3,
            max_visible_columns: 5,
            column_width: 150.0,
            allows_empty_selection: false,
            needs_reload: true,
            columns: Vec::new(),
            hover_indices: Vec::new(),
            scroll_offset: 0.0,
        }
    }
}

/// Data for a single column in the browser
#[derive(Clone)]
pub(crate) struct ColumnData {
    /// Path to this column (indices from root)
    pub(crate) path: Vec<usize>,
    /// Items in this column
    pub(crate) items: Vec<String>,
    /// Selected index in this column (if any)
    pub(crate) selected_index: Option<usize>,
}

/// Internal view delegate for NSBrowser rendering and events.
pub(crate) struct NSBrowserViewDelegate {
    pub(crate) state: Arc<Mutex<BrowserState>>,
    pub(crate) delegate: Arc<Mutex<Option<Box<dyn NSBrowserDelegate>>>>,
}

impl NSBrowserViewDelegate {
    /// Reload column data from the delegate
    fn reload_columns(&self, state: &mut BrowserState) {
        let delegate_lock = self.delegate.lock().unwrap();
        if let Some(ref delegate) = *delegate_lock {
            state.columns.clear();
            state.hover_indices.clear();

            // Build columns based on selected path
            let mut current_path = Vec::new();
            
            // Always show root column
            let root_items = self.load_column_items(delegate.as_ref(), &current_path);
            state.columns.push(ColumnData {
                path: current_path.clone(),
                items: root_items,
                selected_index: state.selected_path.get(0).copied(),
            });
            state.hover_indices.push(None);

            // Build columns for each level in the selected path
            for (depth, &selected_index) in state.selected_path.iter().enumerate() {
                current_path.push(selected_index);
                
                // Only add column if the selected item is not a leaf
                if !delegate.is_leaf(&current_path) {
                    let items = self.load_column_items(delegate.as_ref(), &current_path);
                    state.columns.push(ColumnData {
                        path: current_path.clone(),
                        items,
                        selected_index: state.selected_path.get(depth + 1).copied(),
                    });
                    state.hover_indices.push(None);
                }
            }

            state.needs_reload = false;
        }
    }

    /// Load items for a column at the given path
    fn load_column_items(&self, delegate: &dyn NSBrowserDelegate, path: &[usize]) -> Vec<String> {
        let count = delegate.number_of_children(path);
        (0..count)
            .map(|i| delegate.child_at_index(path, i))
            .collect()
    }

    /// Get the column index and item index at the given point
    fn hit_test(&self, state: &BrowserState, point: NSPoint, frame: NSRect) -> Option<(usize, usize)> {
        let x = point.x + state.scroll_offset;
        let column_index = (x / state.column_width) as usize;
        
        if column_index >= state.columns.len() {
            return None;
        }

        let item_height = 20.0;
        let y = point.y;
        let item_index = (y / item_height) as usize;

        if item_index < state.columns[column_index].items.len() {
            Some((column_index, item_index))
        } else {
            None
        }
    }
}

impl NSViewDelegate for NSBrowserViewDelegate {
    fn draw(&self, view: &NSView, context: &mut crate::render::RenderContext) {
        let mut state = self.state.lock().unwrap();
        let frame = view.bounds();

        // Reload if needed
        if state.needs_reload {
            self.reload_columns(&mut state);
        }

        // Draw background
        context.set_color(crate::theme::color(crate::theme::keys::CLR_BG_WINDOW));
        context.fill_rect(frame);

        // Draw columns
        let item_height = 20.0;
        let column_width = state.column_width;
        
        for (col_idx, column) in state.columns.iter().enumerate() {
            let col_x = col_idx as f32 * column_width - state.scroll_offset;
            
            // Skip columns that are off-screen
            if col_x + column_width < 0.0 || col_x > frame.width() {
                continue;
            }

            // Draw column background
            let col_rect = NSRect::new(col_x, 0.0, column_width, frame.height());
            context.set_color(crate::theme::color(crate::theme::keys::CLR_BG_CONTENT));
            context.fill_rect(col_rect);

            // Draw column separator (using a thin rectangle instead of stroke_line)
            let separator_rect = NSRect::new(col_x + column_width - 1.0, 0.0, 1.0, frame.height());
            context.set_color(crate::theme::color(crate::theme::keys::CLR_BORDER));
            context.fill_rect(separator_rect);

            // Draw items
            for (item_idx, item) in column.items.iter().enumerate() {
                let item_y = item_idx as f32 * item_height;
                let item_rect = NSRect::new(col_x, item_y, column_width, item_height);

                // Highlight selected item
                if Some(item_idx) == column.selected_index {
                    context.set_color(crate::theme::color(crate::theme::keys::CLR_ACCENT));
                    context.fill_rect(item_rect);
                }
                // Highlight hovered item
                else if state.hover_indices.get(col_idx) == Some(&Some(item_idx)) {
                    context.set_color(crate::theme::color(crate::theme::keys::CLR_BG_CONTROL));
                    context.fill_rect(item_rect);
                }

                // Draw item text
                let text_color = if Some(item_idx) == column.selected_index {
                    NSColor::white()
                } else {
                    crate::theme::color(crate::theme::keys::CLR_TEXT_LABEL)
                };
                context.set_color(text_color);
                context.draw_text(
                    item,
                    NSPoint::new(col_x + 8.0, item_y + 4.0),
                    12.0,
                );
            }
        }
    }

    fn mouse_down(&mut self, view: &mut NSView, event: &crate::event::NSEvent) {
        let mut state = self.state.lock().unwrap();
        let point = event.location_in_window();

        if let Some((col_idx, item_idx)) = self.hit_test(&state, point, view.frame()) {
            // Build new selected path
            let mut new_path = state.columns[col_idx].path.clone();
            new_path.push(item_idx);

            state.selected_path = new_path.clone();
            state.needs_reload = true;
            drop(state);

            // Notify delegate
            if let Some(ref mut delegate) = *self.delegate.lock().unwrap() {
                delegate.did_select_item(&new_path);
            }

            view.set_needs_display();
        }
    }

    fn mouse_moved(&mut self, view: &mut NSView, event: &crate::event::NSEvent) {
        let mut state = self.state.lock().unwrap();
        let point = event.location_in_window();

        // Update hover state
        let mut needs_redraw = false;
        if let Some((col_idx, item_idx)) = self.hit_test(&state, point, view.frame()) {
            if state.hover_indices.get(col_idx) != Some(&Some(item_idx)) {
                // Clear all hover states
                for hover in state.hover_indices.iter_mut() {
                    *hover = None;
                }
                // Set new hover state
                if col_idx < state.hover_indices.len() {
                    state.hover_indices[col_idx] = Some(item_idx);
                }
                needs_redraw = true;
            }
        } else {
            // Clear hover states if not over any item
            let had_hover = state.hover_indices.iter().any(|h| h.is_some());
            if had_hover {
                for hover in state.hover_indices.iter_mut() {
                    *hover = None;
                }
                needs_redraw = true;
            }
        }

        drop(state);
        if needs_redraw {
            view.set_needs_display();
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
