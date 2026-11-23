//! # Theme System
//!
//! Composable, layered appearance engine inspired by Apple's NSAppearance.
//!
//! ## Architecture
//!
//! ```text
//! theme/
//! ├── mod.rs          ← this file (orchestrator + re-exports)
//! ├── appearance.rs   ← Apple's NSAppearance types (mode, vibrancy, control size…)
//! ├── styles.rs       ← Our composable layers (Variant, Material, Typography…)
//! ├── types.rs        ← Resolved data (ThemeData, ThemeFont, ThemeLayout, ThemeId)
//! ├── accessors.rs    ← Public API (color(), set_variant(), set_mode()…)
//! ├── keys.rs         ← Token key constants (CLR_BG_PANEL, RADIUS_BUTTON…)
//! ├── registry.rs     ← Global singleton, composition engine
//! └── variants/       ← Per-variant color/font overrides (apple, tesla, barbie…)
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use rust_appkit::theme;
//!
//! // Read tokens
//! let bg = theme::color(theme::CLR_BG_PANEL);
//! let radius = theme::f32_or(theme::RADIUS_PANEL, 12.0);
//!
//! // Switch layers at runtime
//! theme::set_variant("tesla");
//! theme::set_mode("dark");
//! theme::set_material("glass");
//! ```

// ─── Submodules ──────────────────────────────────────────────────────────────

pub mod appearance;
pub mod styles;
pub mod types;
pub mod accessors;
pub mod keys;
pub mod registry;
pub mod variants;

// ─── Re-exports: appearance ──────────────────────────────────────────────────

pub use appearance::{
    AppearanceMode, AppearanceName, AppearanceState, BackgroundStyle, BlendingMode, ControlSize,
    NSAppearance, NSVisualEffectMaterial, ThemeMode, Vibrancy,
};

// ─── Re-exports: styles ──────────────────────────────────────────────────────

pub use styles::{
    ColorGradient, LayoutDensity, Material, RadiusStyle, ThemeMaterial, ThemeVariant,
    Typography, Variant, VisualEffects,
};

// ─── Re-exports: types ───────────────────────────────────────────────────────

pub use types::{Appearance, ThemeData, ThemeFont, ThemeId, ThemeLayout, ThemeProvider};

// ─── Re-exports: accessors (the primary public API) ──────────────────────────

pub use accessors::{
    bool_or, color, color_or, f32, f32_or, resolve_system_color, set_accent, set_current_theme,
    set_density, set_effect, set_gradient, set_layout, set_material, set_mode, set_radius,
    set_shadow, set_tint, set_typography, set_variant,
};

// ─── Re-exports: keys (flattened so consumers write `theme::CLR_BG_PANEL`) ──

pub use keys::*;

pub mod token;

// ─── Re-exports: token (clean f32 accessor) ──────────────────────────────────

pub use token::ThemeToken;

// ─── Re-exports: variant trait ───────────────────────────────────────────────

pub use variants::NSThemeVariant;
