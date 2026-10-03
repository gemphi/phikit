//! What a channel row paints — `draw_backdrops` and `draw_keyframes` in
//! `action_draw.cc`, and the channel region's names
//! (`action_channel_region_draw`) — in the theme's colours (`BAnimColors`).

use super::super::set::{hsla, rect, shape, text_at};
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
pub fn paint_rows(rows: &[BTimeRow], colors: &BAnimColors, at: Bounds<Pixels>, x_of: &dyn Fn(f32) -> f32, picked: &dyn Fn(usize, f32) -> bool, window: &mut Window) {
    let width = f32::from(at.size.width);
    for (index, row) in rows.iter().enumerate() {
        let (top, center) = (BTimeRow::top(index, ROW_HEIGHT), BTimeRow::center(index, ROW_HEIGHT));
        let back = match &row.kind {
            BRowKind::Summary { expanded: true } => Some(colors.summary_back),
            BRowKind::Summary { expanded: false } => None,
            BRowKind::Object(_) => Some(colors.object_back),
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
pub fn paint_channels(rows: &[BTimeRow], row_height: f32, colors: &BAnimColors, at: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
    let height = f32::from(at.size.height);
    window.paint_quad(fill(rect(0.0, 0.0, CHANNEL_WIDTH, height, at), hsla(colors.channel_back)));
    let indent = super::super::WIDGET_UNIT * 0.5;
    for (index, row) in rows.iter().enumerate() {
        let top = BTimeRow::top(index, row_height);
        let back = match &row.kind {
            BRowKind::Object(_) if row_height > ROW_HEIGHT => Some(colors.object_back),
            BRowKind::Action { name, tweaking } => Some(super::super::strip::set::action_color(name.is_some(), *tweaking, colors)),
            BRowKind::Track(_) => Some(colors.track_back),
            BRowKind::Driver { .. } => Some(colors.curve_back),
            BRowKind::Summary { .. } | BRowKind::Object(_) => None,
        };
        if let Some(back) = back {
            window.paint_quad(fill(rect(0.0, top, CHANNEL_WIDTH, top + row_height, at), hsla(back)));
        }
        // `ANIM_channel_draw` in the Graph editor: an F-curve's colour
        // swatch, then its visibility toggle's width, before the name.
        if let BRowKind::Driver { color, .. } = &row.kind {
            let swatch = gpui::Rgba { r: color[0], g: color[1], b: color[2], a: 1.0 };
            window.paint_quad(fill(rect(CURVE_OFFSET, top, CURVE_OFFSET + ICON_WIDTH, top + row_height, at), swatch));
        }
        let ink = match &row.kind {
            BRowKind::Track(track) if track.flag & crate::phiphx::blend::TRACK_SELECTED != 0 => colors.strip_select,
            _ => colors.channel_text,
        };
        let line = shape(row.name(), ink, window);
        let x = match row.kind {
            BRowKind::Summary { .. } | BRowKind::Object(_) if row_height > ROW_HEIGHT => indent,
            BRowKind::Summary { .. } => indent,
            BRowKind::Object(_) => indent * 2.0,
            BRowKind::Action { .. } | BRowKind::Track(_) => indent * 2.0,
            BRowKind::Driver { .. } => CURVE_OFFSET + ICON_WIDTH * 2.0 + NAME_PAD,
        };
        text_at(&line, x, BTimeRow::center(index, row_height), at, window, cx);
    }
}
