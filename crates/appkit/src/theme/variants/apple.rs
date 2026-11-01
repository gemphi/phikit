//! Apple variant — macOS Sonoma blue accent, SF Pro typography.
//!
//! The default variant for the toolkit. Clean, minimal, system-native feel.
//! Uses Apple's SF font family throughout.

use super::NSThemeVariant;
use crate::graphics::NSColor;
use crate::theme::{keys, ThemeVariant, ThemeFont};
use std::collections::HashMap;

use crate::theme::registry::rgb;

/// Apple variant singleton.
pub struct AppleVariant;

impl NSThemeVariant for AppleVariant {
    fn name(&self) -> &'static str {
        "apple"
    }

    fn variant(&self) -> ThemeVariant {
        ThemeVariant::Apple
    }

    fn apply_colors(&self, c: &mut HashMap<String, NSColor>) {
        // Accent: Apple blue
        c.insert(keys::CLR_ACCENT.into(), rgb(0.0, 0.48, 1.0));
        c.insert(keys::CLR_ACCENT_LIGHT.into(), rgb(0.25, 0.65, 1.0));
        c.insert(keys::CLR_ACCENT_DARK.into(), rgb(0.0, 0.35, 0.85));
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
