//! Theming integration: appearance, visual properties, and NSThemeable trait.

use crate::graphics::{NSColor, NSSize};
use super::properties::NSView;

impl NSView {
    /// Set a per-view appearance override.
    pub fn set_appearance(&mut self, appearance: Option<crate::theme::NSAppearance>) {
        self.appearance = appearance;
    }

    /// The explicitly set appearance for this view, or `None` if inheriting.
    pub fn appearance(&self) -> Option<crate::theme::NSAppearance> { self.appearance }

    /// The resolved appearance for this view.
    pub fn effective_appearance(&self) -> crate::theme::NSAppearance {
        self.appearance.unwrap_or_else(|| crate::theme::NSAppearance::current_drawing())
    }

    pub fn set_theme_token(&mut self, token: &str) {
        self.theme_token = Some(token.to_string());
    }
    pub fn set_radius_token(&mut self, token: &str) {
        self.radius_token = Some(token.to_string());
    }
    pub fn set_glass_token(&mut self, token: &str) {
        self.glass_token = Some(token.to_string());
    }
    pub fn set_border_color_token(&mut self, token: &str) {
        self.border_color_token = Some(token.to_string());
    }

    pub fn background_color(&self) -> Option<NSColor> { self.background_color }
    pub fn set_background_color(&mut self, color: Option<NSColor>) {
        self.background_color = color;
    }

    /// Set the hover color — displayed when the cursor is over this view.
    pub fn set_hover_color(&mut self, color: Option<NSColor>) { self.hover_color = color; }
    /// Returns whether this view is currently hovered.
    pub fn is_hovered(&self) -> bool { self.is_hovered }
    /// Set the hovered state of this view.
    pub fn set_hovered(&mut self, hovered: bool) { self.is_hovered = hovered; }

    /// Set a vertical gradient background (top to bottom).
    pub fn set_gradient_background(&mut self, top: NSColor, bottom: NSColor) {
        self.gradient_top = Some(top);
        self.gradient_bottom = Some(bottom);
    }

    pub fn set_corner_radius(&mut self, radius: f32) { self.corner_radius = radius.max(0.0); }
    pub fn corner_radius(&self) -> f32 { self.corner_radius }
    pub fn set_glass(&mut self, glass: bool) { self.is_glass = glass; }
    pub fn is_glass(&self) -> bool { self.is_glass }

    /// Set the border width (in pixels). A value > 0 enables border rendering.
    pub fn set_border_width(&mut self, width: f32) { self.border_width = width.max(0.0); }
    pub fn border_width(&self) -> f32 { self.border_width }

    /// Set the border color.
    pub fn set_border_color(&mut self, color: Option<NSColor>) { self.border_color = color; }
    pub fn border_color(&self) -> Option<NSColor> { self.border_color }

    pub fn set_wants_layer(&mut self, wants: bool) { self.wants_layer = wants; }
    pub fn wants_layer(&self) -> bool { self.wants_layer }

    pub fn set_tool_tip(&mut self, text: &str) { self.tool_tip = Some(text.to_string()); }
    pub fn tool_tip(&self) -> Option<String> { self.tool_tip.clone() }

    /// Set the transparency (0.0 to 1.0)
    pub fn set_alpha_value(&mut self, alpha: f32) { self.alpha_value = alpha.clamp(0.0, 1.0); }
    pub fn alpha_value(&self) -> f32 { self.alpha_value }

    pub fn set_min_size(&mut self, size: NSSize) { self.min_size = size; }
    pub fn min_size(&self) -> NSSize { self.min_size }
    pub fn set_max_size(&mut self, size: NSSize) { self.max_size = size; }
    pub fn max_size(&self) -> NSSize { self.max_size }

    pub fn clips_to_bounds(&self) -> bool { self.clips_to_bounds }
    pub fn set_clips_to_bounds(&mut self, clip: bool) { self.clips_to_bounds = clip; }
}

/// Trait for views that respond to theme changes
pub trait NSThemeable {
    fn apply_theme(&mut self);
}

impl NSThemeable for NSView {
    fn apply_theme(&mut self) {
        if let Some(ref token) = self.theme_token {
            let color = crate::theme::color(token);
            self.set_background_color(Some(color));
        }
        if let Some(ref token) = self.radius_token {
            let radius = crate::theme::f32_or(token, 0.0);
            self.set_corner_radius(radius);
        }
        if let Some(ref token) = self.glass_token {
            let glass = crate::theme::bool_or(token, false);
            self.set_glass(glass);
        }
        if let Some(ref token) = self.border_color_token {
            let color = crate::theme::color(token);
            self.set_border_color(Some(color));
        }
        for subview in &mut self.subviews {
            subview.apply_theme();
        }
    }
}
