//! # NSTabView — Tabbed Content Pane Switcher
//!
//! Mirrors Apple's [`NSTabView`](https://developer.apple.com/documentation/appkit/nstabview)
//! from the AppKit framework.  Presents a row of clickable tabs at the top of
//! the view; clicking a tab reveals the associated content pane and hides all
//! others — matching the behaviour of `NSTabView.selectTabViewItem(_:)`.
//!
//! ## Usage
//!
//! ```rust,ignore
//! let mut tabs = NSTabView::new(rect);
//! tabs.add_tab_view_item(NSTabViewItem::new("Documents", documents_view));
//! tabs.add_tab_view_item(NSTabViewItem::new("Settings", settings_view));
//! tabs.set_selected_index(0); // Show the first tab
//! ```

pub mod delegate;
pub mod item;

use crate::{NSAutoresizingMask, NSColor, NSRect, NSView};
pub use item::NSTabViewItem;
use std::sync::{Arc, Mutex};

/// A tab view control that switches between multiple content panes.
///
/// Mirrors Swift's [`NSTabView`](https://developer.apple.com/documentation/appkit/nstabview).
/// Each tab is represented by an [`NSTabViewItem`] holding a label and a
/// content view.  The delegate handles tab-bar rendering, hover highlighting,
/// click selection, and layout of the active pane.
#[derive(Clone)]
pub struct NSTabView {
    pub(crate) view: NSView,
    /// Ordered list of tab items.
    pub(crate) items: Arc<Mutex<Vec<NSTabViewItem>>>,
    /// Index of the currently selected (visible) tab, or `None` if empty.
    pub(crate) selected_index: Arc<Mutex<Option<usize>>>,
}

impl NSTabView {
    /// Creates a new tab view with the given frame.
    ///
    /// The view is initially empty — call [`add_tab_view_item`](Self::add_tab_view_item)
    /// to populate it.
    pub fn new(rect: NSRect) -> Self {
        let items = Arc::new(Mutex::new(Vec::new()));
        let selected_index = Arc::new(Mutex::new(None));
        let mut view = NSView::new(rect).with_name("NSTabView");
        view.set_delegate(delegate::NSTabViewDelegate {
            items: items.clone(),
            selected_index: selected_index.clone(),
            hover_index: Arc::new(Mutex::new(None)),
        });
        Self {
            view,
            items,
            selected_index,
        }
    }

    /// Appends a tab item and selects it if this is the first tab added.
    ///
    /// Mirrors Swift's `NSTabView.addTabViewItem(_:)`.
    pub fn add_tab_view_item(&mut self, item: NSTabViewItem) {
        self.view.add_subview(item.view.clone());
        self.items.lock().unwrap().push(item);
        if self.selected_index.lock().unwrap().is_none() {
            *self.selected_index.lock().unwrap() = Some(0);
        }
        self.view.layout_subviews();
    }

    /// Programmatically selects the tab at the given zero-based index.
    ///
    /// Mirrors Swift's `NSTabView.selectTabViewItem(at:)`.  Triggers an
    /// immediate layout pass and redraw so the selected pane becomes visible.
    pub fn set_selected_index(&mut self, index: usize) {
        if let Ok(mut sel) = self.selected_index.lock() {
            *sel = Some(index);
        }
        self.view.layout_subviews();
        self.view.set_needs_display();
    }

    /// Returns a shared reference to the underlying `NSView`.
    pub fn view(&self) -> &NSView {
        &self.view
    }

    /// Returns a mutable reference to the underlying `NSView`.
    pub fn view_mut(&mut self) -> &mut NSView {
        &mut self.view
    }
}
