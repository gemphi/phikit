//! Panel Traits — framework-level interfaces for editor panels.
//!
//! These traits define the contract between the workspace layout system
//! and individual panel implementations. Apps implement these on their
//! panel wrappers to participate in the workspace.

use crate::NSView;

/// Editor panel trait — every workspace region implements this.
///
/// Defines the standard interface for workspace panels to interact with the app.
/// Handles view lifecycle, selection updates, and layout properties.
pub trait EditorPanel: Send + Sync {
    /// Returns the underlying NSView for layout and rendering.
    fn view(&self) -> &NSView;

    /// Returns a mutable reference to the underlying NSView.
    fn view_mut(&mut self) -> &mut NSView;

    /// Convenience method to clone the view handle.
    fn view_clone(&self) -> NSView {
        self.view().clone()
    }

    /// Notifies the panel that the global entity selection has changed.
    fn on_selection_changed(&mut self, entity_id: Option<u64>);

    /// Returns the unique name of the panel for registry lookup.
    fn name(&self) -> &str;

    /// Triggers a data refresh from the underlying data source or scene.
    fn reload_data(&mut self) {}

    /// Returns the minimum thickness required by the panel for layout.
    fn min_size(&self) -> f32 {
        100.0
    }

    /// Returns the maximum thickness allowed for the panel.
    fn max_size(&self) -> f32 {
        f32::INFINITY
    }

    /// Returns the preferred thickness for initial layout passes.
    fn preferred_size(&self) -> f32 {
        320.0
    }

    /// Returns whether the panel can be collapsed by the user.
    fn is_collapsible(&self) -> bool {
        true
    }

    /// Returns the current collapse state of the panel.
    fn is_collapsed(&self) -> bool {
        self.view().is_hidden()
    }

    /// Updates the collapse state and visibility of the panel.
    fn set_collapsed(&mut self, collapsed: bool) {
        self.view_mut().set_hidden(collapsed);
    }

    /// Helper for downcasting to specific panel implementations.
    fn as_any(&self) -> &dyn std::any::Any;

    /// Helper for mutable downcasting to specific implementations.
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

/// Panel configuration metadata — panels implement this for display info.
pub trait PanelConfig {
    fn title(&self) -> &str;
    fn icon_name(&self) -> &str;
    fn default_size(&self) -> (f32, f32);
    fn background_color(&self) -> (f32, f32, f32, f32);
    fn is_collapsible(&self) -> bool;
}
