//! # NSBrowser — Column-Based Browser Control
//!
//! A browser control that displays hierarchical data in multiple columns, allowing users to
//! navigate through nested content. Each column shows the children of the selected item in
//! the previous column.
//!
//! ## Apple Documentation
//! - [NSBrowser](https://developer.apple.com/documentation/appkit/nsbrowser)
//! - [NSBrowserDelegate](https://developer.apple.com/documentation/appkit/nsbrowserdelegate)
//!
//! ## Usage
//!
//! ```rust,ignore
//! let mut browser = NSBrowser::new(frame);
//! browser.set_delegate(Box::new(MyBrowserDelegate));
//! browser.reload();
//! ```
//!
//! ## Delegate Pattern
//!
//! ```rust,ignore
//! struct MyBrowserDelegate;
//!
//! impl NSBrowserDelegate for MyBrowserDelegate {
//!     fn number_of_children(&self, path: &[usize]) -> usize {
//!         // Return number of children at this path
//!         if path.is_empty() {
//!             3 // Root level has 3 items
//!         } else {
//!             0 // Leaf nodes
//!         }
//!     }
//!
//!     fn child_at_index(&self, path: &[usize], index: usize) -> String {
//!         // Return the label for the child at the given index
//!         format!("Item {}", index)
//!     }
//!
//!     fn is_leaf(&self, path: &[usize]) -> bool {
//!         // Return true if this path represents a leaf node
//!         !path.is_empty()
//!     }
//! }
//! ```
//!
//! ## See Also
//! - [NSBrowser](https://developer.apple.com/documentation/appkit/nsbrowser)
//! - [NSOutlineView](https://developer.apple.com/documentation/appkit/nsoutlineview)

pub mod delegate;

pub use delegate::NSBrowserDelegate;

use crate::graphics::NSRect;
use crate::view::NSView;
use delegate::{BrowserState, NSBrowserViewDelegate};
use std::sync::{Arc, Mutex};

/// Column-based browser control for hierarchical data navigation.
///
/// Apple: [NSBrowser](https://developer.apple.com/documentation/appkit/nsbrowser)
#[derive(Clone)]
pub struct NSBrowser {
    view: NSView,
    state: Arc<Mutex<BrowserState>>,
    delegate: Arc<Mutex<Option<Box<dyn NSBrowserDelegate>>>>,
}

impl NSBrowser {
    /// Create a new browser control.
    ///
    /// Apple: `[[NSBrowser alloc] initWithFrame:]`
    pub fn new(frame: NSRect) -> Self {
        let state = Arc::new(Mutex::new(BrowserState::default()));
        let delegate = Arc::new(Mutex::new(None));

        let mut view = NSView::new(frame);
        view.set_delegate(NSBrowserViewDelegate {
            state: state.clone(),
            delegate: delegate.clone(),
        });

        Self {
            view,
            state,
            delegate,
        }
    }

    /// Set the delegate for browser events and data source.
    ///
    /// Apple: `setDelegate:`
    pub fn set_delegate(&mut self, delegate: Box<dyn NSBrowserDelegate>) {
        *self.delegate.lock().unwrap() = Some(delegate);
    }

    /// Reload the browser data from the delegate.
    ///
    /// Apple: `reloadColumn:`
    pub fn reload(&mut self) {
        let mut state = self.state.lock().unwrap();
        state.needs_reload = true;
        drop(state);
        self.view.set_needs_display();
    }

    /// Get the currently selected path.
    ///
    /// Returns a vector of indices representing the path from root to the selected item.
    ///
    /// Apple: `selectionIndexPath`
    pub fn selected_path(&self) -> Vec<usize> {
        self.state.lock().unwrap().selected_path.clone()
    }

    /// Set the selected path programmatically.
    ///
    /// Apple: `setSelectionIndexPath:`
    pub fn set_selected_path(&mut self, path: Vec<usize>) {
        let mut state = self.state.lock().unwrap();
        state.selected_path = path;
        state.needs_reload = true;
        drop(state);
        self.view.set_needs_display();
    }

    /// Get the number of visible columns.
    ///
    /// Apple: `numberOfVisibleColumns`
    pub fn number_of_visible_columns(&self) -> usize {
        self.state.lock().unwrap().visible_columns
    }

    /// Set the number of visible columns.
    ///
    /// Apple: `setMaxVisibleColumns:`
    pub fn set_max_visible_columns(&mut self, max: usize) {
        let mut state = self.state.lock().unwrap();
        state.max_visible_columns = max;
        drop(state);
        self.view.set_needs_display();
    }

    /// Get the width of each column.
    ///
    /// Apple: `columnWidthForColumnContentWidth:`
    pub fn column_width(&self) -> f32 {
        self.state.lock().unwrap().column_width
    }

    /// Set the width of each column.
    ///
    /// Apple: `setColumnResizingType:`
    pub fn set_column_width(&mut self, width: f32) {
        let mut state = self.state.lock().unwrap();
        state.column_width = width;
        drop(state);
        self.view.set_needs_display();
    }

    /// Get whether the browser allows empty selection.
    ///
    /// Apple: `allowsEmptySelection`
    pub fn allows_empty_selection(&self) -> bool {
        self.state.lock().unwrap().allows_empty_selection
    }

    /// Set whether the browser allows empty selection.
    ///
    /// Apple: `setAllowsEmptySelection:`
    pub fn set_allows_empty_selection(&mut self, allows: bool) {
        self.state.lock().unwrap().allows_empty_selection = allows;
    }

    // ── View Access ──────────────────────────────────────────────────

    pub fn view(&self) -> &NSView {
        &self.view
    }

    pub fn view_mut(&mut self) -> &mut NSView {
        &mut self.view
    }

    pub fn view_clone(&self) -> NSView {
        self.view.clone()
    }
}
