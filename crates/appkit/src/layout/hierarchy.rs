//! # Visual Hierarchy
//!
//! Provide theme-based shading for visual depth and hierarchy.
//! Each layer get a mathematically calculated shade of the current theme.

use crate::graphics::NSColor;
use crate::theme;

// ═══════════════════════════════════════════════════════════════════════
// Layer Hierarchy (from back to front)
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    /// Window background (deepest layer)
    Window = 0,
    /// Content area background
    Content = 1,
    /// Panel background
    Panel = 2,
    /// Sidebar background
    Sidebar = 3,
    /// Control background (buttons, fields)
    Control = 4,
    /// Elevated control (hover, active)
    ControlElevated = 5,
    /// Overlay (menus, popovers)
    Overlay = 6,
    /// Modal (dialogs, sheets)
    Modal = 7,
}

impl Layer {
    /// Get the depth multiplier for this layer (0.0 = darkest, 1.0 = lightest)
    pub fn depth_multiplier(self) -> f32 {
        match self {
            Layer::Window => 0.0,
            Layer::Content => 0.15,
            Layer::Panel => 0.25,
            Layer::Sidebar => 0.20,
            Layer::Control => 0.35,
            Layer::ControlElevated => 0.45,
            Layer::Overlay => 0.50,
            Layer::Modal => 0.60,
        }
    }

    /// Get the background color for this layer based on current theme
    pub fn background_color(self) -> NSColor {
        let base = theme::color(theme::keys::CLR_BG_WINDOW);
        let multiplier = self.depth_multiplier();
        
        // In dark mode, lighten; in light mode, darken
        let is_dark = base.r < 0.5;
        
        if is_dark {
            // Dark mode: add brightness
            NSColor::new(
                (base.r + multiplier * 0.15).min(1.0),
                (base.g + multiplier * 0.15).min(1.0),
                (base.b + multiplier * 0.15).min(1.0),
                base.a,
            )
        } else {
            // Light mode: reduce brightness
            NSColor::new(
                (base.r - multiplier * 0.08).max(0.0),
                (base.g - multiplier * 0.08).max(0.0),
                (base.b - multiplier * 0.08).max(0.0),
                base.a,
            )
        }
    }

    /// Get the border color for this layer
    pub fn border_color(self) -> NSColor {
        let base = self.background_color();
        let is_dark = base.r < 0.5;
        
        if is_dark {
            // Dark mode: lighter border
            NSColor::new(
                (base.r + 0.1).min(1.0),
                (base.g + 0.1).min(1.0),
                (base.b + 0.1).min(1.0),
                0.3,
            )
        } else {
            // Light mode: darker border
            NSColor::new(
                (base.r - 0.1).max(0.0),
                (base.g - 0.1).max(0.0),
                (base.b - 0.1).max(0.0),
                0.2,
            )
        }
    }

    /// Get the separator color for this layer
    pub fn separator_color(self) -> NSColor {
        let base = self.background_color();
        let is_dark = base.r < 0.5;
        
        if is_dark {
            NSColor::new(
                (base.r + 0.05).min(1.0),
                (base.g + 0.05).min(1.0),
                (base.b + 0.05).min(1.0),
                0.5,
            )
        } else {
            NSColor::new(
                (base.r - 0.05).max(0.0),
                (base.g - 0.05).max(0.0),
                (base.b - 0.05).max(0.0),
                0.3,
            )
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Component Layer Mapping
// ═══════════════════════════════════════════════════════════════════════

/// Get the appropriate layer for a component type
pub fn component_layer(component: &str) -> Layer {
    match component {
        "window" => Layer::Window,
        "content" => Layer::Content,
        "panel" | "inspector" | "hierarchy" | "viewport" => Layer::Panel,
        "sidebar" | "activity_bar" | "tools_sidebar" => Layer::Sidebar,
        "toolbar" | "tab_bar" | "status_bar" => Layer::Panel,
        "button" | "textfield" | "slider" | "switch" => Layer::Control,
        "button:hover" | "button:active" => Layer::ControlElevated,
        "menu" | "popover" | "dropdown" => Layer::Overlay,
        "dialog" | "sheet" | "alert" => Layer::Modal,
        _ => Layer::Control,
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Convenience Functions
// ═══════════════════════════════════════════════════════════════════════

/// Get background color for a component
pub fn bg_color(component: &str) -> NSColor {
    component_layer(component).background_color()
}

/// Get border color for a component
pub fn border_color(component: &str) -> NSColor {
    component_layer(component).border_color()
}

/// Get separator color for a component
pub fn separator_color(component: &str) -> NSColor {
    component_layer(component).separator_color()
}

/// Apply hover effect to a color (lighten in dark mode, darken in light mode)
pub fn hover_color(base: NSColor) -> NSColor {
    let is_dark = base.r < 0.5;
    let amount = 0.1;
    
    if is_dark {
        NSColor::new(
            (base.r + amount).min(1.0),
            (base.g + amount).min(1.0),
            (base.b + amount).min(1.0),
            base.a,
        )
    } else {
        NSColor::new(
            (base.r - amount).max(0.0),
            (base.g - amount).max(0.0),
            (base.b - amount).max(0.0),
            base.a,
        )
    }
}

/// Apply active/pressed effect to a color
pub fn active_color(base: NSColor) -> NSColor {
    let is_dark = base.r < 0.5;
    let amount = 0.15;
    
    if is_dark {
        NSColor::new(
            (base.r + amount).min(1.0),
            (base.g + amount).min(1.0),
            (base.b + amount).min(1.0),
            base.a,
        )
    } else {
        NSColor::new(
            (base.r - amount).max(0.0),
            (base.g - amount).max(0.0),
            (base.b - amount).max(0.0),
            base.a,
        )
    }
}

/// Apply disabled effect to a color (reduce opacity)
pub fn disabled_color(base: NSColor) -> NSColor {
    NSColor::new(base.r, base.g, base.b, base.a * 0.5)
}
