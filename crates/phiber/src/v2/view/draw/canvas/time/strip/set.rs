//! What the NLA editor paints in its rows — `draw_nla_main_data`: for an
//! action line (`ANIMTYPE_NLAACTION`) the darkened frames outside the
//! action's range (`ANIM_draw_action_framerange`), the action's band across
//! the view (`act_extendmode = HOLD`), the darker band over its keys and the
//! keys themselves as frames (`nla_action_draw_keyframes`); for a track
//! (`ANIMTYPE_NLATRACK`) each strip's hold backdrops, box, outline, repeat
//! lines and name (`nla_draw_strip`, `nla_draw_strip_text`).

use super::super::row::{BRowKind, BTimeRow, NLA_ROW_HEIGHT, ROW_SKIP, set::diamond};
use super::super::set::{hsla, rect, shape, text_at};
use super::super::BTimeView;
use super::{BTrackStrips, HOLD_AFTER_ALPHA, HOLD_BEFORE_ALPHA, MUTED_ALPHA, REPEAT_INSET, TEXT_MARGIN};
use crate::phiphx::blend::{STRIP_ACTIVE, STRIP_MUTED, STRIP_SELECT, TRACK_MUTED};
use crate::v2::theme::{BAnimColors, BColor};
use gpui::{App, BorderStyle, Bounds, Corners, Edges, Pixels, Window, fill, px, quad};

/// `nla_action_draw_keyframes`: the key band is the action colour at up to
/// 2.5× its alpha, capped at 0.7, and its keys are 6 px frames.
const KEY_BAND_GAIN: f32 = 2.5;
const KEY_BAND_ALPHA_CAP: f32 = 0.7;
const KEY_SIZE: f32 = 6.0;
/// The key band sits 2 px inside the row.
const KEY_BAND_INSET: f32 = 2.0;
/// A strip's outline is a pixel.
const OUTLINE: f32 = 1.0;

fn with_alpha(color: BColor, alpha: f32) -> BColor {
    (color.0, color.1, color.2, alpha)
}

/// `nla_action_get_color`: tweaking green, the active action's colour, or
/// the greyer one for no action.
pub fn action_color(has_action: bool, tweaking: bool, colors: &BAnimColors) -> BColor {
    if tweaking {
        colors.tweak
    } else if has_action {
        colors.action_line
    } else {
        colors.action_none
    }
}

impl BTimeView {
    /// Every NLA row of `view`, in the window `at`.
    pub fn paint_nla_rows(&self, colors: &BAnimColors, at: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
        let width = f32::from(at.size.width);
        let x = |frame: f32| self.x_of(frame, width);
        for (index, row) in self.rows.iter().enumerate() {
            let (top, bottom) = (BTimeRow::top(index, NLA_ROW_HEIGHT), BTimeRow::top(index, NLA_ROW_HEIGHT) + NLA_ROW_HEIGHT);
            match &row.kind {
                BRowKind::Action { name, tweaking } => paint_action(row, action_color(name.is_some(), *tweaking, colors), top, bottom, colors, at, &x, window),
                BRowKind::Track(track) => {
                    let laid = BTrackStrips::of(track, self.first, self.last);
                    let muted_track = track.flag & TRACK_MUTED != 0;
                    for boxed in &laid.boxes {
                        paint_strip(boxed, muted_track, top, bottom, colors, at, &x, window, cx);
                    }
                }
                BRowKind::Summary { .. } | BRowKind::Object { .. } | BRowKind::Driver { .. } | BRowKind::Curve { .. } => {}
            }
        }
    }
}

/// `ANIMTYPE_NLAACTION`: the action line.
#[allow(clippy::too_many_arguments)]
fn paint_action(row: &BTimeRow, color: BColor, top: f32, bottom: f32, colors: &BAnimColors, at: Bounds<Pixels>, x: &dyn Fn(f32) -> f32, window: &mut Window) {
    let width = f32::from(at.size.width);
    // `ANIM_draw_action_framerange`: the frames outside the action's range.
    if let (Some(first), Some(last)) = (row.keys.first(), row.keys.last()) {
        window.paint_quad(fill(rect(0.0, top, x(*first), bottom, at), hsla(colors.out_of_range)));
        window.paint_quad(fill(rect(x(*last), top, width, bottom, at), hsla(colors.out_of_range)));
    }
    // `act_extendmode == HOLD`: the action's band across the whole view,
    // a skip in from the row's top.
    window.paint_quad(fill(rect(0.0, top + ROW_SKIP, width, bottom + ROW_SKIP - OUTLINE, at), hsla(color)));
    let (Some(first), Some(last)) = (row.keys.first(), row.keys.last()) else { return };
    let band = with_alpha(color, (color.3 * KEY_BAND_GAIN).min(KEY_BAND_ALPHA_CAP));
    window.paint_quad(fill(rect(x(*first), top + ROW_SKIP + KEY_BAND_INSET, x(*last), bottom + ROW_SKIP - OUTLINE - KEY_BAND_INSET, at), hsla(band)));
    let center = (top + bottom) / 2.0 + ROW_SKIP;
    for &frame in &row.keys {
        diamond(x(frame).round() + 0.5, center, KEY_SIZE, band, colors, at, window);
    }
}

/// `nla_draw_strip` and `nla_draw_strip_text` for one strip.
#[allow(clippy::too_many_arguments)]
fn paint_strip(boxed: &super::BStripBox, muted_track: bool, top: f32, bottom: f32, colors: &BAnimColors, at: Bounds<Pixels>, x: &dyn Fn(f32) -> f32, window: &mut Window, cx: &mut App) {
    let strip = boxed.strip;
    let selected = strip.flag & STRIP_SELECT != 0;
    let active = strip.flag & STRIP_ACTIVE != 0;
    let muted = muted_track || strip.flag & STRIP_MUTED != 0;
    let color = if selected { colors.strip_select } else { colors.strip };
    if let Some(from) = boxed.held_before {
        window.paint_quad(fill(rect(x(from), top, x(strip.start), bottom, at), hsla(with_alpha(color, HOLD_BEFORE_ALPHA))));
    }
    if let Some(to) = boxed.held_after {
        window.paint_quad(fill(rect(x(strip.end), top, x(to), bottom, at), hsla(with_alpha(color, HOLD_AFTER_ALPHA))));
    }
    let inside = if muted { with_alpha(color, MUTED_ALPHA) } else { color };
    let outline = if active { colors.strip_outline_active } else { colors.strip_outline };
    let body = rect(x(strip.start), top, x(strip.end), bottom, at);
    window.paint_quad(quad(body, Corners::all(Pixels::ZERO), hsla(inside), Edges::all(px(OUTLINE)), hsla(outline), BorderStyle::Solid));
    for frame in &boxed.repeats {
        window.paint_quad(fill(rect(x(*frame).floor(), top + REPEAT_INSET, x(*frame).floor() + OUTLINE, bottom - REPEAT_INSET, at), hsla(outline)));
    }
    // The name, black on an active strip and white otherwise, when there is
    // room for it between the margins.
    let (left, right) = (x(strip.start) + TEXT_MARGIN, x(strip.end) - TEXT_MARGIN);
    if left < right {
        let text = if active { colors.strip_text_active } else { colors.strip_text };
        let line = shape(&strip.name, text, window);
        if f32::from(line.width()) <= right - left {
            text_at(&line, left, (top + bottom) / 2.0, at, window, cx);
        }
    }
}
