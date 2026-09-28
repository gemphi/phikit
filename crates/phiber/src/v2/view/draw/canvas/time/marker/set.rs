//! Painting the markers — `ED_markers_draw`: the band first, then for each
//! marker its stalk, its glyph and its name, in the theme's colours
//! (`BAnimColors::marker_line` / `marker` / `marker_text`).

use super::super::set::{hsla, line_x, rect, shape, text_at, triangle};
use super::super::BTimeView;
use super::{dashes, BAND_HEIGHT, GLYPH_FOOT, GLYPH_SIZE, NAME_GAP, STALK_FOOT};
use crate::v2::theme::BAnimColors;
use gpui::{App, Bounds, Pixels, Window, fill, point, px};

/// The glyph: `ICON_MARKER`, a wedge pointing up with its base on the line.
const GLYPH_HALF: f32 = GLYPH_SIZE * 0.3;
const GLYPH_TALL: f32 = GLYPH_SIZE * 0.5;

pub fn paint_markers(view: &BTimeView, colors: &BAnimColors, at: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
    if view.markers.is_empty() {
        return;
    }
    let width = f32::from(at.size.width);
    let height = f32::from(at.size.height);
    let top = BTimeView::marker_band_top(height);
    window.paint_quad(fill(rect(0.0, top, width, top + BAND_HEIGHT, at), hsla(colors.scrub_back)));
    // A box being swept: the band between its frames, in the selection's
    // shade, under the glyphs.
    if let Some((a, b)) = view.marker_box {
        let (x0, x1) = (view.x_of(a.min(b), width), view.x_of(a.max(b), width));
        window.paint_quad(fill(rect(x0, top, x1.max(x0 + 1.0), top + BAND_HEIGHT, at), hsla(colors.marker_line_selected)));
    }
    for glyph in view.marker_glyphs(width) {
        // A selected marker: `TH_TIME_MARKER_LINE_SELECTED`, the name in
        // `TH_TEXT_HI`, the glyph lit (`ICON_MARKER_HLT`).
        let (line_ink, glyph_ink, text_ink) = if glyph.selected {
            (colors.marker_line_selected, colors.marker_selected, colors.marker_text_selected)
        } else {
            (colors.marker_line, colors.marker, colors.marker_text)
        };
        // The stalk: dashed from the scrub's foot down to `STALK_FOOT`.
        for (y0, y1) in dashes(super::super::SCRUB_HEIGHT, height - STALK_FOOT) {
            line_x(glyph.x, y0, y1, 1.0, line_ink, at, window);
        }
        // The glyph, astride the frame, its base `GLYPH_FOOT` up.
        let base = height - GLYPH_FOOT;
        triangle(
            point(at.origin.x + px(glyph.x - GLYPH_HALF), at.origin.y + px(base)),
            point(at.origin.x + px(glyph.x + GLYPH_HALF + 1.0), at.origin.y + px(base)),
            point(at.origin.x + px(glyph.x + 0.5), at.origin.y + px(base - GLYPH_TALL)),
            glyph_ink,
            window,
        );
        // The name, to the right of the glyph, on the glyph's line.
        let line = shape(&glyph.name, text_ink, window);
        text_at(&line, glyph.x + GLYPH_HALF + NAME_GAP, base - GLYPH_TALL / 2.0, at, window, cx);
    }
}
