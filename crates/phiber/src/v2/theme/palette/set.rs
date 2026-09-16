//! Color palettes — one static per theme. `AppleDark` (blue-grey) is the
//! default; Blender Dark and Tokyo Night follow.
//! Radius and effect defaults are selected centrally by palette name; the
//! palette itself owns only the completed color-token data.

use super::BPaletteName;
use crate::v2::part::BPartType;
use crate::v2::spec::{BSpec, BSpecKind};

pub type BColor = (f32, f32, f32, f32);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BPalette {
    pub name: BPaletteName,

    // Backgrounds
    pub bg_app: BColor,
    pub bg_activity: BColor,
    pub bg_top: BColor,
    pub bg_bottom: BColor,
    pub bg_left: BColor,
    pub bg_right: BColor,
    pub bg_header: BColor,
    pub bg_panel: BColor,
    pub bg_viewport: BColor,
    pub bg_hover: BColor,
    pub bg_active: BColor,
    pub bg_modal_dim: BColor,

    // Text
    pub text: BColor,
    pub text_dim: BColor,

    // Accents
    pub accent: BColor,
    pub accent_soft: BColor,
    pub online: BColor,

    // Structural
    pub border: BColor,
    pub sep: BColor,

    // Rows (outliner / file list / any `ListRow`)
    /// Alternating band drawn on every other row.
    pub row_alt: BColor,
    /// Selected-but-not-active row fill.
    pub row_selected: BColor,
    /// Active (selected + focused) row fill.
    pub row_active: BColor,
    /// Search-match highlight.
    pub row_match: BColor,

    // Viewport / canvas
    pub viewport_grid: BColor,
    pub axis_x: BColor,
    pub axis_y: BColor,
    pub axis_z: BColor,
    pub wire_active: BColor,
}

impl BSpec for BPalette {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Part(BPartType::Custom)
    }
    fn label(&self) -> &'static str {
        self.name.label()
    }
}

/// Apple Dark — the original phiber blue-grey. Hue 215 chrome with an 211
/// selection blue, restored from `bui::BPalette::apple_dark()` as committed in
/// the reference zed-repo (`crates/phiber/src/bui/theme.rs`, "Apple dark theme
/// — blue-grey, cooler tones with smooth, generous curves").
pub static APPLE_DARK: BPalette = BPalette {
    name: BPaletteName::AppleDark,
    bg_app: (215.0, 15.0, 10.0, 1.0),
    bg_activity: (215.0, 14.0, 12.0, 1.0),
    bg_top: (215.0, 18.0, 14.0, 1.0),
    bg_bottom: (215.0, 16.0, 12.0, 1.0),
    bg_left: (215.0, 14.0, 15.0, 1.0),
    bg_right: (215.0, 14.0, 15.0, 1.0),
    bg_header: (215.0, 16.0, 18.0, 1.0),
    bg_panel: (215.0, 14.0, 16.0, 1.0),
    // Original hue/saturation (220°, 20%) at Blender's viewport lightness, so
    // the viewport keeps the blue-grey cast without going near-black.
    bg_viewport: (220.0, 20.0, 24.0, 1.0),
    // Hover / active / row-state colours stay in the parent tonal family
    // (hue 215, low saturation like bg_panel) instead of drifting toward the
    // accent hue (211). Selection is signalled by a small lightness lift +
    // an accent border painted on top by the widget, not by tinting the
    // resting background — this matches the shader / boxed-slot pattern used
    // elsewhere in the app and kills the "foreign blue" outliner rows.
    bg_hover: (215.0, 12.0, 22.0, 1.0),
    bg_active: (215.0, 14.0, 28.0, 1.0),
    bg_modal_dim: (0.0, 0.0, 0.0, 0.65),
    text: (0.0, 0.0, 90.0, 1.0),
    text_dim: (215.0, 8.0, 55.0, 1.0),
    accent: (211.0, 60.0, 55.0, 1.0),
    accent_soft: (211.0, 60.0, 55.0, 0.35),
    online: (142.0, 60.0, 48.0, 1.0),
    border: (215.0, 10.0, 22.0, 1.0),
    sep: (215.0, 10.0, 20.0, 1.0),

    row_alt: (215.0, 12.0, 19.0, 1.0),
    row_selected: (215.0, 13.0, 25.0, 1.0),
    row_active: (215.0, 15.0, 32.0, 1.0),
    row_match: (45.0, 70.0, 45.0, 0.50),

    viewport_grid: (215.0, 5.0, 34.0, 1.0),
    axis_x: (348.6, 100.0, 60.0, 1.0),
    axis_y: (81.0, 100.0, 43.0, 1.0),
    axis_z: (211.0, 100.0, 58.0, 1.0),
    wire_active: (27.6, 100.0, 53.0, 1.0),
};

pub static BLENDER_DARK: BPalette = BPalette {
    name: BPaletteName::BlenderDark,
    bg_app: (220.0, 13.0, 10.0, 1.0),
    bg_activity: (220.0, 15.0, 12.0, 1.0),
    bg_top: (220.0, 12.0, 18.0, 1.0),
    bg_bottom: (220.0, 12.0, 13.0, 1.0),
    bg_left: (220.0, 12.0, 15.0, 1.0),
    bg_right: (220.0, 12.0, 15.0, 1.0),
    bg_header: (220.0, 12.0, 18.0, 1.0),
    bg_panel: (220.0, 12.0, 16.0, 1.0),
    bg_viewport: (220.0, 5.0, 55.0, 1.0),
    bg_hover: (220.0, 10.0, 24.0, 1.0),
    bg_active: (220.0, 10.0, 28.0, 1.0),
    bg_modal_dim: (0.0, 0.0, 0.0, 0.65),
    text: (0.0, 0.0, 92.0, 1.0),
    text_dim: (0.0, 0.0, 55.0, 1.0),
    accent: (28.0, 95.0, 54.0, 1.0),
    accent_soft: (28.0, 95.0, 54.0, 0.35),
    online: (142.0, 70.0, 45.0, 1.0),
    border: (220.0, 10.0, 12.0, 1.0),
    sep: (220.0, 10.0, 14.0, 1.0),

    row_alt: (220.0, 10.0, 19.0, 1.0),
    row_selected: (220.0, 14.0, 26.0, 1.0),
    row_active: (28.0, 60.0, 34.0, 1.0),
    row_match: (45.0, 70.0, 45.0, 0.50),

    viewport_grid: (220.0, 4.0, 44.0, 1.0),
    axis_x: (348.6, 100.0, 60.0, 1.0),
    axis_y: (81.0, 100.0, 43.0, 1.0),
    axis_z: (211.0, 100.0, 58.0, 1.0),
    wire_active: (27.6, 100.0, 53.0, 1.0),
};

pub static TOKYO_NIGHT: BPalette = BPalette {
    name: BPaletteName::TokyoNight,
    bg_app: (232.0, 24.0, 11.0, 1.0),
    bg_activity: (232.0, 24.0, 14.0, 1.0),
    bg_top: (232.0, 20.0, 17.0, 1.0),
    bg_bottom: (232.0, 22.0, 12.0, 1.0),
    bg_left: (232.0, 22.0, 14.0, 1.0),
    bg_right: (232.0, 22.0, 14.0, 1.0),
    bg_header: (232.0, 20.0, 17.0, 1.0),
    bg_panel: (232.0, 22.0, 15.0, 1.0),
    bg_viewport: (232.0, 12.0, 54.0, 1.0),
    bg_hover: (232.0, 22.0, 22.0, 1.0),
    bg_active: (232.0, 22.0, 28.0, 1.0),
    bg_modal_dim: (0.0, 0.0, 0.0, 0.7),
    text: (215.0, 30.0, 90.0, 1.0),
    text_dim: (215.0, 15.0, 60.0, 1.0),
    accent: (217.0, 91.0, 60.0, 1.0),
    accent_soft: (217.0, 91.0, 60.0, 0.30),
    online: (145.0, 61.0, 60.0, 1.0),
    border: (232.0, 20.0, 8.0, 1.0),
    sep: (232.0, 20.0, 20.0, 1.0),

    row_alt: (232.0, 20.0, 18.0, 1.0),
    row_selected: (217.0, 40.0, 26.0, 1.0),
    row_active: (217.0, 70.0, 36.0, 1.0),
    row_match: (45.0, 70.0, 45.0, 0.50),

    viewport_grid: (232.0, 10.0, 40.0, 1.0),
    axis_x: (348.6, 100.0, 60.0, 1.0),
    axis_y: (81.0, 100.0, 43.0, 1.0),
    axis_z: (217.0, 100.0, 62.0, 1.0),
    wire_active: (27.6, 100.0, 53.0, 1.0),
};

pub fn palette_for(name: BPaletteName) -> &'static BPalette {
    match name {
        BPaletteName::AppleDark => &APPLE_DARK,
        BPaletteName::BlenderDark => &BLENDER_DARK,
        BPaletteName::TokyoNight => &TOKYO_NIGHT,
    }
}
