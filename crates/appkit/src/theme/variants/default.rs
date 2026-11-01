//! Default/Custom variant — no overrides, inherits everything from base + mode.
//!
//! Used as the fallback when an unknown variant key is passed to `set_variant()`.
//! Also serves as the "custom" variant for user-defined themes.

use super::NSThemeVariant;
use crate::graphics::NSColor;
use crate::theme::ThemeVariant;
use std::collections::HashMap;

/// Default variant singleton — applies no overrides.
pub struct DefaultVariant;

impl NSThemeVariant for DefaultVariant {
    fn name(&self) -> &'static str {
        "custom"
    }

    fn variant(&self) -> ThemeVariant {
        ThemeVariant::Custom
    }

    fn apply_colors(&self, _c: &mut HashMap<String, NSColor>) {
        // No overrides — inherits base + mode colors as-is.
    }
}
