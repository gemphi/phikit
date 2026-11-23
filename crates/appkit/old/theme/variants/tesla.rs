//! Tesla variant — deep black surfaces, red accent.
//!
//! Inspired by Tesla's automotive UI: ultra-dark backgrounds with
//! a bold red accent. High contrast, minimal chrome.

use super::NSThemeVariant;
use crate::graphics::NSColor;
use crate::theme::{keys, ThemeVariant};
use std::collections::HashMap;

use crate::theme::registry::rgb;

/// Tesla variant singleton.
pub struct TeslaVariant;

impl NSThemeVariant for TeslaVariant {
    fn name(&self) -> &'static str {
        "tesla"
    }

    fn variant(&self) -> ThemeVariant {
        ThemeVariant::Tesla
    }

    fn apply_colors(&self, c: &mut HashMap<String, NSColor>) {
        let tesla_red = rgb(0.90, 0.10, 0.15);

        // Accent: Tesla red
        c.insert(keys::CLR_ACCENT.into(), tesla_red);
        c.insert(keys::CLR_ACCENT_LIGHT.into(), rgb(1.00, 0.25, 0.30));
        c.insert(keys::CLR_ACCENT_DARK.into(), rgb(0.70, 0.05, 0.10));

        // Surfaces: ultra-dark
        c.insert(keys::CLR_BG_WINDOW.into(), rgb(0.05, 0.05, 0.05));
        c.insert(keys::CLR_BG_SIDEBAR.into(), rgb(0.02, 0.02, 0.02));
        c.insert(keys::CLR_BG_CANVAS.into(), rgb(0.00, 0.00, 0.00));

        // Text: near-white for maximum contrast
        c.insert(keys::CLR_TEXT_LABEL.into(), rgb(0.98, 0.98, 0.98));
    }
}
