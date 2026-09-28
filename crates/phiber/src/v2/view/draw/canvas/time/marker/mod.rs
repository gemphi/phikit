//! Timeline markers as `anim_markers.cc` lays them out: a band along the
//! foot of the window (`UI_MARKER_MARGIN_Y`, in `TH_TIME_SCRUB_BACKGROUND`)
//! once the scene has any, and in it, per marker, a dashed stalk up the
//! window (`draw_marker_line`: six-pixel dashes, half drawn), the marker
//! glyph (`ICON_MARKER`) astride the frame, and the name to its right
//! (`draw_marker_name`). Two markers on one pixel column draw once.
//! Structure here; painting is `set.rs`.

pub mod set;

use super::BTimeView;

/// `UI_MARKER_MARGIN_Y` at scale 1: the band's height.
pub const BAND_HEIGHT: f32 = 42.0;
/// `draw_marker`: the stalk rises from 20 px above the foot; the glyph and
/// the name sit 18 px above it.
pub const STALK_FOOT: f32 = 20.0;
pub const GLYPH_FOOT: f32 = 18.0;
/// `UI_ICON_SIZE`: the glyph is drawn `0.55 × ICON_SIZE` left of the frame.
pub const GLYPH_SIZE: f32 = 16.0;
/// `draw_marker_name`: the name starts `4 px` past the glyph's right edge.
pub const NAME_GAP: f32 = 4.0;
/// `draw_marker_line`: the dash period and the part of it drawn.
pub const DASH: f32 = 6.0;
pub const DASH_DRAWN: f32 = 0.5;

/// One marker as it sits on the canvas.
#[derive(Clone, Debug, PartialEq)]
pub struct BMarkerGlyph {
    pub x: f32,
    pub frame: f32,
    pub name: String,
    pub selected: bool,
}

impl BTimeView {
    /// The band's top edge, `height` down the canvas.
    pub fn marker_band_top(height: f32) -> f32 {
        (height - BAND_HEIGHT).max(super::SCRUB_HEIGHT)
    }

    /// The markers across a canvas `width` wide — those inside the view,
    /// one per pixel column, in frame order.
    pub fn marker_glyphs(&self, width: f32) -> Vec<BMarkerGlyph> {
        let mut out: Vec<BMarkerGlyph> = Vec::new();
        for (frame, name, selected) in &self.markers {
            if *frame < self.first || *frame > self.last {
                continue;
            }
            let x = self.x_of(*frame, width).floor();
            if out.last().is_some_and(|last| last.x == x) {
                continue;
            }
            out.push(BMarkerGlyph { x, frame: *frame, name: name.clone(), selected: *selected });
        }
        out
    }

    /// The marker under `(x, y)` on a canvas `width × height`: in the band,
    /// within a glyph's width of its frame (`ed_marker_select`'s
    /// `find_nearest_marker`, which takes the nearest within a few pixels).
    pub fn marker_hit(&self, x: f32, y: f32, width: f32, height: f32) -> Option<BMarkerGlyph> {
        if self.markers.is_empty() || y < Self::marker_band_top(height) {
            return None;
        }
        let reach = GLYPH_SIZE * 0.5;
        self.marker_glyphs(width)
            .into_iter()
            .filter(|g| (g.x - x).abs() <= reach)
            .min_by(|a, b| (a.x - x).abs().total_cmp(&(b.x - x).abs()))
    }

    /// Whether `(x, y)` is in the marker band at all.
    pub fn in_marker_band(&self, y: f32, height: f32) -> bool {
        !self.markers.is_empty() && y >= Self::marker_band_top(height)
    }
}

/// The dashes of a stalk from `top` down to `foot`: `(y0, y1)` pairs,
/// `DASH_DRAWN` of each `DASH` period, the last one clipped at the foot.
pub fn dashes(top: f32, foot: f32) -> Vec<(f32, f32)> {
    let mut out = Vec::new();
    let mut y = top;
    while y < foot {
        out.push((y, (y + DASH * DASH_DRAWN).min(foot)));
        y += DASH;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markers_in_view_get_a_glyph_each_column_once_and_stalks_dash_by_six() {
        let mut view = BTimeView::of_range(1, 250, 24.0);
        view.markers = vec![(-30.0, "gone".into(), false), (12.0, "Intro".into(), true), (12.0, "Twin".into(), false), (100.0, "Cut".into(), false), (300.0, "past".into(), false)];
        let glyphs = view.marker_glyphs(1000.0);
        assert_eq!(glyphs.iter().map(|g| g.name.as_str()).collect::<Vec<_>>(), ["Intro", "Cut"], "only those in view, one per column");
        assert_eq!(glyphs[0].x, view.x_of(12.0, 1000.0).floor());
        assert!(glyphs[0].selected && !glyphs[1].selected);
        // A click in the band near a glyph finds it; above the band, or far from one, nothing.
        let cut_x = view.x_of(100.0, 1000.0);
        assert_eq!(view.marker_hit(cut_x + 3.0, 390.0, 1000.0, 400.0).map(|g| g.name), Some("Cut".to_string()));
        assert_eq!(view.marker_hit(cut_x + 3.0, 100.0, 1000.0, 400.0), None, "above the band");
        assert_eq!(view.marker_hit(cut_x + 40.0, 390.0, 1000.0, 400.0), None, "too far");
        assert!(view.in_marker_band(390.0, 400.0) && !view.in_marker_band(100.0, 400.0));
        assert_eq!(BTimeView::marker_band_top(400.0), 400.0 - BAND_HEIGHT);
        assert_eq!(BTimeView::marker_band_top(30.0), super::super::SCRUB_HEIGHT, "never over the scrub");
        let d = dashes(0.0, 20.0);
        assert_eq!(d, [(0.0, 3.0), (6.0, 9.0), (12.0, 15.0), (18.0, 20.0)], "half of every six pixels, clipped at the foot");
    }
}
