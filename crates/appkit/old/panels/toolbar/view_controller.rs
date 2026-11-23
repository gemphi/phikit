//! # NSToolbarViewController
//!
//! Configures NSToolbar component for the editor.
//!
//! ## Apple Pattern
//! Follows Apple's [`NSToolbar`](https://developer.apple.com/documentation/appkit/nstoolbar) 
//! and [`NSViewController`](https://developer.apple.com/documentation/appkit/nsviewcontroller) patterns.
//!
//! The controller is a thin wrapper that configures the toolbar component.
//! All UI logic is handled by NSToolbar itself.
//!
//! ## Architecture
//! ```
//! NSToolbarViewController (thin controller)
//!   └── NSToolbar (view component - handles all UI)
//!       ├── NSToolbarItem (play button)
//!       ├── NSToolbarItem (pause button)
//!       ├── Flexible space
//!       └── NSToolbarItem (search field)
//! ```
//!
//! ## See Also
//! - [NSToolbar](https://developer.apple.com/documentation/appkit/nstoolbar)
//! - [NSToolbarItem](https://developer.apple.com/documentation/appkit/nstoolbaritem)
//! - [NSViewController](https://developer.apple.com/documentation/appkit/nsviewcontroller)

use super::model::*;
use crate::prelude::*;
use crate::toolbar::{NSToolbar, NSToolbarItem};

/// Toolbar view controller — configures NSToolbar for the editor.
pub struct NSToolbarViewController {
    pub toolbar: NSToolbar,
    pub state: NSToolbarState,
    cached_view: Option<NSView>,
}

impl NSToolbarViewController {
    /// Create a new toolbar view controller.
    /// Builds toolbar manually using generic NSToolbar APIs.
    pub fn new(width: f32, _height: f32) -> Self {
        let state = NSToolbarState::default();
        let toolbar = Self::build_toolbar(&state);
        let cached_view = Some(toolbar.build_view(width));
        Self { toolbar, state, cached_view }
    }

    /// Update the playback state.
    pub fn set_playback(&mut self, playback: NSPlaybackState) {
        self.state.playback = playback;
        self.rebuild();
    }

    /// Set the editor mode.
    pub fn set_editor_mode(&mut self, mode: NSEditorMode) {
        self.state.editor_mode = mode;
        self.rebuild();
    }

    /// Toggle panel visibility.
    pub fn toggle_sidebar(&mut self) {
        self.state.panel_toggles.sidebar_visible = !self.state.panel_toggles.sidebar_visible;
        self.rebuild();
    }

    pub fn toggle_bottom(&mut self) {
        self.state.panel_toggles.bottom_visible = !self.state.panel_toggles.bottom_visible;
        self.rebuild();
    }

    pub fn toggle_inspector(&mut self) {
        self.state.panel_toggles.inspector_visible = !self.state.panel_toggles.inspector_visible;
        self.rebuild();
    }

    /// Set the search query.
    pub fn set_search_query(&mut self, query: &str) {
        self.state.search_query = query.to_string();
    }

    /// Get current panel toggle state.
    pub fn panel_toggles(&self) -> &NSPanelToggles {
        &self.state.panel_toggles
    }

    /// Rebuild the toolbar from current state.
    /// Builds toolbar manually using generic NSToolbar APIs.
    pub fn rebuild(&mut self) {
        self.toolbar = Self::build_toolbar(&self.state);
        // Rebuild the cached view with the current width
        if let Some(ref view) = self.cached_view {
            let width = view.frame().width();
            self.cached_view = Some(self.toolbar.build_view(width));
        }
    }

    /// Build toolbar from state using generic NSToolbar APIs.
    fn build_toolbar(state: &NSToolbarState) -> NSToolbar {
        let mut toolbar = NSToolbar::new("EditorToolbar");
        toolbar.set_display_mode(crate::toolbar::DisplayMode::IconOnly);

        // Add playback controls
        let play_btn = NSButton::new(NSRect::sized(32.0, 32.0))
            .with_title("Play");
        toolbar.add_item(NSToolbarItem::button("play", "Play", play_btn.view_clone()));

        let pause_btn = NSButton::new(NSRect::sized(32.0, 32.0))
            .with_title("Pause");
        toolbar.add_item(NSToolbarItem::button("pause", "Pause", pause_btn.view_clone()));

        let stop_btn = NSButton::new(NSRect::sized(32.0, 32.0))
            .with_title("Stop");
        toolbar.add_item(NSToolbarItem::button("stop", "Stop", stop_btn.view_clone()));

        // Add flexible space
        toolbar.add_flexible_space();

        // Add panel toggles
        if state.panel_toggles.sidebar_visible {
            let sidebar_btn = NSButton::new(NSRect::sized(32.0, 32.0))
                .with_title("Sidebar");
            toolbar.add_item(NSToolbarItem::button("sidebar", "Sidebar", sidebar_btn.view_clone()));
        }

        if state.panel_toggles.inspector_visible {
            let inspector_btn = NSButton::new(NSRect::sized(32.0, 32.0))
                .with_title("Inspector");
            toolbar.add_item(NSToolbarItem::button("inspector", "Inspector", inspector_btn.view_clone()));
        }

        toolbar
    }
}

impl EditorPanel for NSToolbarViewController {
    fn view(&self) -> &NSView {
        self.cached_view.as_ref().expect("Toolbar view not initialized")
    }
    fn view_mut(&mut self) -> &mut NSView {
        self.cached_view.as_mut().expect("Toolbar view not initialized")
    }
    fn on_selection_changed(&mut self, _entity_id: Option<u64>) {
        // Toolbar doesn't react to selection changes
    }
    fn reload_data(&mut self) {
        // No data to reload
    }
    fn name(&self) -> &str { "Toolbar" }
    fn min_size(&self) -> f32 { 44.0 }
    fn preferred_size(&self) -> f32 { 52.0 }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

use crate::panels::EditorPanel;
