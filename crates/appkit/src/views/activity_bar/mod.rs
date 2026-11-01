//! # NSActivityBar — Vertical Icon Navigation Bar
//!
//! A vertical bar of icon buttons for switching between major app modes.
//! Similar to VS Code's activity bar or Xcode's navigator sidebar selector.
//!
//! ## Apple Equivalent
//! While AppKit doesn't have a direct equivalent, this combines concepts from:
//! - [`NSSegmentedControl`](https://developer.apple.com/documentation/appkit/nssegmentedcontrol) - Grouped button selection
//! - [`NSToolbar`](https://developer.apple.com/documentation/appkit/nstoolbar) - Icon-based navigation
//! - Custom sidebar navigation patterns in Xcode and other Apple apps
//!
//! ## Features
//! - Vertical icon button layout
//! - Automatic selection state management
//! - Hover effects
//! - Top/middle/bottom grouping
//! - Icon-only or icon+label modes
//!
//! ## Usage
//!
//! ```rust,ignore
//! let mut activity_bar = NSActivityBar::new(rect);
//! activity_bar.add_item(NSActivityBarItem::new("hierarchy", IconId::Hierarchy));
//! activity_bar.add_item(NSActivityBarItem::new("project", IconId::Project));
//! activity_bar.set_selected("hierarchy");
//! ```
//!
//! ## See Also
//! - [NSSegmentedControl](https://developer.apple.com/documentation/appkit/nssegmentedcontrol)
//! - [NSToolbar](https://developer.apple.com/documentation/appkit/nstoolbar)

use crate::prelude::*;
use crate::svg_icons::IconId;
use crate::theme::{self, keys};
use std::sync::Arc;

/// Position group for activity bar items
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityBarGroup {
    Top,
    Middle,
    Bottom,
}

/// A single item in the activity bar
#[derive(Clone)]
pub struct NSActivityBarItem {
    pub id: String,
    pub icon: IconId,
    pub label: String,
    pub group: ActivityBarGroup,
}

impl NSActivityBarItem {
    pub fn new(id: &str, icon: IconId) -> Self {
        Self {
            id: id.to_string(),
            icon,
            label: id.to_string(),
            group: ActivityBarGroup::Middle,
        }
    }

    pub fn with_label(mut self, label: &str) -> Self {
        self.label = label.to_string();
        self
    }

    pub fn with_group(mut self, group: ActivityBarGroup) -> Self {
        self.group = group;
        self
    }
}

/// Vertical activity bar component
pub struct NSActivityBar {
    view: NSView,
    items: Vec<NSActivityBarItem>,
    selected_id: Option<String>,
    on_selection_changed: Option<Arc<dyn Fn(&str) + Send + Sync>>,
}

impl NSActivityBar {
    /// Create a new activity bar
    pub fn new(rect: NSRect) -> Self {
        let mut view = NSView::new(rect).with_name("NSActivityBar");
        view.set_background_color(None);
        view.set_clips_to_bounds(true);
        
        Self {
            view,
            items: Vec::new(),
            selected_id: None,
            on_selection_changed: None,
        }
    }

    /// Add an item to the activity bar
    pub fn add_item(&mut self, item: NSActivityBarItem) {
        self.items.push(item);
        self.rebuild();
    }

    /// Set the selected item by ID
    pub fn set_selected(&mut self, id: &str) {
        self.selected_id = Some(id.to_string());
        self.rebuild();
        
        if let Some(ref callback) = self.on_selection_changed {
            callback(id);
        }
    }

    /// Get the currently selected item ID
    pub fn selected(&self) -> Option<&str> {
        self.selected_id.as_deref()
    }

    /// Set callback for selection changes
    pub fn set_on_selection_changed<F>(&mut self, callback: F)
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_selection_changed = Some(Arc::new(callback));
    }

    /// Get the underlying view
    pub fn view(&self) -> &NSView {
        &self.view
    }

    /// Get mutable view reference
    pub fn view_mut(&mut self) -> &mut NSView {
        &mut self.view
    }

    /// Rebuild the activity bar layout
    fn rebuild(&mut self) {
        self.view.remove_all_subviews();

        let icon_size = crate::layout_constants::icon_size_lg();
        let icon_gap = crate::layout_constants::activity_bar_icon_gap();
        let margin_y = crate::layout_constants::activity_bar_icon_inset_y();
        let bar_w = self.view.frame().width();
        let bar_h = self.view.frame().height();
        let icon_x = (bar_w - icon_size) / 2.0;

        // Partition items by group
        let top_items: Vec<&NSActivityBarItem> = self.items.iter()
            .filter(|i| i.group == ActivityBarGroup::Top)
            .collect();
        let mid_items: Vec<&NSActivityBarItem> = self.items.iter()
            .filter(|i| i.group == ActivityBarGroup::Middle)
            .collect();
        let bot_items: Vec<&NSActivityBarItem> = self.items.iter()
            .filter(|i| i.group == ActivityBarGroup::Bottom)
            .collect();

        // Top group (pinned to top)
        let mut y = margin_y;
        for item in &top_items {
            let btn = self.build_button(NSRect::new(icon_x, y, icon_size, icon_size), item);
            self.view.add_subview(btn);
            y += icon_size + icon_gap;
        }

        // Bottom group (pinned to bottom)
        let mut bot_y = bar_h - margin_y - icon_size;
        for item in bot_items.iter().rev() {
            let btn = self.build_button(NSRect::new(icon_x, bot_y, icon_size, icon_size), item);
            self.view.add_subview(btn);
            bot_y -= icon_size + icon_gap;
        }

        // Middle group (centered in remaining space)
        let mid_total_h = if mid_items.is_empty() {
            0.0
        } else {
            icon_size * mid_items.len() as f32 + icon_gap * (mid_items.len() - 1) as f32
        };
        let mid_start = (bar_h - mid_total_h) / 2.0;

        for (i, item) in mid_items.iter().enumerate() {
            let iy = mid_start + (icon_size + icon_gap) * i as f32;
            let btn = self.build_button(NSRect::new(icon_x, iy, icon_size, icon_size), item);
            self.view.add_subview(btn);
        }
    }

    /// Build a single icon button
    fn build_button(&self, frame: NSRect, item: &NSActivityBarItem) -> NSView {
        let is_active = self.selected_id.as_ref().map_or(false, |id| id == &item.id);
        
        let mut btn = NSView::new(frame)
            .activity_bar_style(is_active)
            .with_name(&item.id);

        // Add icon
        let icon_size = crate::layout_constants::icon_size_md();
        if let Some(icon) = crate::svg_icons::get_sized(item.icon.clone(), icon_size, icon_size) {
            let mut iv = crate::NSImageView::new(NSRect::new(
                (frame.width() - icon_size) / 2.0,
                (frame.height() - icon_size) / 2.0,
                icon_size,
                icon_size,
            ));
            iv.set_image(Some(icon));
            btn.add_subview(iv.view_clone());
        }

        btn
    }
}
