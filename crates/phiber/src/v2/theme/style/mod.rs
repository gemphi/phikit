//! Resolved-UI-style types + palette-defaults factory.
//!
//! Palette colors remain the completed color-token source; every other
//! design default is owned here and palette colors are copied into resolved
//! colors.
//!
//! Split for the file cap:
//!   • `style/mod.rs` (this file) — aggregate `BUiStyleDefaults`/`BResolvedUiStyle`
//!     plus the per-theme defaults + re-exports.
//!   • `style/radius.rs` — radius scales, tokens, role-map.
//!   • `style/metric.rs` — typography / spacing / dimensions / strokes / resize.
//!   • `style/font.rs`   — the eight typefaces Preferences offers.
//!   • `style/effect.rs` — shadow style + gradient/lift/etc effects.
//!   • `style/color.rs`  — `BResolvedColors` + palette→colours copy.
//!   • `style/tests.rs`  — invariants across all defaults + roles.

pub mod anim;
pub mod color;
pub mod effect;
pub mod elevation;
pub mod font;
pub mod metric;
pub mod radius;
pub mod uv;

pub use anim::BAnimColors;
pub use uv::BUvColors;
pub use color::BResolvedColors;
pub use effect::{
    APPLE_EFFECTS, BARBIE_EFFECTS, BEffects, BShadowStyle, CHANEL_EFFECTS, CYBERPUNK_EFFECTS,
    FENTY_EFFECTS, GRAPHITE_EFFECTS, GREY_EFFECTS, ONYX_EFFECTS, SPHERE_EFFECTS, SWIFT_EFFECTS,
    TOKYO_EFFECTS,
};
pub use elevation::BElevation;
pub use font::BFontFamily;
pub use metric::{
    BDimensions, BResizeStyle, BSpacing, BStrokes, BTypography, DEFAULT_DIMENSIONS,
    DEFAULT_RESIZE, DEFAULT_SPACING, DEFAULT_STROKES, DEFAULT_TYPOGRAPHY, HEAD_ROOM};
pub use radius::{
    BRadiusScale, BRadiusToken, BRoleMap, BStyleRole, DEFAULT_ROLES, MEDIUM_RADII, PILL_RADII,
    ROUND_RADII, SOFT_RADII, SQUARE_RADII, TIGHT_RADII,
};

use super::palette::BPaletteName;

#[cfg(test)]
mod tests;

// ── Aggregate defaults + resolved style ─────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BUiStyleDefaults {
    pub colors: BResolvedColors,
    pub radii: BRadiusScale,
    pub typography: BTypography,
    pub spacing: BSpacing,
    pub dimensions: BDimensions,
    pub strokes: BStrokes,
    pub resize: BResizeStyle,
    pub effects: BEffects,
    pub roles: BRoleMap,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BResolvedUiStyle {
    pub colors: BResolvedColors,
    pub radii: BRadiusScale,
    pub typography: BTypography,
    pub spacing: BSpacing,
    pub dimensions: BDimensions,
    pub strokes: BStrokes,
    pub resize: BResizeStyle,
    pub effects: BEffects,
    pub roles: BRoleMap,
}

impl BResolvedUiStyle {
    /// Resolves the current radius for a role from this style's live scale.
    pub const fn radius_for(&self, role: BStyleRole) -> f32 {
        self.roles.radius_for(role, &self.radii)
    }

    /// Restores cross-field resize invariants after independent overrides.
    ///
    /// Descriptor validation already guarantees finite, bounded fields. This
    /// final normalization only widens a conflicting hit target so the visible
    /// seam always remains fully interactive.
    pub(super) fn normalize_resize_geometry(&mut self) {
        // The seam's hovered thickness is clamped to the hit band where it
        // is drawn (`resize::BSeamLine`), so no field bends another here.
    }
}

impl From<&BUiStyleDefaults> for BResolvedUiStyle {
    fn from(defaults: &BUiStyleDefaults) -> Self {
        Self {
            colors: defaults.colors,
            radii: defaults.radii,
            typography: defaults.typography,
            spacing: defaults.spacing,
            dimensions: defaults.dimensions,
            strokes: defaults.strokes,
            resize: defaults.resize,
            effects: defaults.effects,
            roles: defaults.roles,
        }
    }
}

// ── Palette-driven defaults ─────────────────────────────────────────────

/// A theme is its palette plus the four things that differ between themes:
/// radius scale, effects (sweeps and shadow) and the seam gap. Typography,
/// spacing, dimensions, strokes and roles are the same in every theme, so a
/// header is the same height and a tab the same shape everywhere.
const fn defaults(
    palette: &super::palette::BPalette,
    radii: BRadiusScale,
    effects: BEffects,
    seam_gap: f32,
) -> BUiStyleDefaults {
    BUiStyleDefaults {
        colors: BResolvedColors::from_palette(palette),
        radii,
        typography: DEFAULT_TYPOGRAPHY,
        spacing: DEFAULT_SPACING,
        dimensions: DEFAULT_DIMENSIONS,
        strokes: DEFAULT_STROKES,
        resize: BResizeStyle::with_gap(seam_gap),
        effects,
        roles: DEFAULT_ROLES,
    }
}

use super::palette::BThemeMode;
use super::palette::theme::set as palette;

/// One row per theme: its dark and light palettes, and the radius scale,
/// effects and seam gap both modes share — a mode changes colours, nothing
/// else.
macro_rules! themes {
    ($($name:ident : $dark:ident / $light:ident = ($palette:ident, $palette_light:ident, $radii:ident, $effects:ident, $gap:expr);)+) => {
        $(
            pub static $dark: BUiStyleDefaults = defaults(&palette::$palette, $radii, $effects, $gap);
            pub static $light: BUiStyleDefaults = defaults(&palette::$palette_light, $radii, $effects, $gap);
        )+
        /// A theme's defaults in `mode`.
        pub fn ui_style_defaults_in(name: BPaletteName, mode: BThemeMode) -> &'static BUiStyleDefaults {
            match (name, mode) {
                $(
                    (BPaletteName::$name, BThemeMode::Dark) => &$dark,
                    (BPaletteName::$name, BThemeMode::Light) => &$light,
                )+
            }
        }
    };
}

/// A theme's defaults in its dark mode.
pub fn ui_style_defaults(name: BPaletteName) -> &'static BUiStyleDefaults {
    ui_style_defaults_in(name, BThemeMode::Dark)
}

// Every theme: its palettes, radius scale, effects and seam gap.
themes! {
    Sphere: SPHERE_UI_STYLE_DEFAULTS / SPHERE_LIGHT_UI_STYLE_DEFAULTS = (SPHERE, SPHERE_LIGHT, SOFT_RADII, SPHERE_EFFECTS, 4.0);
    Graphite: GRAPHITE_UI_STYLE_DEFAULTS / GRAPHITE_LIGHT_UI_STYLE_DEFAULTS = (GRAPHITE, GRAPHITE_LIGHT, TIGHT_RADII, GRAPHITE_EFFECTS, 4.0);
    Tokyo: TOKYO_UI_STYLE_DEFAULTS / TOKYO_LIGHT_UI_STYLE_DEFAULTS = (TOKYO, TOKYO_LIGHT, MEDIUM_RADII, TOKYO_EFFECTS, 6.0);
    Onyx: ONYX_UI_STYLE_DEFAULTS / ONYX_LIGHT_UI_STYLE_DEFAULTS = (ONYX, ONYX_LIGHT, MEDIUM_RADII, ONYX_EFFECTS, 4.0);
    Grey: GREY_UI_STYLE_DEFAULTS / GREY_LIGHT_UI_STYLE_DEFAULTS = (GREY, GREY_LIGHT, ROUND_RADII, GREY_EFFECTS, 6.0);
    Apple: APPLE_UI_STYLE_DEFAULTS / APPLE_LIGHT_UI_STYLE_DEFAULTS = (APPLE, APPLE_LIGHT, MEDIUM_RADII, APPLE_EFFECTS, 4.0);
    Swift: SWIFT_UI_STYLE_DEFAULTS / SWIFT_LIGHT_UI_STYLE_DEFAULTS = (SWIFT, SWIFT_LIGHT, SOFT_RADII, SWIFT_EFFECTS, 4.0);
    Barbie: BARBIE_UI_STYLE_DEFAULTS / BARBIE_LIGHT_UI_STYLE_DEFAULTS = (BARBIE, BARBIE_LIGHT, PILL_RADII, BARBIE_EFFECTS, 8.0);
    Fenty: FENTY_UI_STYLE_DEFAULTS / FENTY_LIGHT_UI_STYLE_DEFAULTS = (FENTY, FENTY_LIGHT, ROUND_RADII, FENTY_EFFECTS, 6.0);
    Cyberpunk: CYBERPUNK_UI_STYLE_DEFAULTS / CYBERPUNK_LIGHT_UI_STYLE_DEFAULTS = (CYBERPUNK, CYBERPUNK_LIGHT, SQUARE_RADII, CYBERPUNK_EFFECTS, 2.0);
    Chanel: CHANEL_UI_STYLE_DEFAULTS / CHANEL_LIGHT_UI_STYLE_DEFAULTS = (CHANEL, CHANEL_LIGHT, TIGHT_RADII, CHANEL_EFFECTS, 4.0);
}
