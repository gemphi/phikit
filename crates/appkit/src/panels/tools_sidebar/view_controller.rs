//! # NSToolsSidebarViewController — Tool Palette Panel
//!
//! Implements the assembly and management logic for the manipulation tools sidebar.
//! Mirrors Apple's view controller pattern for modular utility toolbars.
//!
//! ## Apple Documentation
//! - [NSViewController](https://developer.apple.com/documentation/appkit/nsviewcontroller)
//! - [NSView](https://developer.apple.com/documentation/appkit/nsview)
//! - [NSButton](https://developer.apple.com/documentation/appkit/nsbutton)
//!
//! ## See Also
//! - [NSToolbar](https://developer.apple.com/documentation/appkit/nstoolbar)
//! - [NSSegmentedControl](https://developer.apple.com/documentation/appkit/nssegmentedcontrol)

use super::model::*;
use crate::graphics::image::NSImage;
use crate::prelude::*;
use crate::sidebar::{NSSidebar, NSSidebarConfig, NSSidebarItem};
use crate::layout::spacing;
use std::sync::Arc;

pub struct NSToolsSidebarViewController {
    pub view: NSView,
    sidebar: NSSidebar,
    tools: Vec<NSToolDefinition>,
}

impl NSToolsSidebarViewController {
    /// Create a new tools sidebar view controller.
    ///
    /// # Arguments
    /// * `rect` - The frame rectangle for the sidebar
    /// * `tools` - The list of tool definitions to display
    /// * `icon_resolver` - A function to resolve icon names to NSImage instances
    pub fn new(
        rect: NSRect,
        tools: Vec<NSToolDefinition>,
        icon_resolver: Arc<dyn Fn(&str, f32, f32) -> Option<Arc<NSImage>> + Send + Sync>,
    ) -> Self {
        // Create sidebar with custom configuration
        let config = NSSidebarConfig {
            button_size: 40.0,
            icon_size: 32.0,
            item_spacing: spacing::gap::SIDEBAR,
            label_height: 12.0,
            label_font_size: 9.0,
            top_margin: spacing::padding::SIDEBAR,
        };

        let mut sidebar = NSSidebar::new_with_config(rect, config);

        // Add items to sidebar
        for tool in &tools {
            let icon = icon_resolver(&tool.icon_name, 32.0, 32.0);
            let item = NSSidebarItem::new(tool.id, tool.name, tool.label, icon);
            sidebar.add_item(item);
        }

        // Set initial selection
        if let Some(first_tool) = tools.first() {
            sidebar.set_selected_item(first_tool.id);
        }

        let view = sidebar.view_clone();

        Self {
            view,
            sidebar,
            tools,
        }
    }

    /// Set the active tool by ID
    pub fn set_active_tool(&mut self, tool_id: u32) {
        if self.tools.iter().any(|t| t.id == tool_id) {
            self.sidebar.set_selected_item(tool_id);
        }
    }

    /// Get the currently active tool ID
    pub fn active_tool_id(&self) -> Option<u32> {
        self.sidebar.selected_item()
    }

    /// Get the list of tools
    pub fn tools(&self) -> &[NSToolDefinition] {
        &self.tools
    }
}
