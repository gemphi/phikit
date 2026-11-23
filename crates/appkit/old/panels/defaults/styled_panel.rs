//! `StyledPanel` trait and marker traits for type-safe panel styling.
//!
//! Implement `StyledPanel` on your panel struct to get automatic styling.
//! You only need to implement `build_content()` — styling is handled.

use crate::prelude::*;

use super::{PanelDefaults, PanelKind};

/// Trait for panels that use default styling.
///
/// Implement this on your panel struct to get automatic styling.
/// You only need to implement `build_content()` — styling is handled.
pub trait StyledPanel: Send + Sync {
    /// The kind of panel — determines default styling.
    fn panel_kind() -> PanelKind;

    /// Get the defaults for this panel type.
    fn defaults() -> PanelDefaults {
        match Self::panel_kind() {
            PanelKind::Sidebar => PanelDefaults::sidebar(),
            PanelKind::Inspector => PanelDefaults::inspector(),
            PanelKind::BottomPanel => PanelDefaults::project_browser(),
            PanelKind::Viewport => PanelDefaults::viewport(),
            PanelKind::ToolPalette => PanelDefaults::tool_palette(),
            _ => PanelDefaults::sidebar(),
        }
    }

    /// Apply default styling to a view.
    fn apply_defaults(view: &mut NSView) {
        Self::defaults().apply_to(view);
    }

    /// Create a view with defaults applied.
    fn create_view(rect: NSRect, name: &str) -> NSView {
        Self::defaults().create_view(rect, name)
    }

    /// Create a preconfigured stack view.
    fn create_stack(vertical: bool) -> NSStackView {
        Self::defaults().create_stack(vertical)
    }
}

/// Marker traits for specific panel types — use these for type safety.
pub trait SidebarPanel: StyledPanel {}
pub trait InspectorPanelTrait: StyledPanel {}
pub trait ProjectBrowserPanelTrait: StyledPanel {}
pub trait ViewportPanelTrait: StyledPanel {}

// Blanket implementations
impl<T: StyledPanel> SidebarPanel for T where T: Send + Sync {}
impl<T: StyledPanel> InspectorPanelTrait for T where T: Send + Sync {}
impl<T: StyledPanel> ProjectBrowserPanelTrait for T where T: Send + Sync {}
impl<T: StyledPanel> ViewportPanelTrait for T where T: Send + Sync {}
