//! Theme-aware color helpers and utility functions for NSColor.

use super::{NSColor, NSSystemColor};

// System color convenience methods
impl NSColor {
    pub fn system_background() -> Self {
        Self::system_color(NSSystemColor::SystemBackground)
    }
    pub fn window_background() -> Self {
        Self::system_color(NSSystemColor::WindowBackgroundColor)
    }
    pub fn control_background() -> Self {
        Self::system_color(NSSystemColor::ControlBackgroundColor)
    }
    pub fn system_label() -> Self {
        Self::system_color(NSSystemColor::Label)
    }
    pub fn system_text() -> Self {
        Self::system_color(NSSystemColor::Label)
    }
    pub fn system_accent() -> Self {
        Self::system_color(NSSystemColor::SystemBlue)
    }
    pub fn selected_control() -> Self {
        Self::system_color(NSSystemColor::SelectedControlColor)
    }
    pub fn selected_text_background() -> Self {
        Self::system_color(NSSystemColor::SelectedTextBackgroundColor)
    }
    pub fn secondary_label() -> Self {
        Self::system_color(NSSystemColor::SecondaryLabel)
    }
    pub fn tertiary_label() -> Self {
        Self::system_color(NSSystemColor::TertiaryLabel)
    }
    pub fn quaternary_label() -> Self {
        Self::system_color(NSSystemColor::QuaternaryLabel)
    }
    pub fn placeholder_text() -> Self {
        Self::system_color(NSSystemColor::PlaceholderTextColor)
    }
    pub fn separator() -> Self {
        Self::system_color(NSSystemColor::SeparatorColor)
    }
    pub fn opaque_separator() -> Self {
        Self::system_color(NSSystemColor::SeparatorColor)
    }
    pub fn link() -> Self {
        Self::system_color(NSSystemColor::LinkColor)
    }
    pub fn window_frame() -> Self {
        Self::system_color(NSSystemColor::WindowFrameColor)
    }
    pub fn under_page_background() -> Self {
        Self::system_color(NSSystemColor::UnderPageBackgroundColor)
    }

    // Primary system colors
    pub fn system_blue() -> Self {
        Self::system_color(NSSystemColor::SystemBlue)
    }
    pub fn system_green() -> Self {
        Self::system_color(NSSystemColor::SystemGreen)
    }
    pub fn system_red() -> Self {
        Self::system_color(NSSystemColor::SystemRed)
    }
    pub fn system_yellow() -> Self {
        Self::system_color(NSSystemColor::SystemYellow)
    }
    pub fn system_gray() -> Self {
        Self::system_color(NSSystemColor::SystemGray)
    }

    // Backgrounds
    pub fn secondary_system_background() -> Self {
        Self::new(0.18, 0.18, 0.18, 1.0)
    }
    pub fn tertiary_system_background() -> Self {
        Self::new(0.22, 0.22, 0.22, 1.0)
    }

    // Traffic Lights (visionOS/RCP aesthetic)
    pub fn close_button() -> Self {
        Self::system_color(NSSystemColor::TrafficClose)
    }
    pub fn minimize_button() -> Self {
        Self::system_color(NSSystemColor::TrafficMin)
    }
    pub fn zoom_button() -> Self {
        Self::system_color(NSSystemColor::TrafficZoom)
    }
}

// Theme-aware color helpers
impl NSColor {
    pub fn label() -> Self {
        Self::system_text()
    }

    // ── Semantic background colors (resolve from active theme) ────────
    pub fn bg_content() -> Self {
        crate::theme::color(crate::theme::CLR_BG_CONTENT)
    }
    pub fn bg_panel() -> Self {
        crate::theme::color(crate::theme::CLR_BG_PANEL)
    }
    pub fn bg_sidebar() -> Self {
        crate::theme::color(crate::theme::CLR_BG_SIDEBAR)
    }
    pub fn bg_toolbar() -> Self {
        crate::theme::color(crate::theme::CLR_BG_TOOLBAR)
    }
    pub fn bg_canvas() -> Self {
        crate::theme::color(crate::theme::CLR_BG_CANVAS)
    }
    pub fn bg_hover() -> Self {
        crate::theme::color(crate::theme::CLR_BG_HOVER)
    }
    pub fn bg_selected() -> Self {
        crate::theme::color(crate::theme::CLR_BG_SELECTED)
    }
    pub fn bg_control() -> Self {
        crate::theme::color(crate::theme::CLR_BG_CONTROL)
    }
    pub fn bg_dropdown() -> Self {
        crate::theme::color(crate::theme::CLR_BG_DROPDOWN)
    }

    // ── Semantic text colors ─────────────────────────────────────────
    pub fn text_label() -> Self {
        crate::theme::color(crate::theme::CLR_TEXT_LABEL)
    }
    pub fn text_secondary() -> Self {
        crate::theme::color(crate::theme::CLR_TEXT_SECONDARY)
    }
    pub fn text_tertiary() -> Self {
        crate::theme::color(crate::theme::CLR_TEXT_TERTIARY)
    }
    pub fn text_disabled() -> Self {
        crate::theme::color(crate::theme::CLR_TEXT_DISABLED)
    }

    // ── Semantic accents ─────────────────────────────────────────────
    pub fn accent() -> Self {
        crate::theme::color(crate::theme::CLR_ACCENT)
    }
    pub fn themed_separator() -> Self {
        crate::theme::color(crate::theme::CLR_SEPARATOR)
    }
    pub fn border() -> Self {
        crate::theme::color(crate::theme::CLR_BORDER)
    }

    // ── Generic themed lookup ────────────────────────────────────────
    pub fn themed(token: &str) -> Self {
        crate::theme::color(token)
    }
}

/// Standalone utility function to brighten a color by a percentage.
/// Each RGB channel is multiplied by `(1 + percentage/100)` and clamped to [0, 1].
/// Alpha channel is preserved unchanged.
///
/// This is equivalent to calling `color.brightened(percentage)` but provided as a
/// free function for convenience.
///
/// # Arguments
/// * `color` - The source color to brighten
/// * `percentage` - Brightening percentage (0-100). 15 means 15% brighter.
///
/// # Returns
/// A new `NSColor` with brightened RGB channels, clamped to [0, 1].
pub fn brighten(color: NSColor, percentage: f32) -> NSColor {
    color.brightened(percentage)
}
