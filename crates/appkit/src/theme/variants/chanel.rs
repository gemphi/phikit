//! Chanel variant — Haute Couture SS26 mushroom forest palette.
//!
//! Inspired by Chanel's understated luxury: muted earth tones,
//! champagne gold accent, and refined serif typography.
//! Deep forest greens and mushroom browns create an organic,
//! sophisticated atmosphere.

use super::NSThemeVariant;
use crate::graphics::NSColor;
use crate::theme::{keys, ThemeVariant, ThemeFont};
use std::collections::HashMap;

use crate::theme::registry::{rgb, rgba};

/// Chanel variant singleton.
pub struct ChanelVariant;

impl NSThemeVariant for ChanelVariant {
    fn name(&self) -> &'static str {
        "chanel"
    }

    fn variant(&self) -> ThemeVariant {
        ThemeVariant::Chanel
    }

    fn apply_colors(&self, c: &mut HashMap<String, NSColor>) {
        // Accent: champagne gold
        c.insert(keys::CLR_ACCENT.into(), rgb(0.80, 0.72, 0.55));
        c.insert(keys::CLR_ACCENT_LIGHT.into(), rgb(0.92, 0.85, 0.70));
        c.insert(keys::CLR_ACCENT_DARK.into(), rgb(0.60, 0.52, 0.38));

        // Surfaces: deep mushroom/forest
        c.insert(keys::CLR_BG_WINDOW.into(), rgb(0.08, 0.07, 0.06));
        c.insert(keys::CLR_BG_SIDEBAR.into(), rgb(0.06, 0.05, 0.04));
        c.insert(keys::CLR_BG_PANEL.into(), rgb(0.10, 0.09, 0.07));
        c.insert(keys::CLR_BG_PANEL.into(), rgba(0.09, 0.08, 0.06, 0.95));
        c.insert(keys::CLR_BG_CANVAS.into(), rgb(0.04, 0.03, 0.03));

        // Interactive states: gold-tinted
        c.insert(keys::CLR_BG_SELECTED.into(), rgba(0.80, 0.72, 0.55, 0.25));
        c.insert(keys::CLR_BG_HOVER.into(), rgba(0.80, 0.72, 0.55, 0.12));

        // Text: warm ivory with muted secondary
        c.insert(keys::CLR_TEXT_LABEL.into(), rgb(0.95, 0.92, 0.88));
        c.insert(keys::CLR_TEXT_SECONDARY.into(), rgb(0.70, 0.65, 0.55));
        c.insert(keys::CLR_TEXT_TERTIARY.into(), rgba(0.70, 0.65, 0.55, 0.5));

        // Chrome: subtle gold separators
        c.insert(keys::CLR_SEPARATOR.into(), rgba(0.80, 0.72, 0.55, 0.15));
        c.insert(keys::CLR_BORDER.into(), rgba(0.80, 0.72, 0.55, 0.10));
    }

    fn apply_fonts(&self, font: &mut ThemeFont) {
        *font = ThemeFont {
            ui_font: "Optima".into(),
            ui_font_size: 13.0,
            title_font: "Didot".into(),
            title_font_size: 15.0,
            monospace_font: "Menlo".into(),
            monospace_font_size: 12.0,
            icon_font: None,
        };
    }
}
