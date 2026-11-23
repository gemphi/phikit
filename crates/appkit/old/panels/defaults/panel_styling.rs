//! Panel type classification and default styling configuration.
//!
//! Defines `PanelKind` and `PanelDefaults` — the core types that determine
//! how each panel type is styled (background, corner radius, sizing, insets).

use crate::glass::GlassStyle;
use crate::prelude::*;
use crate::theme::keys;

/// Panel type classification — each has distinct default styling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PanelKind {
    /// Left/right sidebar (hierarchy, layers)
    Sidebar,
    /// Right-side property inspector
    Inspector,
    /// Bottom asset browser/project panel
    BottomPanel,
    /// Main viewport area
    Viewport,
    /// Floating tool palette
    ToolPalette,
    /// Modal dialog content
    Dialog,
    /// Generic container
    Container,
}

/// Default styling configuration for a panel kind.
#[derive(Debug, Clone)]
pub struct PanelDefaults {
    pub kind: PanelKind,
    /// The panel's glass, or `None` for a solid background.
    pub glass: Option<GlassStyle>,
    pub background_key: &'static str,
    pub fallback_bg: NSColor,
    pub corner_radius_key: &'static str,
    pub fallback_radius: f32,
    pub clips_to_bounds: bool,
    pub autoresizing: NSAutoresizingMask,
    pub min_size: f32,
    pub preferred_size: f32,
    pub max_size: f32,
    pub content_insets: NSEdgeInsets,
    pub spacing: f32,
}

impl PanelDefaults {
    /// Sidebar defaults (hierarchy, layers panels)
    pub fn sidebar() -> Self {
        Self {
            kind: PanelKind::Sidebar,
            glass: Some(GlassStyle::Regular),
            background_key: keys::CLR_BG_PANEL,
            fallback_bg: NSColor::bg_panel(),
            corner_radius_key: keys::RADIUS_PANEL,
            fallback_radius: 12.0,
            clips_to_bounds: true,
            autoresizing: NSAutoresizingMask::WIDTH_SIZABLE | NSAutoresizingMask::HEIGHT_SIZABLE,
            min_size: 200.0,
            preferred_size: 320.0,
            max_size: 450.0,
            content_insets: NSEdgeInsets::uniform(NSLayoutConstants::MARGIN),
            spacing: NSLayoutConstants::MARGIN_XSMALL,
        }
    }

    /// Inspector panel defaults (right-side properties)
    pub fn inspector() -> Self {
        Self {
            kind: PanelKind::Inspector,
            glass: Some(GlassStyle::Regular),
            background_key: keys::CLR_BG_PANEL,
            fallback_bg: NSColor::bg_panel(),
            corner_radius_key: keys::RADIUS_PANEL,
            fallback_radius: 12.0,
            clips_to_bounds: true,
            autoresizing: NSAutoresizingMask::WIDTH_SIZABLE | NSAutoresizingMask::HEIGHT_SIZABLE,
            min_size: 250.0,
            preferred_size: 320.0,
            max_size: 450.0,
            content_insets: NSEdgeInsets::uniform(NSLayoutConstants::MARGIN),
            spacing: NSLayoutConstants::MARGIN_XSMALL,
        }
    }

    /// Project browser defaults (bottom asset panel)
    pub fn project_browser() -> Self {
        Self {
            kind: PanelKind::BottomPanel,
            glass: Some(GlassStyle::Regular),
            background_key: keys::CLR_BG_PANEL,
            fallback_bg: NSColor::bg_panel(),
            corner_radius_key: keys::RADIUS_PANEL,
            fallback_radius: 12.0,
            clips_to_bounds: false,
            autoresizing: NSAutoresizingMask::WIDTH_SIZABLE | NSAutoresizingMask::HEIGHT_SIZABLE,
            min_size: 120.0,
            preferred_size: 200.0,
            max_size: 400.0,
            content_insets: NSEdgeInsets::uniform(NSLayoutConstants::MARGIN),
            spacing: NSLayoutConstants::MARGIN,
        }
    }

    /// Viewport defaults (center 3D view)
    pub fn viewport() -> Self {
        Self {
            kind: PanelKind::Viewport,
            glass: None,
            background_key: keys::CLR_BG_PANEL,
            fallback_bg: NSColor::bg_panel(),
            corner_radius_key: keys::RADIUS_PANEL,
            fallback_radius: 12.0,
            clips_to_bounds: true,
            autoresizing: NSAutoresizingMask::WIDTH_SIZABLE | NSAutoresizingMask::HEIGHT_SIZABLE,
            min_size: 300.0,
            preferred_size: 800.0,
            max_size: f32::INFINITY,
            content_insets: NSEdgeInsets::uniform(0.0),
            spacing: 0.0,
        }
    }

    /// Tool palette defaults (floating tool strip)
    pub fn tool_palette() -> Self {
        Self {
            kind: PanelKind::ToolPalette,
            glass: Some(GlassStyle::Regular),
            background_key: keys::CLR_BG_PANEL,
            fallback_bg: NSColor::bg_panel(),
            corner_radius_key: keys::RADIUS_CONTROL,
            fallback_radius: 6.0,
            clips_to_bounds: true,
            autoresizing: NSAutoresizingMask::HEIGHT_SIZABLE,
            min_size: 40.0,
            preferred_size: 44.0,
            max_size: 60.0,
            content_insets: NSEdgeInsets::new(4.0, 4.0, 4.0, 4.0),
            spacing: NSLayoutConstants::MARGIN_SMALL,
        }
    }

    /// Apply all default styling to a view.
    pub fn apply_to(&self, view: &mut NSView) {
        view.set_background_color(Some(crate::theme::color(self.background_key)));
        view.set_corner_radius(crate::theme::registry::get_geometry(self.corner_radius_key));
        view.set_clips_to_bounds(self.clips_to_bounds);
        view.set_autoresizing_mask(self.autoresizing);
        view.set_glass(self.glass.is_some());
        view.set_glass_style(self.glass.unwrap_or_default());
    }

    /// Create a new view with these defaults applied.
    pub fn create_view(&self, rect: NSRect, name: &str) -> NSView {
        let mut view = NSView::new(rect).with_name(name);
        self.apply_to(&mut view);
        view
    }

    /// Create a stack view preconfigured for this panel type.
    pub fn create_stack(&self, vertical: bool) -> NSStackView {
        let mut stack = if vertical {
            NSStackView::vertical()
        } else {
            NSStackView::horizontal()
        };
        stack.set_spacing(self.spacing);
        stack.set_edge_insets(self.content_insets);
        stack
    }
}
