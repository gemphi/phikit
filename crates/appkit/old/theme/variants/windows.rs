//! Windows variant — Fluent Design blue accent, Segoe UI Variable typography.
//!
//! Matches the Windows 11 design language with slightly larger font sizes
//! and the Fluent icon set.

use super::NSThemeVariant;
use crate::graphics::NSColor;
use crate::theme::{keys, ThemeVariant, ThemeFont};
use std::collections::HashMap;

use crate::theme::registry::rgb;

/// Windows variant singleton.
pub struct WindowsVariant;

impl NSThemeVariant for WindowsVariant {
    fn name(&self) -> &'static str {
        "windows"
    }

    fn variant(&self) -> ThemeVariant {
        ThemeVariant::Windows
    }

    fn apply_colors(&self, c: &mut HashMap<String, NSColor>) {
        // Accent: Fluent blue (lighter than Apple)
        c.insert(keys::CLR_ACCENT.into(), rgb(0.31, 0.71, 1.0));
        c.insert(keys::CLR_ACCENT_LIGHT.into(), rgb(0.50, 0.82, 1.0));
        c.insert(keys::CLR_ACCENT_DARK.into(), rgb(0.20, 0.58, 0.90));
    }

    fn apply_fonts(&self, font: &mut ThemeFont) {
        *font = ThemeFont {
            ui_font: "Segoe UI Variable".into(),
            ui_font_size: 14.0,
            title_font: "Segoe UI Variable Display".into(),
            title_font_size: 20.0,
            monospace_font: "Cascadia Mono".into(),
            monospace_font_size: 14.0,
            icon_font: Some("Segoe Fluent Icons".into()),
        };
    }
}
