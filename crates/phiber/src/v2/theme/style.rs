//! Resolved-UI-style types + palette-defaults factory.
//!
//! Palette colors remain the completed color-token source; every other
//! design default is owned here and palette colors are copied into resolved
//! colors.
//!
//! Split for the 200-line file cap:
//!   • `style.rs` (this file) — aggregate `BUiStyleDefaults`/`BResolvedUiStyle`
//!     plus the palette-driven defaults factory + re-exports.
//!   • `style/radii.rs`   — radius scale, tokens, role-map.
//!   • `style/metrics.rs` — typography / spacing / dimensions / strokes / resize.
//!   • `style/effects.rs` — shadow style + gradient/lift/etc effects.
//!   • `style/colors.rs`  — `BResolvedColors` + palette→colours copy.
//!   • `style/tests.rs`   — invariants across all defaults + roles.

pub mod colors;
pub mod effects;
pub mod metrics;
pub mod radii;

pub use colors::BResolvedColors;
pub use effects::{APPLE_EFFECTS, BEffects, BLENDER_EFFECTS, BShadowStyle, TOKYO_NIGHT_EFFECTS};
pub use metrics::{
    BDimensions, BResizeStyle, BSpacing, BStrokes, BTypography, DEFAULT_DIMENSIONS,
    DEFAULT_RESIZE, DEFAULT_SPACING, DEFAULT_STROKES, DEFAULT_TYPOGRAPHY,
};
pub use radii::{
    APPLE_RADII, BRadiusScale, BRadiusToken, BRoleMap, BStyleRole, DEFAULT_ROLES, TIGHT_RADII,
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
        self.resize.hit_width = self.resize.hit_width.max(self.resize.visible_seam);
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

const fn defaults(
    palette: &super::palette::BPalette,
    radii: BRadiusScale,
    effects: BEffects,
) -> BUiStyleDefaults {
    BUiStyleDefaults {
        colors: BResolvedColors::from_palette(palette),
        radii,
        typography: DEFAULT_TYPOGRAPHY,
        spacing: DEFAULT_SPACING,
        dimensions: DEFAULT_DIMENSIONS,
        strokes: DEFAULT_STROKES,
        resize: DEFAULT_RESIZE,
        effects,
        roles: DEFAULT_ROLES,
    }
}

pub static APPLE_UI_STYLE_DEFAULTS: BUiStyleDefaults =
    defaults(&super::palette::set::APPLE_DARK, APPLE_RADII, APPLE_EFFECTS);
pub static BLENDER_UI_STYLE_DEFAULTS: BUiStyleDefaults = defaults(
    &super::palette::set::BLENDER_DARK,
    TIGHT_RADII,
    BLENDER_EFFECTS,
);
pub static TOKYO_NIGHT_UI_STYLE_DEFAULTS: BUiStyleDefaults = defaults(
    &super::palette::set::TOKYO_NIGHT,
    TIGHT_RADII,
    TOKYO_NIGHT_EFFECTS,
);

pub fn ui_style_defaults(name: BPaletteName) -> &'static BUiStyleDefaults {
    match name {
        BPaletteName::AppleDark => &APPLE_UI_STYLE_DEFAULTS,
        BPaletteName::BlenderDark => &BLENDER_UI_STYLE_DEFAULTS,
        BPaletteName::TokyoNight => &TOKYO_NIGHT_UI_STYLE_DEFAULTS,
    }
}
