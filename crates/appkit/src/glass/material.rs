//! How a piece of glass looks. Plain numbers here; [`GlassMaterial::themed`]
//! is the one place they meet the theme, so no colour is fixed in this module.

use crate::theme::keys;
use crate::NSColor;

/// `NSGlassEffectView.Style`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GlassStyle {
    /// Frosted: a wide blur and a real tint, for panels with text on them.
    #[default]
    Regular,
    /// Barely frosted: the backdrop stays readable, the rim does the work.
    Clear,
}

/// Everything the shading stage needs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlassMaterial {
    /// Blur reach in pixels; the Gaussian's sigma is half of this.
    pub blur_radius: f32,
    /// 1 leaves colour alone. Blurring greys colour out; this brings it back.
    pub saturation: f32,
    pub tint: [f32; 3],
    /// How far colour moves toward `tint`, 0..=1.
    pub tint_amount: f32,
    /// How far the rim pulls its sample inward, in pixels.
    pub lens: f32,
    pub edge: [f32; 3],
    pub edge_strength: f32,
    pub shadow: [f32; 3],
    pub shadow_strength: f32,
}

impl GlassMaterial {
    /// The style's shape, without colours: a neutral tint and edge.
    pub fn of(style: GlassStyle) -> Self {
        let (blur_radius, saturation, tint_amount, lens) = match style {
            GlassStyle::Regular => (24.0, 1.5, 0.28, 26.0),
            GlassStyle::Clear => (8.0, 1.1, 0.05, 36.0),
        };
        Self {
            blur_radius,
            saturation,
            tint: [0.0; 3],
            tint_amount,
            lens,
            edge: [1.0; 3],
            edge_strength: 0.75,
            shadow: [0.0; 3],
            shadow_strength: 0.42,
        }
    }

    /// The style in the current theme. `tint` is the view's own colour; the
    /// theme's `glass_opacity` sets how much of it covers a Regular backdrop.
    pub fn themed(style: GlassStyle, tint: Option<NSColor>) -> Self {
        let base = Self::of(style);
        let opacity = crate::theme::f32_or(keys::GLASS_OPACITY, base.tint_amount);
        let share = match style {
            GlassStyle::Regular => opacity,
            GlassStyle::Clear => opacity * base.tint_amount / Self::of(GlassStyle::Regular).tint_amount,
        };
        let tint = tint.unwrap_or_else(|| crate::theme::color(keys::CLR_BG_PANEL));
        Self {
            tint: rgb(tint),
            tint_amount: (share * tint.a).clamp(0.0, 1.0),
            edge: rgb(crate::theme::color_or(keys::CLR_GLASS_EDGE, NSColor::white())),
            shadow: rgb(crate::theme::color(keys::CLR_SHADOW)),
            ..base
        }
    }
}

fn rgb(c: NSColor) -> [f32; 3] {
    [c.r, c.g, c.b]
}
