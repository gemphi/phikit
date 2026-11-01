//! # NSPopUpButton — Dropdown Menu Control
//!
//! A button that displays a menu of choices when clicked.
//!
//! ## Apple Documentation
//! - [NSPopUpButton](https://developer.apple.com/documentation/appkit/nspopupbutton)
//! - [NSPopUpButtonCell](https://developer.apple.com/documentation/appkit/nspopupbuttoncell)
//! - [NSMenu](https://developer.apple.com/documentation/appkit/nsmenu)
//!
//! ## Usage
//!
//! ```rust,ignore
//! let mut popup = NSPopUpButton::new(frame);
//! popup.add_item("Wireframe");
//! popup.add_item("Solid");
//! popup.add_item("Material");
//! popup.select_item_at_index(1); // "Solid"
//! ```
//!
//! ## Delegate Pattern
//!
//! ```rust,ignore
//! struct MyDelegate;
//!
//! impl NSPopUpButtonDelegate for MyDelegate {
//!     fn will_pop_up(&mut self) {
//!         println!("Menu is about to open");
//!     }
//!     
//!     fn did_select_item(&mut self, index: usize, title: &str) {
//!         println!("Selected: {} at index {}", title, index);
//!     }
//! }
//!
//! let mut popup = NSPopUpButton::new(frame);
//! popup.set_delegate(Box::new(MyDelegate));
//! ```
//!
//! ## See Also
//! - [NSMenu](https://developer.apple.com/documentation/appkit/nsmenu)
//! - [NSComboBox](https://developer.apple.com/documentation/appkit/nscombobox)

pub mod delegate;

pub use delegate::NSPopUpButtonDelegate;

use crate::control::NSControl;
use crate::graphics::NSRect;
use crate::view::NSView;
use delegate::{NSPopUpButtonViewDelegate, PopUpButtonState};
use std::sync::{Arc, Mutex};

/// Popup button control (dropdown menu).
///
/// Apple: [NSPopUpButton](https://developer.apple.com/documentation/appkit/nspopupbutton)
#[derive(Clone)]
pub struct NSPopUpButton {
    control: NSControl,
    state: Arc<Mutex<PopUpButtonState>>,
    delegate: Arc<Mutex<Option<Box<dyn NSPopUpButtonDelegate>>>>,
}

impl NSPopUpButton {
    /// Create a new popup button.
    ///
    /// Apple: `[[NSPopUpButton alloc] initWithFrame:]`
    pub fn new(frame: NSRect) -> Self {
        let state = Arc::new(Mutex::new(PopUpButtonState::default()));
        let delegate = Arc::new(Mutex::new(None));
        
        let mut control = NSControl::new(frame);
        control.view_mut().set_delegate(NSPopUpButtonViewDelegate {
            state: state.clone(),
            delegate: delegate.clone(),
        });
        
        Self { control, state, delegate }
    }

    /// Add a menu item.
    ///
    /// Apple: `addItemWithTitle:`
    pub fn add_item(&mut self, title: &str) {
        self.state.lock().unwrap().items.push(title.to_string());
        self.control.view_mut().set_needs_display();
    }

    /// Add multiple items at once.
    ///
    /// Apple: `addItemsWithTitles:`
    pub fn add_items(&mut self, titles: &[&str]) {
        let mut state = self.state.lock().unwrap();
        for t in titles {
            state.items.push(t.to_string());
        }
        drop(state);
        self.control.view_mut().set_needs_display();
    }

    /// Select an item by index.
    ///
    /// Apple: `selectItemAtIndex:`
    pub fn select_item_at_index(&mut self, index: usize) {
        self.state.lock().unwrap().selected_index = index;
        self.control.view_mut().set_needs_display();
    }

    /// Get the currently selected index.
    ///
    /// Apple: `indexOfSelectedItem`
    pub fn index_of_selected_item(&self) -> usize {
        self.state.lock().unwrap().selected_index
    }

    /// Get the title of the selected item.
    ///
    /// Apple: `titleOfSelectedItem`
    pub fn title_of_selected_item(&self) -> String {
        let state = self.state.lock().unwrap();
        state.items.get(state.selected_index).cloned().unwrap_or_default()
    }

    /// Remove all items.
    ///
    /// Apple: `removeAllItems`
    pub fn remove_all_items(&mut self) {
        let mut state = self.state.lock().unwrap();
        state.items.clear();
        state.selected_index = 0;
        drop(state);
        self.control.view_mut().set_needs_display();
    }

    /// Set the delegate for popup button events.
    ///
    /// Apple: `setTarget:action:` (we use delegate pattern)
    pub fn set_delegate(&mut self, delegate: Box<dyn NSPopUpButtonDelegate>) {
        *self.delegate.lock().unwrap() = Some(delegate);
    }

    // ── View Access ──────────────────────────────────────────────────

    pub fn view(&self) -> NSView {
        self.control.view().clone()
    }

    pub fn view_mut(&mut self) -> &mut NSView {
        self.control.view_mut()
    }

    pub fn view_clone(&self) -> NSView {
        self.control.view().clone()
    }
}
