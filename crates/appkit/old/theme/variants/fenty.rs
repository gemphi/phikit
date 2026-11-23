//! Fenty variant — warm bronze/terracotta accent, luxury cosmetics aesthetic.
//!
//! Inspired by Fenty Beauty's warm, inclusive palette.
//! Rich earth tones with elegant serif display typography.

use super::NSThemeVariant;
use crate::graphics::NSColor;
use crate::theme::{keys, ThemeVariant, ThemeFont};
use std::collections::HashMap;

use crate::theme::registry::{rgb, rgba};

/// Fenty variant singleton.
pub struct FentyVariant;

impl NSThemeVariant for FentyVariant {
    fn name(&self) -> &'static str {
        "fenty"
    }

    fn variant(&self) -> ThemeVariant {
        ThemeVariant::Fenty
    }

    fn apply_colors(&self, c: &mut HashMap<String, NSColor>) {
        // Accent: warm bronze
        c.insert(keys::CLR_ACCENT.into(), rgb(0.82, 0.55, 0.35));
        c.insert(keys::CLR_ACCENT_LIGHT.into(), rgb(0.95, 0.72, 0.52));
        c.insert(keys::CLR_ACCENT_DARK.into(), rgb(0.60, 0.38, 0.22));

        // Surfaces: warm-tinted dark
        c.insert(keys::CLR_BG_WINDOW.into(), rgb(0.10, 0.08, 0.06));
        c.insert(keys::CLR_BG_SIDEBAR.into(), rgb(0.07, 0.05, 0.04));
        c.insert(keys::CLR_BG_PANEL.into(), rgb(0.13, 0.10, 0.08));
        c.insert(keys::CLR_BG_PANEL.into(), rgba(0.12, 0.09, 0.07, 0.95));
        c.insert(keys::CLR_BG_CANVAS.into(), rgb(0.04, 0.03, 0.02));

        // Interactive states: bronze-tinted
        c.insert(keys::CLR_BG_SELECTED.into(), rgba(0.82, 0.55, 0.35, 0.3));
        c.insert(keys::CLR_BG_HOVER.into(), rgba(0.82, 0.55, 0.35, 0.15));

        // Text: warm cream with bronze secondary
        c.insert(keys::CLR_TEXT_LABEL.into(), rgb(0.98, 0.94, 0.90));
        c.insert(keys::CLR_TEXT_SECONDARY.into(), rgb(0.75, 0.62, 0.52));

        // Chrome: bronze separator
        c.insert(keys::CLR_SEPARATOR.into(), rgba(0.82, 0.55, 0.35, 0.2));
    }

    fn apply_fonts(&self, font: &mut ThemeFont) {
        *font = ThemeFont {
            ui_font: "Helvetica Neue".into(),
            ui_font_size: 13.0,
            title_font: "Didot".into(),
            title_font_size: 16.0,
            monospace_font: "Courier New".into(),
            monospace_font_size: 12.0,
            icon_font: None,
        };
    }
}
