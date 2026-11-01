//! Barbie variant — hot pink accent, playful typography.
//!
//! Bold, unapologetic pink palette with warm-tinted dark surfaces.
//! Uses Avenir Next for a friendly, approachable feel.

use super::NSThemeVariant;
use crate::graphics::NSColor;
use crate::theme::{keys, ThemeVariant, ThemeFont};
use std::collections::HashMap;

use crate::theme::registry::{rgb, rgba};

/// Barbie variant singleton.
pub struct BarbieVariant;

impl NSThemeVariant for BarbieVariant {
    fn name(&self) -> &'static str {
        "barbie"
    }

    fn variant(&self) -> ThemeVariant {
        ThemeVariant::Barbie
    }

    fn apply_colors(&self, c: &mut HashMap<String, NSColor>) {
        // Accent: hot pink
        c.insert(keys::CLR_ACCENT.into(), rgb(0.93, 0.11, 0.55));
        c.insert(keys::CLR_ACCENT_LIGHT.into(), rgb(1.0, 0.40, 0.72));
        c.insert(keys::CLR_ACCENT_DARK.into(), rgb(0.72, 0.05, 0.40));

        // Surfaces: pink-tinted dark
        c.insert(keys::CLR_BG_WINDOW.into(), rgb(0.12, 0.06, 0.10));
        c.insert(keys::CLR_BG_SIDEBAR.into(), rgb(0.09, 0.04, 0.07));
        c.insert(keys::CLR_BG_PANEL.into(), rgb(0.15, 0.08, 0.12));
        c.insert(keys::CLR_BG_PANEL.into(), rgba(0.14, 0.07, 0.11, 0.95));
        c.insert(keys::CLR_BG_CANVAS.into(), rgb(0.05, 0.02, 0.04));

        // Interactive states: pink-tinted
        c.insert(keys::CLR_BG_SELECTED.into(), rgba(0.93, 0.11, 0.55, 0.3));
        c.insert(keys::CLR_BG_HOVER.into(), rgba(0.93, 0.11, 0.55, 0.15));

        // Text: warm white with pink secondary
        c.insert(keys::CLR_TEXT_LABEL.into(), rgb(1.0, 0.95, 0.97));
        c.insert(keys::CLR_TEXT_SECONDARY.into(), rgb(0.85, 0.65, 0.75));

        // Chrome: pink separator
        c.insert(keys::CLR_SEPARATOR.into(), rgba(0.93, 0.11, 0.55, 0.25));
    }

    fn apply_fonts(&self, font: &mut ThemeFont) {
        *font = ThemeFont {
            ui_font: "Avenir Next".into(),
            ui_font_size: 13.0,
            title_font: "Avenir Next".into(),
            title_font_size: 16.0,
            monospace_font: "Menlo".into(),
            monospace_font_size: 12.0,
            icon_font: None,
        };
    }
}
