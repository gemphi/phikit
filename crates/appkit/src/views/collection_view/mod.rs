//! # NSCollectionView — Grid-Based Item Display
//!
//! A view that displays a grid of items provided by a data source.
//!
//! ## Apple Documentation
//! - [NSCollectionView](https://developer.apple.com/documentation/appkit/nscollectionview)
//! - [NSCollectionViewDataSource](https://developer.apple.com/documentation/appkit/nscollectionviewdatasource)
//! - [NSCollectionViewDelegate](https://developer.apple.com/documentation/appkit/nscollectionviewdelegate)
//!
//! ## Usage
//!
//! ```rust,ignore
//! let mut collection = NSCollectionView::new(frame);
//! collection.set_data_source(MyDataSource);
//! collection.set_delegate(MyDelegate);
//! collection.set_item_size(NSRect::sized(90.0, 90.0));
//! ```
//!
//! ## See Also
//! - [NSTableView](https://developer.apple.com/documentation/appkit/nstableview)
//! - [NSOutlineView](https://developer.apple.com/documentation/appkit/nsoutlineview)

pub mod builder;
pub mod delegate;
pub mod traits;

use crate::{NSRect, NSView};
pub(crate) use delegate::CollectionViewDelegate;
use std::sync::{Arc, Mutex};
pub use traits::{NSCollectionViewDataSource, NSCollectionViewDelegate};

/// A view that displays a grid of items provided by a data source.
///
/// manages selection state and coordinates with an internal delegate for layout.
/// supports customizable item sizes and interaction protocols.
#[derive(Clone)]
pub struct NSCollectionView {
    pub(crate) view: NSView,
    pub(crate) data_source: Option<Arc<Mutex<dyn NSCollectionViewDataSource>>>,
    pub(crate) delegate: Option<Arc<Mutex<dyn NSCollectionViewDelegate>>>,
    pub(crate) item_size: NSRect,
    pub(crate) selected_index: Option<usize>,
}

impl NSCollectionView {
    /// Creates a new collection view with the given frame.
    /// initializes with a default item size and no data source.
    pub fn new(frame: NSRect) -> Self {
        Self {
            view: NSView::new(frame),
            data_source: None,
            delegate: None,
            item_size: NSRect::sized(90.0, 90.0),
            selected_index: None,
        }
    }

    /// Sets the data source for providing items to the grid.
    /// automatically installs the internal view delegate for rendering.
    pub fn set_data_source(&mut self, ds: impl NSCollectionViewDataSource + 'static) {
        let arc_ds = Arc::new(Mutex::new(ds));
        self.data_source = Some(arc_ds.clone());
        let delegate = CollectionViewDelegate {
            item_size: self.item_size,
            data_source: arc_ds,
            collection_delegate: self.delegate.clone(),
            item_views: Mutex::new(Vec::new()),
        };
        self.view.set_delegate(delegate);
    }

    /// Sets the delegate for responding to interaction events.
    /// refreshes the internal view delegate to include the new protocol provider.
    pub fn set_delegate(&mut self, delegate: impl NSCollectionViewDelegate + 'static) {
        let arc_del = Arc::new(Mutex::new(delegate));
        self.delegate = Some(arc_del);
        if let Some(ref ds) = self.data_source {
            let view_delegate = CollectionViewDelegate {
                item_size: self.item_size,
                data_source: ds.clone(),
                collection_delegate: self.delegate.clone(),
                item_views: Mutex::new(Vec::new()),
            };
            self.view.set_delegate(view_delegate);
        }
    }

    /// Programmatically selects an item at the given index.
    /// notifies both the data source and the delegate of the change.
    pub fn select_item_at_index(&mut self, index: Option<usize>) {
        self.selected_index = index;
        if let Some(ref ds) = self.data_source {
            if let Ok(mut ds_guard) = ds.lock() {
                if let Some(idx) = index {
                    ds_guard.collection_view_did_select_item(idx);
                }
            }
        }
        if let Some(ref delegate) = self.delegate {
            if let Ok(mut delegate_guard) = delegate.lock() {
                delegate_guard.collection_view_selection_did_change(index);
            }
        }
    }

    /// Returns the index of the currently selected item.
    pub fn selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    /// Returns a shared reference to the underlying NSView.
    pub fn view(&self) -> &NSView {
        &self.view
    }

    /// Returns a mutable reference to the underlying NSView.
    pub fn view_mut(&mut self) -> &mut NSView {
        &mut self.view
    }
}
