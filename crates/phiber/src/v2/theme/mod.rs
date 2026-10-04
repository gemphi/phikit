//! Theme defaults, resolved style domains, and completed color palettes.

pub mod descriptor;
pub mod palette;
pub mod resolver;
pub mod semantic;
pub mod set;
pub mod style;
/// The bridge to the vendored `theme` crate (the only one in phiber).
pub mod vendor;

pub use descriptor::{
    BPreferenceControl, BPreferenceMetadata, BStyleChoice, BStyleDescriptor, BStyleParseError,
    BStyleParser, BStyleValue, UI_STYLE_DESCRIPTORS,
};
pub use palette::{BAccents, BColor, BPalette, BPaletteName, BShadeLadder, BThemeMode, palette_for, palette_in};
pub use resolver::resolve_ui_style;
pub use semantic::{
    BCheckerColors, BControlState, BGradientAxis, BGradientColors, BStrokeStyle, BSurfaceTone, SWEEP_STOP,
};
pub use set::{BMetric, METRICS};
pub use style::{
    BAnimColors, BDimensions, BEffects, BElevation, BFontFamily, BRadiusScale, BRadiusToken, BResizeStyle,
    BResolvedColors, BResolvedUiStyle, BRoleMap, BShadowStyle, BSpacing, BStrokes, BStyleRole,
    BTypography, BUiStyleDefaults, BUvColors, MEDIUM_RADII, PILL_RADII, ROUND_RADII, SOFT_RADII,
    SQUARE_RADII, TIGHT_RADII, HEAD_ROOM, BNodeClass, class_color, class_of, socket_color, ui_style_defaults, ui_style_defaults_in,
};

/// The theme and mode the state names — `ctx.theme` and `ctx.theme_mode`
/// — or the defaults: what the window draws with, what Preferences shows.
pub fn theme_in_force(state: &crate::v2::state::BState) -> (BPaletteName, BThemeMode) {
    use crate::v2::state::AppKey;
    let name = state.get(AppKey::Theme).and_then(BPaletteName::from_slug).unwrap_or(BPaletteName::DEFAULT);
    let mode = state.get(AppKey::ThemeMode).and_then(BThemeMode::from_slug).unwrap_or(BThemeMode::DEFAULT);
    (name, mode)
}

/// The defaults of the theme in force, in its mode.
pub fn ui_style_defaults_for(state: &crate::v2::state::BState) -> &'static BUiStyleDefaults {
    let (name, mode) = theme_in_force(state);
    ui_style_defaults_in(name, mode)
}
