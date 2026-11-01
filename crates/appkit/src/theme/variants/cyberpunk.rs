//! Cyberpunk variant — neon cyan/magenta/yellow on deep purple-black.
//!
//! High-energy, futuristic aesthetic. Neon accents against near-black
//! purple surfaces. Uses Orbitron for a sci-fi display feel.

use super::NSThemeVariant;
use crate::graphics::NSColor;
use crate::theme::{keys, ThemeVariant, ThemeFont};
use std::collections::HashMap;

use crate::theme::registry::{rgb, rgba};

/// Cyberpunk variant singleton.
pub struct CyberpunkVariant;

impl NSThemeVariant for CyberpunkVariant {
    fn name(&self) -> &'static str {
        "cyberpunk"
    }

    fn variant(&self) -> ThemeVariant {
        ThemeVariant::Cyberpunk
    }

    fn apply_colors(&self, c: &mut HashMap<String, NSColor>) {
        // Accent: neon cyan
        c.insert(keys::CLR_ACCENT.into(), rgb(0.0, 1.0, 0.9));
        c.insert(keys::CLR_ACCENT_LIGHT.into(), rgb(0.3, 1.0, 1.0));
        c.insert(keys::CLR_ACCENT_DARK.into(), rgb(0.0, 0.7, 0.6));

        // Surfaces: deep purple-black
        c.insert(keys::CLR_BG_WINDOW.into(), rgb(0.04, 0.02, 0.08));
        c.insert(keys::CLR_BG_SIDEBAR.into(), rgb(0.03, 0.01, 0.06));
        c.insert(keys::CLR_BG_PANEL.into(), rgb(0.06, 0.03, 0.10));
        c.insert(keys::CLR_BG_PANEL.into(), rgba(0.05, 0.02, 0.09, 0.95));
        c.insert(keys::CLR_BG_CANVAS.into(), rgb(0.02, 0.01, 0.04));

        // Interactive states: neon-tinted
        c.insert(keys::CLR_BG_SELECTED.into(), rgba(0.0, 1.0, 0.9, 0.25));
        c.insert(keys::CLR_BG_HOVER.into(), rgba(1.0, 0.0, 0.8, 0.15));

        // Text: neon multi-color hierarchy
        c.insert(keys::CLR_TEXT_LABEL.into(), rgb(0.0, 1.0, 1.0));
        c.insert(keys::CLR_TEXT_SECONDARY.into(), rgb(1.0, 0.0, 0.8));
        c.insert(keys::CLR_TEXT_TERTIARY.into(), rgb(1.0, 1.0, 0.0));

        // Chrome: neon separators and borders
        c.insert(keys::CLR_SEPARATOR.into(), rgba(0.0, 1.0, 0.9, 0.3));
        c.insert(keys::CLR_BORDER.into(), rgba(1.0, 0.0, 0.8, 0.4));
    }

    fn apply_fonts(&self, font: &mut ThemeFont) {
        *font = ThemeFont {
            ui_font: "Orbitron".into(),
            ui_font_size: 12.0,
            title_font: "Orbitron".into(),
            title_font_size: 14.0,
            monospace_font: "Fira Code".into(),
            monospace_font_size: 13.0,
            icon_font: None,
        };
    }
}
