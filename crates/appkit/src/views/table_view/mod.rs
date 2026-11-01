//! # NSTableView
//!
//! Mirrors Apple's [`NSTableView`](https://developer.apple.com/documentation/appkit/nstableview).
//! Displays tabular data in rows and columns with selection, sorting, and scrolling.
//!
//! ## Architecture
//!
//! | Type | Apple Equivalent | Role |
//! |------|-----------------|------|
//! | `NSTableView` | `NSTableView` | The table container |
//! | `NSTableColumn` | `NSTableColumn` | Column definition (identifier, title, width) |
//! | `NSTableViewDataSource` | `NSTableViewDataSource` | Provides row count and cell values |
//! | `NSTableViewDelegate` | `NSTableViewDelegate` | Selection, row height, custom cells |
//!
//! ## Usage
//!
//! ```rust,ignore
//! let mut table = NSTableView::new(frame);
//! table.add_column(NSTableColumn::new("name", "Name", 200.0));
//! table.add_column(NSTableColumn::new("type", "Type", 100.0));
//! table.set_data_source(my_data_source);
//! ```

pub mod delegate;

use crate::theme::{self, keys};
use crate::{NSColor, NSRect, NSSize, NSView};
use std::sync::{Arc, Mutex};

// ═══════════════════════════════════════════════════════════════════════
// NSTableColumn
// ═══════════════════════════════════════════════════════════════════════

/// A column in an NSTableView.
#[derive(Debug, Clone)]
pub struct NSTableColumn {
    pub identifier: String,
    pub title: String,
    pub width: f32,
    pub min_width: f32,
    pub max_width: f32,
    pub is_editable: bool,
    pub is_hidden: bool,
}

impl NSTableColumn {
    pub fn new(identifier: &str, title: &str, width: f32) -> Self {
        Self {
            identifier: identifier.to_string(),
            title: title.to_string(),
            width,
            min_width: 40.0,
            max_width: f32::MAX,
            is_editable: false,
            is_hidden: false,
        }
    }

    pub fn with_min_width(mut self, w: f32) -> Self {
        self.min_width = w;
        self
    }
    pub fn with_max_width(mut self, w: f32) -> Self {
        self.max_width = w;
        self
    }
    pub fn with_editable(mut self, e: bool) -> Self {
        self.is_editable = e;
        self
    }
}

// ═══════════════════════════════════════════════════════════════════════
// NSTableViewDataSource
// ═══════════════════════════════════════════════════════════════════════

/// Data source protocol for NSTableView.
pub trait NSTableViewDataSource: Send + Sync {
    /// Number of rows in the table.
    fn number_of_rows(&self) -> usize;

    /// Value for a cell at (row, column_identifier).
    fn value_for(&self, row: usize, column: &str) -> String;
}

// ═══════════════════════════════════════════════════════════════════════
// NSTableViewDelegate
// ═══════════════════════════════════════════════════════════════════════

/// Delegate protocol for NSTableView.
pub trait NSTableViewDelegate: Send + Sync {
    /// Called when selection changes.
    fn table_view_selection_did_change(&mut self, selected_row: Option<usize>) {}

    /// Row height (default from theme).
    fn table_view_height_of_row(&self, _row: usize) -> f32 {
        theme::f32_or(keys::SIZE_CONTROL_HEIGHT, 24.0)
    }
}

// ═══════════════════════════════════════════════════════════════════════
// NSTableView
// ═══════════════════════════════════════════════════════════════════════

/// A table view displaying rows and columns of data.
#[derive(Clone)]
pub struct NSTableView {
    pub(crate) view: NSView,
    pub(crate) columns: Arc<Mutex<Vec<NSTableColumn>>>,
    pub(crate) selected_row: Arc<Mutex<Option<usize>>>,
    pub(crate) data_source: Option<Arc<dyn NSTableViewDataSource>>,
}

impl NSTableView {
    pub fn new(frame: NSRect) -> Self {
        let columns = Arc::new(Mutex::new(Vec::new()));
        let selected_row = Arc::new(Mutex::new(None));
        let view = NSView::new(frame).with_name("NSTableView");

        Self {
            view,
            columns,
            selected_row,
            data_source: None,
        }
    }

    /// Add a column definition.
    pub fn add_column(&mut self, column: NSTableColumn) {
        self.columns.lock().unwrap().push(column);
    }

    /// Set the data source and install the rendering delegate.
    pub fn set_data_source(&mut self, ds: impl NSTableViewDataSource + 'static) {
        let ds_arc = Arc::new(ds);
        self.data_source = Some(ds_arc.clone());
        let delegate = delegate::NSTableViewRenderDelegate {
            columns: self.columns.clone(),
            selected_row: self.selected_row.clone(),
            data_source: ds_arc,
        };
        self.view.set_delegate(delegate);
    }

    /// Programmatically select a row.
    pub fn select_row(&mut self, row: Option<usize>) {
        *self.selected_row.lock().unwrap() = row;
        self.view.set_needs_display();
    }

    /// Get the currently selected row.
    pub fn selected_row(&self) -> Option<usize> {
        *self.selected_row.lock().unwrap()
    }

    /// Reload data from the data source.
    pub fn reload_data(&mut self) {
        self.view.set_needs_display();
    }

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
