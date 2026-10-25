//! Color palettes — one static per theme, each a shade ladder
//! (`shade/`) read at fixed steps plus the few colours that carry meaning
//! and sit off the ladder (the accent, the axes, the status green, the
//! search match). The sixteen themes themselves are `theme/set.rs`; radius
//! and effect defaults are selected centrally by palette name.

use super::BPaletteName;
use super::shade::BShadeLadder;
use crate::v2::part::BPartType;
use crate::v2::spec::{BSpec, BSpecKind};

pub type BColor = (f32, f32, f32, f32);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BPalette {
    pub name: BPaletteName,
    /// The ladder every surface token below is a step of.
    pub ladder: BShadeLadder,

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
    /// A control at rest — a field, a dropdown, a button, a menu's body.
    pub bg_control: BColor,
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
    /// The rule under a heading — a header strip, a card's title, a
    /// section: two steps past the outline, so a heading is closed by a line
    /// you see rather than a shade you find by looking for it.
    pub rule: BColor,
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

/// Where each slot sits on the ladder — the same step in every theme, so a
/// header is a header and a canvas a canvas whatever the seed. The ladder
/// climbs inward as Blender's does (window `#181818`, header and region
/// `#303030`, panel `#3D3D3D`, field `#545454`): the chrome darkest, then
/// a host's header, then its body a step lighter, its cards a step over
/// that, its controls a step over those, hover and active beyond.
pub const SHADE_DIM: u8 = 1;
pub const SHADE_APP: u8 = 2;
pub const SHADE_CHROME: u8 = 3;
pub const SHADE_HEADER: u8 = 4;
pub const SHADE_BODY: u8 = 5;
pub const SHADE_CARD: u8 = 6;
pub const SHADE_CONTROL: u8 = 7;
pub const SHADE_HOVER: u8 = 8;
pub const SHADE_ACTIVE: u8 = 9;
pub const SHADE_OUTLINE: u8 = 9;
pub const SHADE_CANVAS: u8 = 10;
/// The grid on a canvas: the step inward of it, as Blender's viewport grid
/// `#545454` sits over its `#3D3D3D` (`TH_GRID` over `TH_BACK`).
pub const SHADE_GRID: u8 = 11;
pub const SHADE_DISABLED: u8 = 11;
pub const SHADE_TEXT_DIM: u8 = 12;
pub const SHADE_PLACEHOLDER: u8 = 13;
pub const SHADE_ICON: u8 = 14;
pub const SHADE_VALUE: u8 = 15;
pub const SHADE_TEXT: u8 = 16;
pub const SHADE_HEADING: u8 = 17;
pub const SHADE_TEXT_HI: u8 = 18;
/// The modal dim's alpha and the soft accent's.
const DIM_ALPHA: f32 = 0.65;
const ACCENT_SOFT_ALPHA: f32 = 0.35;

/// A selected row is the theme's accent at low strength, and the active one
/// among the selected the same accent at nearly double it — the way Blender
/// fills `TH_SELECT` and `TH_ACTIVE` with its selection colour rather than
/// with a step of the region's own shade.
///
/// These two were ladder shades 8 and 9, which are also `bg_hover` and
/// `bg_active`: a selected row was painted in exactly the hover colour, so
/// hovering an unselected row made it look selected, and a selected row was
/// indistinguishable from an active control. Selection means something, so
/// like the accent and the axes it stays off the ladder.
const ROW_SELECTED_ALPHA: f32 = 0.28;
const ROW_ACTIVE_ALPHA: f32 = 0.46;

/// The colours a theme keeps off its ladder because they mean something.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BAccents {
    pub accent: BColor,
    pub online: BColor,
    pub row_match: BColor,
    pub axis_x: BColor,
    pub axis_y: BColor,
    pub axis_z: BColor,
    pub wire_active: BColor,
}

/// Blender's axis reds, greens and blues, its active-wire orange, and the
/// search-match yellow every theme shares.
pub const BLENDER_MEANINGS: BAccents = BAccents {
    accent: (211.0, 60.0, 55.0, 1.0),
    online: (142.0, 60.0, 48.0, 1.0),
    row_match: (45.0, 70.0, 45.0, 0.50),
    axis_x: (348.6, 100.0, 60.0, 1.0),
    axis_y: (81.0, 100.0, 43.0, 1.0),
    axis_z: (211.0, 100.0, 58.0, 1.0),
    wire_active: (27.6, 100.0, 53.0, 1.0),
};

impl BPalette {
    /// A theme from its ladder: every surface, text and structural token a
    /// fixed step, the meanings passed in.
    pub const fn on(name: BPaletteName, ladder: BShadeLadder, meanings: BAccents) -> Self {
        let (hue, saturation, lightness, _) = meanings.accent;
        Self {
            name,
            ladder,
            bg_app: ladder.color(SHADE_APP),
            bg_activity: ladder.color(SHADE_CHROME),
            bg_top: ladder.color(SHADE_CHROME),
            bg_bottom: ladder.color(SHADE_CHROME),
            bg_left: ladder.color(SHADE_BODY),
            bg_right: ladder.color(SHADE_BODY),
            bg_header: ladder.color(SHADE_HEADER),
            bg_panel: ladder.color(SHADE_BODY),
            bg_viewport: ladder.color(SHADE_CANVAS),
            bg_control: ladder.color(SHADE_CONTROL),
            bg_hover: ladder.color(SHADE_HOVER),
            bg_active: ladder.color(SHADE_ACTIVE),
            bg_modal_dim: ladder.at(SHADE_DIM, DIM_ALPHA),
            text: ladder.color(SHADE_TEXT),
            text_dim: ladder.color(SHADE_TEXT_DIM),
            accent: meanings.accent,
            accent_soft: (hue, saturation, lightness, ACCENT_SOFT_ALPHA),
            online: meanings.online,
            border: ladder.color(SHADE_OUTLINE),
            rule: ladder.color(SHADE_GRID),
            sep: ladder.color(SHADE_APP),
            row_alt: ladder.color(SHADE_CARD),
            row_selected: (hue, saturation, lightness, ROW_SELECTED_ALPHA),
            row_active: (hue, saturation, lightness, ROW_ACTIVE_ALPHA),
            row_match: meanings.row_match,
            viewport_grid: ladder.color(SHADE_GRID),
            axis_x: meanings.axis_x,
            axis_y: meanings.axis_y,
            axis_z: meanings.axis_z,
            wire_active: meanings.wire_active,
        }
    }
}

impl BPalette {
    /// A theme that differs from Blender's meanings only in its accent, given
    /// as (hue, saturation, lightness).
    pub const fn accented(name: BPaletteName, ladder: BShadeLadder, accent: (f32, f32, f32)) -> Self {
        let (hue, saturation, lightness) = accent;
        Self::on(name, ladder, BAccents { accent: (hue, saturation, lightness, 1.0), ..BLENDER_MEANINGS })
    }
}

impl BSpec for BPalette {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Part(BPartType::Custom)
    }
    fn label(&self) -> &'static str {
        self.name.label()
    }
}
