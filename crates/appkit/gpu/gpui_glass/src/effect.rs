//! Apple's names for what a host sets on glass, as plain data:
//! `NSGlassEffectView` and `NSGlassEffectContainerView` (AppKit, macOS 26;
//! `NSGlassEffectView.h:14-57`). A view system keeps one of these on a view
//! and hands the crate the material and the shape it gives.

use crate::{GlassMaterial, GlassShape, GlassStyle};

/// `NSGlassEffectView`: one piece of glass. Its content draws on top, sharp.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct GlassEffect {
    /// `style`.
    pub style: GlassStyle,
    /// `cornerRadius`, in logical pixels.
    pub corner_radius: f32,
    /// `tintColor`, with its alpha. Without one the theme's panel colour tints.
    pub tint: Option<[f32; 4]>,
}

impl GlassEffect {
    /// The material in a theme: this effect's own tint, or else `panel`.
    pub fn material(&self, panel: [f32; 4], opacity: Option<f32>, edge: [f32; 3], shadow: [f32; 3]) -> GlassMaterial {
        GlassMaterial::themed(self.style, self.tint.unwrap_or(panel), opacity, edge, shadow)
    }

    /// The outline of the glass at a place.
    pub fn shape(&self, x: f32, y: f32, width: f32, height: f32) -> GlassShape {
        GlassShape::new(x, y, width, height, self.corner_radius)
    }
}

/// `NSGlassEffectContainerView`: the glass inside it renders as one layer,
/// and pieces within `spacing` of each other merge.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct GlassContainer {
    /// `spacing`, in logical pixels.
    pub spacing: f32,
}
