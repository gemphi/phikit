//! What a channel row paints — `draw_backdrops` and `draw_keyframes` in
//! `action_draw.cc`, and the channel region's names
//! (`action_channel_region_draw`) — in the theme's colours (`BAnimColors`).

use super::super::set::{hsla, rect, shape, text_at, triangle};
use super::{BRowKind, BTimeRow, CHANNEL_WIDTH, ROW_HEIGHT};
use crate::v2::theme::BAnimColors;
use gpui::{App, Bounds, PathBuilder, Pixels, Window, fill, point, px};

/// `channel_ui_data_init`: `icon_size = widget_unit / 2`.
const ICON_SIZE: f32 = super::super::WIDGET_UNIT * 0.5;
/// `anim_channels_defines.cc`: `ICON_WIDTH` is `0.85 × widget_unit`; a
/// channel under an ID outside the Action editor sits `0.7 × widget_unit`
/// in (`acf_generic_group_offset`); the name follows its widgets by 3.
const ICON_WIDTH: f32 = 0.85 * super::super::WIDGET_UNIT;
const CURVE_OFFSET: f32 = 0.7 * super::super::WIDGET_UNIT;
const NAME_PAD: f32 = 3.0;

/// The backdrops, then the diamonds, of every Dope Sheet row, in the
/// window `at` (`x_of` maps a frame to a pixel across it); a picked column
/// (`picked(row, frame)`) in `TH_KEYTYPE_KEYFRAME_SELECT`. The NLA's rows
/// are `strip/set.rs`.
pub fn paint_rows(rows: &[BTimeRow], colors: &BAnimColors, at: Bounds<Pixels>, x_of: &dyn Fn(f32) -> f32, picked: &dyn Fn(usize, f32) -> bool, scrub: f32, window: &mut Window) {
    let width = f32::from(at.size.width);
    for (index, row) in rows.iter().enumerate() {
        let (top, center) = (BTimeRow::top(index, ROW_HEIGHT, scrub), BTimeRow::center(index, ROW_HEIGHT, scrub));
        let back = match &row.kind {
            BRowKind::Summary { expanded: true } => Some(colors.summary_back),
            BRowKind::Summary { expanded: false } => None,
            BRowKind::Object { .. } => Some(colors.object_back),
            BRowKind::Curve { .. } => Some(colors.curve_back),
            BRowKind::Action { .. } | BRowKind::Track(_) | BRowKind::Driver { .. } => None,
        };
        if let Some(back) = back {
            window.paint_quad(fill(rect(0.0, top, width, top + ROW_HEIGHT, at), hsla(back)));
        }
        // The row's keys in three strokes — every border, the plain
        // fills, the picked fills — not two paths a key: a rig's rows hold
        // hundreds of keys, and each path is tessellated (Blender batches
        // its diamonds in one instanced draw, `draw_keyframe_shape`).
        let shown: Vec<(f32, bool)> = row.keys.iter().map(|&f| (x_of(f).round() + 0.5, picked(index, f))).filter(|(x, _)| *x > -ICON_SIZE && *x < width + ICON_SIZE).collect();
        let half = ICON_SIZE / 2.0;
        diamonds(shown.iter().map(|(x, _)| *x), center, half, colors.key_border, at, window);
        diamonds(shown.iter().filter(|(_, p)| !p).map(|(x, _)| *x), center, half - 1.0, colors.key, at, window);
        diamonds(shown.iter().filter(|(_, p)| *p).map(|(x, _)| *x), center, half - 1.0, colors.key_select, at, window);
    }
}

/// One path of diamonds `r` half-wide at each `x` on the line `y`, in one
/// colour.
fn diamonds(xs: impl Iterator<Item = f32>, y: f32, r: f32, color: crate::v2::theme::BColor, at: Bounds<Pixels>, window: &mut Window) {
    let mut path = PathBuilder::fill();
    let mut any = false;
    for x in xs {
        let corner = |dx: f32, dy: f32| point(at.origin.x + px(x + dx), at.origin.y + px(y + dy));
        path.move_to(corner(0.0, -r));
        path.line_to(corner(r, 0.0));
        path.line_to(corner(0.0, r));
        path.line_to(corner(-r, 0.0));
        path.close();
        any = true;
    }
    if let (true, Ok(path)) = (any, path.build()) {
        window.paint_path(path, hsla(color));
    }
}

/// `draw_keyframe_shape`: a diamond `size` across, its border a pixel,
/// filled with `inner` — the key colour, or the band behind it for an
/// outline-only `KEYFRAME_SHAPE_FRAME`.
pub fn diamond(x: f32, y: f32, size: f32, inner: crate::v2::theme::BColor, colors: &BAnimColors, at: Bounds<Pixels>, window: &mut Window) {
    let half = size / 2.0;
    for (r, color) in [(half, colors.key_border), (half - 1.0, inner)] {
        let corner = |dx: f32, dy: f32| point(at.origin.x + px(x + dx), at.origin.y + px(y + dy));
        let mut path = PathBuilder::fill();
        path.move_to(corner(0.0, -r));
        path.line_to(corner(r, 0.0));
        path.line_to(corner(0.0, r));
        path.line_to(corner(-r, 0.0));
        path.close();
        if let Ok(path) = path.build() {
            window.paint_path(path, hsla(color));
        }
    }
}

/// The channel region: its background, each row's backdrop where Blender
/// gives it one (`acf_generic_root_backdrop` for an object, the action
/// line's own colour, a track's), and each row's name on its line — a
/// selected track's in the selection's colour, as its strips are
/// (`TH_TEXT_HI` in Blender).
pub fn paint_channels(rows: &[BTimeRow], row_height: f32, scrub: f32, colors: &BAnimColors, at: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
    let height = f32::from(at.size.height);
    window.paint_quad(fill(rect(0.0, 0.0, CHANNEL_WIDTH, height, at), hsla(colors.channel_back)));
    let indent = super::super::WIDGET_UNIT * 0.5;
    for (index, row) in rows.iter().enumerate() {
        let top = BTimeRow::top(index, row_height, scrub);
        let back = match &row.kind {
            BRowKind::Object { .. } if row_height > ROW_HEIGHT => Some(colors.object_back),
            BRowKind::Action { name, tweaking } => Some(super::super::strip::set::action_color(name.is_some(), *tweaking, colors)),
            BRowKind::Track(_) => Some(colors.track_back),
            BRowKind::Driver { .. } | BRowKind::Curve { .. } => Some(colors.curve_back),
            BRowKind::Summary { .. } | BRowKind::Object { .. } => None,
        };
        if let Some(back) = back {
            window.paint_quad(fill(rect(0.0, top, CHANNEL_WIDTH, top + row_height, at), hsla(back)));
        }
        // `ANIM_channel_draw` in the Graph editor: an F-curve's colour
        // swatch, then its visibility toggle's width, before the name.
        if let BRowKind::Driver { color, .. } | BRowKind::Curve { color, .. } = &row.kind {
            let swatch = gpui::Rgba { r: color[0], g: color[1], b: color[2], a: 1.0 };
            window.paint_quad(fill(rect(CURVE_OFFSET, top, CURVE_OFFSET + ICON_WIDTH, top + row_height, at), swatch));
        }
        // `ANIM_channel_draw` step 2: a channel that CAN expand reserves
        // `ICON_WIDTH` for its widget; one that cannot, and is not an F-curve,
        // pads `0.2 × widget_unit` instead; an F-curve gets neither, because
        // "F-Curves are not containers" (`acf_fcurve_setting_valid`).
        if let BRowKind::Object { expanded, .. } = &row.kind {
            expander(*expanded, indent, BTimeRow::center(index, row_height, scrub), colors, at, window);
        }
        let ink = match &row.kind {
            BRowKind::Track(track) if track.flag & crate::phiphx::blend::TRACK_SELECTED != 0 => colors.strip_select,
            _ => colors.channel_text,
        };
        let line = shape(row.name(), ink, window);
        let x = match row.kind {
            BRowKind::Summary { .. } | BRowKind::Object { .. } if row_height > ROW_HEIGHT => indent,
            BRowKind::Summary { .. } => indent,
            // Past the expander widget this row reserves, then its gap.
            BRowKind::Object { .. } => indent + ICON_WIDTH + NAME_PAD,
            BRowKind::Action { .. } | BRowKind::Track(_) => indent * 2.0,
            // `acf_generic_group_offset`: a curve under an ID sits one step in
            // from the channel it belongs to, past its swatch and toggle.
            BRowKind::Driver { .. } | BRowKind::Curve { .. } => CURVE_OFFSET + ICON_WIDTH * 2.0 + NAME_PAD,
        };
        text_at(&line, x, BTimeRow::center(index, row_height, scrub), at, window, cx);
    }
}

/// The expand widget — `ICON_TRIA_DOWN` when open, `ICON_TRIA_RIGHT` when shut
/// (`anim_channels_defines.cc:5885`), drawn in the `ICON_WIDTH` slot
/// `ANIM_channel_draw` reserves for it.
///
/// Blender draws this as a real UI widget rather than a glyph, so what transfers
/// is the pair of shapes and the slot, not a font character: a right-pointing
/// triangle for shut and a down-pointing one for open, which is the convention
/// every tree in the program uses.
fn expander(expanded: bool, x: f32, mid: f32, colors: &BAnimColors, at: Bounds<Pixels>, window: &mut Window) {
    // Sized off the icon slot so it tracks `widget_unit` with everything else.
    let half = ICON_SIZE * 0.5;
    let centre = x + ICON_WIDTH * 0.5;
    let point = |dx: f32, dy: f32| gpui::point(at.origin.x + px(centre + dx), at.origin.y + px(mid + dy));
    let (a, b, c) = if expanded {
        // Down: the flat edge on top, the tip below.
        (point(-half, -half * 0.6), point(half, -half * 0.6), point(0.0, half * 0.8))
    } else {
        // Right: the flat edge at the left, the tip to the right.
        (point(-half * 0.6, -half), point(-half * 0.6, half), point(half * 0.8, 0.0))
    };
    triangle(a, b, c, colors.channel_text, window);
}
