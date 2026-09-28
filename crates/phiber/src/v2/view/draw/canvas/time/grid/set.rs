//! The grid the animation window paints — `view2d_draw_lines_x_frames` for
//! the frames, and for the Graph editor alone `view2d_draw_lines_y` for the
//! values and `view2d_draw_scale_y` for their numbers up the left edge —
//! in the theme's colours (`BAnimColors`).

use super::super::set::{hsla, line_x, rect, shape, text_at, FONT};
use super::super::{BTimeView, MIN_GRID, SCRUB_HEIGHT};
use crate::v2::theme::{BAnimColors, BColor};
use gpui::{App, Bounds, Pixels, Window, fill};

/// `graph_main_region_draw_overlay`: the numbers keep 15 px off the foot of
/// the window and sit 8 px in from its left edge.
const SCALE_FOOT: f32 = 15.0;
const SCALE_INSET: f32 = 8.0;
/// `graph_main_region_draw`: the value grid steps in tens.
pub const VALUE_BASE: i32 = 10;

fn line_y(y: f32, x0: f32, x1: f32, color: BColor, at: Bounds<Pixels>, window: &mut Window) {
    let y = y.floor();
    window.paint_quad(fill(rect(x0, y, x1, y + 1.0, at), hsla(color)));
}

/// The frame lines under the scrub strip, `step` frames apart: the minors
/// first, where they are far enough apart, then the majors over them.
pub fn paint_frame_lines(view: &BTimeView, colors: &BAnimColors, step: f32, at: Bounds<Pixels>, window: &mut Window) {
    let width = f32::from(at.size.width);
    let height = f32::from(at.size.height);
    let (major, minor) = (colors.grid, colors.grid_minor);
    if let Some(minor_step) = BTimeView::minor_step(step, view.scale(width), MIN_GRID) {
        for frame in view.lines(minor_step) {
            line_x(view.x_of(frame, width), SCRUB_HEIGHT, height, 1.0, minor, at, window);
        }
    }
    for frame in view.lines(step) {
        line_x(view.x_of(frame, width), SCRUB_HEIGHT, height, 1.0, major, at, window);
    }
}

/// The value lines across the window, and their numbers. Nothing without
/// a value axis.
pub fn paint_value_lines(view: &BTimeView, colors: &BAnimColors, at: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
    if view.values.is_none() {
        return;
    }
    let width = f32::from(at.size.width);
    let height = f32::from(at.size.height);
    let step = view.value_step(height, VALUE_BASE, MIN_GRID);
    let (major, minor) = (colors.grid, colors.grid_minor);
    if let Some(minor_step) = BTimeView::minor_step(step, view.value_scale(height), MIN_GRID) {
        for value in view.value_lines(minor_step) {
            line_y(view.y_of(value, height), 0.0, width, minor, at, window);
        }
    }
    for value in view.value_lines(step) {
        line_y(view.y_of(value, height), 0.0, width, major, at, window);
    }
    // `draw_vertical_scale_indicators`: a number on each major line, kept
    // whole between the strip and the foot.
    let half_glyph = FONT / 2.0 - 1.0;
    for value in view.value_lines(step) {
        let y = view.y_of(value, height);
        if y - half_glyph >= SCRUB_HEIGHT && y + half_glyph <= height - SCALE_FOOT {
            let line = shape(&value_label(value, step), colors.scale_text, window);
            text_at(&line, SCALE_INSET, y, at, window, cx);
        }
    }
}

/// `frame_to_string`: as many decimals as the step needs.
pub fn value_label(value: f32, step: f32) -> String {
    if step >= 1.0 {
        format!("{}", value.round() as i64)
    } else if step >= 0.5 {
        format!("{value:.1}")
    } else if step >= 0.01 {
        format!("{value:.2}")
    } else {
        format!("{value:.3}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_carry_the_decimals_their_step_needs() {
        assert_eq!(value_label(10.0, 10.0), "10");
        assert_eq!(value_label(-0.5, 0.5), "-0.5");
        assert_eq!(value_label(0.25, 0.05), "0.25");
        assert_eq!(value_label(0.005, 0.005), "0.005");
    }
}
