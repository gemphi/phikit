//! # Theme Variants
//!
//! Each variant defines its accent palette, semantic colors, fonts, and identity.
//! Variants compose with Mode (dark/light) and Material (glass/metal/onyx)
//! to produce a complete ThemeData at runtime.
//!
//! All variants implement the `NSThemeVariant` trait, which provides:
//! - `name()` — the string key used in `theme::set_variant("apple")`
//! - `variant()` — the typed `ThemeVariant` variant
//! - `apply()` — mutates a `ThemeData` with variant-specific overrides
//!
//! The registry calls `apply()` during composition. Each variant only overrides
//! what it needs — everything else inherits from the base + mode layers.

pub mod apple;
pub mod barbie;
pub mod chanel;
pub mod cyberpunk;
pub mod default;
pub mod fenty;
pub mod swift;
pub mod tesla;
pub mod windows;

pub use apple::AppleVariant;
pub use barbie::BarbieVariant;
pub use chanel::ChanelVariant;
pub use cyberpunk::CyberpunkVariant;
pub use default::DefaultVariant;
pub use fenty::FentyVariant;
pub use swift::SwiftVariant;
pub use tesla::TeslaVariant;
pub use windows::WindowsVariant;

use crate::graphics::NSColor;
use super::{ThemeVariant, ThemeData, ThemeFont};
use std::collections::HashMap;

/// The core trait every variant must implement.
///
/// Variants are composable layers — they override accent colors, background tints,
/// fonts, and any other tokens that define their visual identity. The base theme
/// and mode layer provide sensible defaults; variants only override what's unique.
pub trait NSThemeVariant: Send + Sync {
    /// The string key used in `theme::set_variant(...)`.
    fn name(&self) -> &'static str;

    /// The typed enum variant for this variant.
    fn variant(&self) -> ThemeVariant;

    /// Apply variant-specific color overrides to the theme's color map.
    fn apply_colors(&self, colors: &mut HashMap<String, NSColor>);

    /// Apply variant-specific font overrides. Default: no-op (inherits base fonts).
    fn apply_fonts(&self, _fonts: &mut ThemeFont) {}

    /// Apply variant-specific geometry overrides. Default: no-op.
    fn apply_geometry(&self, _geometry: &mut HashMap<String, f32>) {}

    /// Full apply: colors + fonts + geometry + variant ID.
    /// Override only if you need custom logic beyond the three sub-methods.
    fn apply(&self, d: &mut ThemeData) {
        self.apply_colors(&mut d.colors);
        self.apply_fonts(&mut d.font);
        self.apply_geometry(&mut d.layout.geometry);
        d.id.variant = self.variant();
    }
}

/// Returns the variant implementation for a given key string.
/// Falls back to `DefaultVariant` for unknown keys.
pub fn variant_for(key: &str) -> &'static dyn NSThemeVariant {
    match key {
        "apple" => &AppleVariant,
        "windows" => &WindowsVariant,
        "tesla" => &TeslaVariant,
        "swift" => &SwiftVariant,
        "barbie" => &BarbieVariant,
        "fenty" => &FentyVariant,
        "cyberpunk" => &CyberpunkVariant,
        "chanel" => &ChanelVariant,
        _ => &DefaultVariant,
    }
}
