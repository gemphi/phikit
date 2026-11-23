//! # NSSidebar — Vertical Tool Palette Component
//!
//! A generic vertical sidebar component for displaying a list of tool buttons with icons and labels.
//! Commonly used for tool palettes, mode selectors, and action bars.
//!
//! ## Apple Documentation
//! Custom component (no direct Apple equivalent). Inspired by tool palettes in various Apple applications.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use rust_appkit::views::sidebar::{NSSidebar, NSSidebarItem};
//!
//! let mut sidebar = NSSidebar::new(NSRect::new(0.0, 0.0, 60.0, 400.0));
//!
//! // Add items with icons and labels
//! sidebar.add_item(NSSidebarItem::new(
//!     0,
//!     "Select",
//!     "Select",
//!     Some(select_icon),
//! ));
//!
//! sidebar.add_item(NSSidebarItem::new(
//!     1,
//!     "Move",
//!     "Move",
//!     Some(move_icon),
//! ));
//!
//! // Set active item
//! sidebar.set_selected_item(0);
//!
//! // Get selected item
//! if let Some(id) = sidebar.selected_item() {
//!     println!("Selected item: {}", id);
//! }
//! ```
//!
//! ## Features
//! - Vertical layout of icon buttons with labels
//! - Configurable item list
//! - Selection state management
//! - Customizable spacing and sizing
//! - Icon and label support
//!
//! ## See Also
//! - [NSButton](https://developer.apple.com/documentation/appkit/nsbutton)
//! - [NSView](https://developer.apple.com/documentation/appkit/nsview)

use crate::graphics::image::NSImage;
use crate::prelude::*;
use crate::theme::{self, keys};
use crate::layout::{spacing, hierarchy};
use std::sync::{Arc, Mutex};

/// Configuration for sidebar layout and appearance
#[derive(Debug, Clone)]
pub struct NSSidebarConfig {
    /// Size of the icon buttons
    pub button_size: f32,
    /// Size of the icons within buttons
    pub icon_size: f32,
    /// Spacing between items
    pub item_spacing: f32,
    /// Height of the label text
    pub label_height: f32,
    /// Font size for labels
    pub label_font_size: f32,
    /// Top margin
    pub top_margin: f32,
}

impl Default for NSSidebarConfig {
    fn default() -> Self {
        Self {
            button_size: 40.0,
            icon_size: 32.0,
            item_spacing: spacing::gap::SIDEBAR,
            label_height: 12.0,
            label_font_size: 9.0,
            top_margin: spacing::padding::SIDEBAR,
        }
    }
}

/// Represents a single item in the sidebar
#[derive(Clone)]
pub struct NSSidebarItem {
    /// Unique identifier for this item
    pub id: u32,
    /// Internal name for the item
    pub name: String,
    /// Display label for the item
    pub label: String,
    /// Optional icon image
    pub icon: Option<Arc<NSImage>>,
    /// Button view
    button: NSButton,
    /// Label view
    label_view: NSTextField,
}

impl NSSidebarItem {
    /// Create a new sidebar item
    pub fn new(id: u32, name: impl Into<String>, label: impl Into<String>, icon: Option<Arc<NSImage>>) -> Self {
        let button = NSButton::new(NSRect::zero());
        let label_view = NSTextField::label("");
        
        Self {
            id,
            name: name.into(),
            label: label.into(),
            icon,
            button,
            label_view,
        }
    }
}

/// Internal state for the sidebar
struct NSSidebarState {
    items: Vec<NSSidebarItem>,
    selected_item_id: Option<u32>,
    config: NSSidebarConfig,
}

/// A vertical sidebar component for tool palettes and mode selectors
pub struct NSSidebar {
    view: NSView,
    state: Arc<Mutex<NSSidebarState>>,
}

impl NSSidebar {
    /// Create a new sidebar with default configuration
    pub fn new(rect: NSRect) -> Self {
        Self::new_with_config(rect, NSSidebarConfig::default())
    }

    /// Create a new sidebar with custom configuration
    pub fn new_with_config(rect: NSRect, config: NSSidebarConfig) -> Self {
        let mut view = NSView::new(rect).with_name("Sidebar");
        view.set_background_color(Some(hierarchy::bg_color("sidebar")));

        let state = Arc::new(Mutex::new(NSSidebarState {
            items: Vec::new(),
            selected_item_id: None,
            config,
        }));

        Self { view, state }
    }

    /// Add an item to the sidebar
    pub fn add_item(&mut self, mut item: NSSidebarItem) {
        let mut state = self.state.lock().unwrap();
        let config = &state.config;
        
        let rect = self.view.frame();
        let x = (rect.width() - config.button_size) / 2.0;
        
        // Calculate y position based on existing items
        let y = config.top_margin + (state.items.len() as f32) * 
                (config.button_size + config.item_spacing + config.label_height);

        // Configure button
        item.button.view_mut().set_frame(NSRect::new(x, y, config.button_size, config.button_size));
        if let Some(icon) = &item.icon {
            item.button.set_image(Some(icon.clone()));
        }
        item.button.set_bezel_style(NSBezelStyle::Inline);
        item.button.set_title("");

        let is_selected = state.selected_item_id == Some(item.id);
        item.button.view_mut().set_background_color(Some(if is_selected {
            hierarchy::active_color(hierarchy::bg_color("button"))
        } else {
            hierarchy::bg_color("button")
        }));
        item.button.view_mut().set_corner_radius(crate::theme::ThemeToken::RADIUS_BUTTON());

        // Configure label
        item.label_view.set_string_value(&item.label);
        item.label_view.control_mut().view_mut().set_frame(NSRect::new(
            0.0,
            y + config.button_size + 2.0,
            rect.width(),
            config.label_height,
        ));
        item.label_view.set_text_alignment(NSTextAlignment::Center);
        item.label_view.set_font_size(config.label_font_size);
        item.label_view.set_text_color(theme::color(keys::CLR_TEXT_TERTIARY));

        // Add to view
        self.view.add_subview(item.button.view_clone());
        self.view.add_subview(item.label_view.view_clone());

        // Store item
        state.items.push(item);
    }

    /// Set the selected item by ID
    pub fn set_selected_item(&mut self, item_id: u32) {
        let mut state = self.state.lock().unwrap();
        
        // Check if item exists
        if !state.items.iter().any(|item| item.id == item_id) {
            return;
        }

        state.selected_item_id = Some(item_id);

        // Update button backgrounds
        for item in &mut state.items {
            let is_selected = item.id == item_id;
            item.button.view_mut().set_background_color(Some(if is_selected {
                hierarchy::active_color(hierarchy::bg_color("button"))
            } else {
                hierarchy::bg_color("button")
            }));
        }
    }

    /// Get the currently selected item ID
    pub fn selected_item(&self) -> Option<u32> {
        self.state.lock().unwrap().selected_item_id
    }

    /// Get all items
    pub fn items(&self) -> Vec<NSSidebarItem> {
        self.state.lock().unwrap().items.clone()
    }

    /// Get the view
    pub fn view(&self) -> &NSView {
        &self.view
    }

    /// Get a mutable reference to the view
    pub fn view_mut(&mut self) -> &mut NSView {
        &mut self.view
    }

    /// Get a cloned view
    pub fn view_clone(&self) -> NSView {
        self.view.clone()
    }
}
