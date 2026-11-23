//! Color types - RGBA color and semantic system color enum.

/// Semantic system colors mirrored from macOS AppKit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NSSystemColor {
    SystemBlue,
    SystemBrown,
    SystemGray,
    SystemGreen,
    SystemIndigo,
    SystemOrange,
    SystemPink,
    SystemPurple,
    SystemRed,
    SystemTeal,
    SystemYellow,
    SystemMint,
    SystemCyan,
    Label,
    SecondaryLabel,
    TertiaryLabel,
    QuaternaryLabel,
    LinkColor,
    PlaceholderTextColor,
    WindowBackgroundColor,
    WindowFrameColor,
    UnderPageBackgroundColor,
    ControlBackgroundColor,
    SelectedControlColor,
    SelectedControlTextColor,
    SelectedTextBackgroundColor,
    SeparatorColor,
    SystemBackground,
    TrafficClose,
    TrafficMin,
    TrafficZoom,
}

/// RGBA Color
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NSColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Default for NSColor {
    fn default() -> Self {
        Self::clear()
    }
}

impl NSColor {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::rgba(r, g, b, 1.0)
    }
    pub fn white() -> Self {
        Self::rgb(1.0, 1.0, 1.0)
    }
    pub fn black() -> Self {
        Self::rgb(0.0, 0.0, 0.0)
    }
    pub fn gray() -> Self {
        Self::rgb(0.5, 0.5, 0.5)
    }
    pub fn clear() -> Self {
        Self::rgba(0.0, 0.0, 0.0, 0.0)
    }

    // System colors - modeled after modern macOS Dark Mode (Reality Composer Pro aesthetic)
    pub fn system_color(color: NSSystemColor) -> Self {
        crate::theme::resolve_system_color(color)
    }

    /// With alpha component
    pub fn with_alpha(&self, alpha: f32) -> Self {
        Self::new(self.r, self.g, self.b, alpha)
    }

    /// Convert to u8 RGBA
    pub fn to_rgba8(&self) -> [u8; 4] {
        [
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
            (self.a * 255.0) as u8,
        ]
    }

    /// Convert to BGRA u32 for Windows bitmap
    pub fn to_bgra_u32(&self) -> u32 {
        let r = (self.r * 255.0) as u32;
        let g = (self.g * 255.0) as u32;
        let b = (self.b * 255.0) as u32;
        let a = (self.a * 255.0) as u32;
        (a << 24) | (r << 16) | (g << 8) | b
    }

    /// Blend with another color
    pub fn blended_with(&self, other: &NSColor, fraction: f32) -> Self {
        Self::new(
            self.r + (other.r - self.r) * fraction,
            self.g + (other.g - self.g) * fraction,
            self.b + (other.b - self.b) * fraction,
            self.a + (other.a - self.a) * fraction,
        )
    }

    /// Brighten the color by a percentage (0-100).
    /// Each RGB channel is multiplied by (1 + percentage/100) and clamped to [0, 1].
    /// Alpha channel is preserved unchanged.
    pub fn brightened(&self, percentage: f32) -> Self {
        let factor = 1.0 + percentage / 100.0;
        Self::new(
            (self.r * factor).min(1.0).max(0.0),
            (self.g * factor).min(1.0).max(0.0),
            (self.b * factor).min(1.0).max(0.0),
            self.a,
        )
    }
}


