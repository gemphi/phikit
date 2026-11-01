//! # NSComboBox — Combo Box Control
//!
//! A control that combines a text field with a drop-down list of selectable values.
//!
//! ## Apple Documentation
//! - [NSComboBox](https://developer.apple.com/documentation/appkit/nscombobox)
//! - [NSComboBoxDelegate](https://developer.apple.com/documentation/appkit/nscomboboxdelegate)
//!
//! ## Usage
//!
//! ```rust,ignore
//! let mut combo = NSComboBox::new(frame);
//! combo.add_item("Option 1");
//! combo.add_item("Option 2");
//! combo.add_item("Option 3");
//! combo.set_delegate(Box::new(MyComboBoxDelegate));
//! ```
//!
//! ## Delegate Pattern
//!
//! ```rust,ignore
//! struct MyDelegate;
//!
//! impl NSComboBoxDelegate for MyDelegate {
//!     fn text_did_change(&mut self, text: &str) {
//!         println!("Text changed: {}", text);
//!     }
//!     
//!     fn did_select_item(&mut self, index: usize) {
//!         println!("Selected item at index: {}", index);
//!     }
//! }
//! ```
//!
//! ## See Also
//! - [NSComboBox](https://developer.apple.com/documentation/appkit/nscombobox)
//! - [NSControl](https://developer.apple.com/documentation/appkit/nscontrol)

pub mod delegate;

pub use delegate::NSComboBoxDelegate;

use crate::control::NSControl;
use crate::graphics::NSRect;
use crate::view::NSView;
use delegate::{ComboBoxState, NSComboBoxViewDelegate};
use std::sync::{Arc, Mutex};

/// Combo box control combining text input with a dropdown list.
///
/// Apple: [NSComboBox](https://developer.apple.com/documentation/appkit/nscombobox)
#[derive(Clone)]
pub struct NSComboBox {
    control: NSControl,
    state: Arc<Mutex<ComboBoxState>>,
    delegate: Arc<Mutex<Option<Box<dyn NSComboBoxDelegate>>>>,
}

impl NSComboBox {
    /// Create a new combo box.
    ///
    /// Apple: `[[NSComboBox alloc] initWithFrame:]`
    pub fn new(frame: NSRect) -> Self {
        let state = Arc::new(Mutex::new(ComboBoxState::default()));
        let delegate = Arc::new(Mutex::new(None));

        let mut control = NSControl::new(frame);
        control.view_mut().set_delegate(NSComboBoxViewDelegate {
            state: state.clone(),
            delegate: delegate.clone(),
        });

        Self {
            control,
            state,
            delegate,
        }
    }

    /// Add an item to the dropdown list.
    ///
    /// Apple: `addItemWithObjectValue:`
    pub fn add_item(&mut self, item: &str) {
        self.state.lock().unwrap().items.push(item.to_string());
    }

    /// Remove an item at the specified index.
    ///
    /// Apple: `removeItemAtIndex:`
    pub fn remove_item(&mut self, index: usize) {
        let mut state = self.state.lock().unwrap();
        if index < state.items.len() {
            state.items.remove(index);
            // Clear selection if removed item was selected
            if state.selected_index == Some(index) {
                state.selected_index = None;
            } else if let Some(sel_idx) = state.selected_index {
                // Adjust selection index if it was after the removed item
                if sel_idx > index {
                    state.selected_index = Some(sel_idx - 1);
                }
            }
        }
    }

    /// Remove all items from the dropdown list.
    ///
    /// Apple: `removeAllItems`
    pub fn remove_all_items(&mut self) {
        let mut state = self.state.lock().unwrap();
        state.items.clear();
        state.selected_index = None;
    }

    /// Get the number of items in the dropdown list.
    ///
    /// Apple: `numberOfItems`
    pub fn number_of_items(&self) -> usize {
        self.state.lock().unwrap().items.len()
    }

    /// Get the item at the specified index.
    ///
    /// Apple: `itemObjectValueAtIndex:`
    pub fn item_at_index(&self, index: usize) -> Option<String> {
        self.state
            .lock()
            .unwrap()
            .items
            .get(index)
            .map(|s| s.clone())
    }

    /// Set the text field value.
    ///
    /// Apple: `setStringValue:`
    pub fn set_text(&mut self, text: &str) {
        self.state.lock().unwrap().text = text.to_string();
    }

    /// Get the current text field value.
    ///
    /// Apple: `stringValue`
    pub fn text(&self) -> String {
        self.state.lock().unwrap().text.clone()
    }

    /// Set the placeholder text.
    ///
    /// Apple: `setPlaceholderString:`
    pub fn set_placeholder_string(&mut self, placeholder: &str) {
        self.state.lock().unwrap().placeholder = placeholder.to_string();
    }

    /// Select an item by index.
    ///
    /// Apple: `selectItemAtIndex:`
    pub fn select_item_at_index(&mut self, index: usize) {
        let mut state = self.state.lock().unwrap();
        if index < state.items.len() {
            state.selected_index = Some(index);
            state.text = state.items[index].clone();
        }
    }

    /// Get the index of the selected item.
    ///
    /// Apple: `indexOfSelectedItem`
    pub fn index_of_selected_item(&self) -> Option<usize> {
        self.state.lock().unwrap().selected_index
    }

    /// Set whether the dropdown is currently open.
    pub fn set_is_open(&mut self, is_open: bool) {
        self.state.lock().unwrap().is_open = is_open;
    }

    /// Check if the dropdown is currently open.
    pub fn is_open(&self) -> bool {
        self.state.lock().unwrap().is_open
    }

    /// Set the delegate for combo box events.
    ///
    /// Apple: `setDelegate:`
    pub fn set_delegate(&mut self, delegate: Box<dyn NSComboBoxDelegate>) {
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
