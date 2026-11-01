//! Swift variant — warm orange accent, SF Pro typography.
//!
//! Inspired by the Swift programming language's identity.
//! Warm-tinted dark surfaces with the iconic Swift orange.

use super::NSThemeVariant;
use crate::graphics::NSColor;
use crate::theme::{keys, ThemeVariant, ThemeFont};
use std::collections::HashMap;

use crate::theme::registry::{rgb, rgba};

/// Swift variant singleton.
pub struct SwiftVariant;

impl NSThemeVariant for SwiftVariant {
    fn name(&self) -> &'static str {
        "swift"
    }

    fn variant(&self) -> ThemeVariant {
        ThemeVariant::Swift
    }

    fn apply_colors(&self, c: &mut HashMap<String, NSColor>) {
        // Accent: Swift orange
        c.insert(keys::CLR_ACCENT.into(), rgb(1.0, 0.35, 0.15));
        c.insert(keys::CLR_ACCENT_LIGHT.into(), rgb(1.0, 0.50, 0.30));
        c.insert(keys::CLR_ACCENT_DARK.into(), rgb(0.85, 0.25, 0.10));

        // Surfaces: warm-tinted dark
        c.insert(keys::CLR_BG_WINDOW.into(), rgb(0.10, 0.08, 0.07));
        c.insert(keys::CLR_BG_SIDEBAR.into(), rgb(0.08, 0.06, 0.05));
        c.insert(keys::CLR_BG_PANEL.into(), rgb(0.12, 0.10, 0.09));
        c.insert(keys::CLR_BG_PANEL.into(), rgba(0.11, 0.09, 0.08, 0.95));
        c.insert(keys::CLR_BG_CANVAS.into(), rgb(0.06, 0.04, 0.03));
    }

    fn apply_fonts(&self, font: &mut ThemeFont) {
        *font = ThemeFont {
            ui_font: "SF Pro".into(),
            ui_font_size: 13.0,
            title_font: "SF Pro Display".into(),
            title_font_size: 15.0,
            monospace_font: "SF Mono".into(),
            monospace_font_size: 13.0,
            icon_font: Some("SF Symbols".into()),
        };
    }
}
