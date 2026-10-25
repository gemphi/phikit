//! Resolved-colour surface — one `BColor` per semantic slot. Split out of
//! `style.rs` so the parent stays under the 200-line cap.

use crate::v2::theme::palette::{BColor, BPalette, BShadeLadder};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BResolvedColors {
    /// The ladder the surface tokens are steps of: what `shade` and `step`
    /// move along.
    pub ladder: BShadeLadder,
    pub bg_app: BColor,
    pub bg_activity: BColor,
    pub bg_top: BColor,
    pub bg_bottom: BColor,
    pub bg_left: BColor,
    pub bg_right: BColor,
    pub bg_header: BColor,
    pub bg_panel: BColor,
    pub bg_viewport: BColor,
    pub bg_control: BColor,
    pub bg_hover: BColor,
    pub bg_active: BColor,
    pub bg_modal_dim: BColor,
    pub text: BColor,
    pub text_dim: BColor,
    pub accent: BColor,
    pub accent_soft: BColor,
    /// The brand mark's two tones. A logo keeps its own colours in every
    /// palette — that is what makes it a logo — so these are fixed rather
    /// than derived from the theme's accent.
    pub brand_warm: BColor,
    pub brand_cool: BColor,
    pub online: BColor,
    pub border: BColor,
    pub rule: BColor,
    pub sep: BColor,
    pub row_alt: BColor,
    pub row_selected: BColor,
    pub row_active: BColor,
    pub row_match: BColor,
    pub viewport_grid: BColor,
    pub axis_x: BColor,
    pub axis_y: BColor,
    pub axis_z: BColor,
    pub wire_active: BColor,
}

impl BResolvedColors {
    pub const fn from_palette(p: &BPalette) -> Self {
        Self {
            ladder: p.ladder,
            bg_app: p.bg_app,
            bg_activity: p.bg_activity,
            bg_top: p.bg_top,
            bg_bottom: p.bg_bottom,
            bg_left: p.bg_left,
            bg_right: p.bg_right,
            bg_header: p.bg_header,
            bg_panel: p.bg_panel,
            bg_viewport: p.bg_viewport,
            bg_control: p.bg_control,
            bg_hover: p.bg_hover,
            bg_active: p.bg_active,
            bg_modal_dim: p.bg_modal_dim,
            text: p.text,
            text_dim: p.text_dim,
            accent: p.accent,
            accent_soft: p.accent_soft,
            // Phiber's orange and blue, in every theme.
            brand_warm: (38.0, 92.0, 50.0, 1.0),
            brand_cool: (217.0, 91.0, 60.0, 1.0),
            online: p.online,
            border: p.border,
            rule: p.rule,
            sep: p.sep,
            row_alt: p.row_alt,
            row_selected: p.row_selected,
            row_active: p.row_active,
            row_match: p.row_match,
            viewport_grid: p.viewport_grid,
            axis_x: p.axis_x,
            axis_y: p.axis_y,
            axis_z: p.axis_z,
            wire_active: p.wire_active,
        }
    }
}
