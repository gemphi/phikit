//! `BResizeStyle` — the gutter width, the hit zone, the hairline. Split
//! from `metric.rs` so each file sits under the 240-line read cap
//! (`tests/level_structure.rs`).

use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BResizeStyle {
    /// The seam is this much thinner than the gap at rest…
    pub seam_rest_under: f32,
    /// …and this much thicker than it under the pointer.
    pub seam_hover_over: f32,
    pub hit_width: f32,
    pub gap: f32,
    pub inset: f32,
    pub snap_threshold: f32,
    pub min_extent: f32,
}

// The gap between stacked slots (Blender's `RGN_ALIGN_*` seam), one value
// for every seam — zone, area, panel, modal — and the transparent drag hit
// band over it, wide enough to grab without pixel-hunting.
const RESIZE_GAP: f32 = 4.0;
const RESIZE_HIT_WIDTH: f32 = 14.0;

impl BResizeStyle {
    /// The resize style for a theme whose seams are `gap` wide.
    pub const fn with_gap(gap: f32) -> Self {
        Self {
            // The seam sits inside the gap at rest (gap − 3) and swells past
            // it under the pointer (gap + 2): a line that answers the hand.
            seam_rest_under: 3.0,
            seam_hover_over: 2.0,
            hit_width: RESIZE_HIT_WIDTH,
            gap,
            // The hit band is drawn inside the lower slot at `.top(inset)`,
            // so centring it on the gap's midpoint (`slot.top − gap/2`)
            // gives `inset = −(hit_width + gap) / 2`: −9 for 14 and 4.
            inset: -(RESIZE_HIT_WIDTH + gap) / 2.0,
            snap_threshold: 16.0,
            min_extent: 60.0,
        }
    }

    /// The seam's thickness at rest: never below a hairline.
    pub const fn rest_thickness(&self) -> f32 {
        let thick = self.gap - self.seam_rest_under;
        if thick < 1.0 { 1.0 } else { thick }
    }

    /// The seam's thickness under the pointer.
    pub const fn hover_thickness(&self) -> f32 {
        let thick = self.gap + self.seam_hover_over;
        let rest = self.rest_thickness();
        if thick < rest { rest } else { thick }
    }
}

pub const DEFAULT_RESIZE: BResizeStyle = BResizeStyle::with_gap(RESIZE_GAP);
