//! Theme defaults, resolved style domains, and completed color palettes.

pub mod descriptor;
pub mod palette;
pub mod resolver;
pub mod semantic;
pub mod set;
pub mod style;

pub use descriptor::{
    BPreferenceControl, BPreferenceMetadata, BStyleChoice, BStyleDescriptor, BStyleParseError,
    BStyleParser, BStyleValue, UI_STYLE_DESCRIPTORS,
};
pub use palette::{BColor, BPalette, BPaletteName, palette_for};
pub use resolver::resolve_ui_style;
pub use semantic::{BCheckerColors, BGradientColors, BStrokeStyle, BSurfaceTone};
pub use set::{BMetric, METRICS};
pub use style::{
    APPLE_RADII, BDimensions, BEffects, BRadiusScale, BRadiusToken, BResizeStyle, BResolvedColors,
    BResolvedUiStyle, BRoleMap, BShadowStyle, BSpacing, BStrokes, BStyleRole, BTypography,
    BUiStyleDefaults, TIGHT_RADII, ui_style_defaults,
};
