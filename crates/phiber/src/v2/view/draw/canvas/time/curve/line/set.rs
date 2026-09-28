//! Stroking one curve's line — `draw_fcurve`'s `GPU_PRIM_LINE_STRIP` over
//! the points `BCurveLine` lists for the view, 2.5 px wide for the active curve and
//! 1 px for the rest, in the colour `curve/set.rs` gives the curve.

use super::super::super::BTimeView;
use super::super::BDrawnCurve;
use super::BCurveLine;
use crate::phiphx::blend::FLAG_PROTECTED;
use crate::v2::theme::BAnimColors;
use gpui::{Bounds, PathBuilder, Pixels, Window, point, px};

/// `draw_fcurve`: the active curve is 2.5 px wide, the rest 1 px.
const LINE_WIDTH: f32 = 1.0;
const ACTIVE_LINE_WIDTH: f32 = 2.5;

/// The stroke a curve earns: thick when active and editable.
pub fn stroke(drawn: &BDrawnCurve) -> f32 {
    if drawn.active && drawn.curve.flag & FLAG_PROTECTED == 0 { ACTIVE_LINE_WIDTH } else { LINE_WIDTH }
}

/// The line of `drawn` across the window `at`.
pub fn paint(view: &BTimeView, drawn: &BDrawnCurve, colors: &BAnimColors, at: Bounds<Pixels>, window: &mut Window) {
    let width = f32::from(at.size.width);
    let height = f32::from(at.size.height);
    let line = BCurveLine::for_view(&drawn.curve, view.first, view.last, [view.scale(width), view.value_scale(height)]);
    if line.points.len() < 2 {
        return;
    }
    let mut path = PathBuilder::stroke(px(stroke(drawn)));
    for (index, [frame, value]) in line.points.iter().enumerate() {
        let here = point(at.origin.x + px(view.x_of(*frame, width)), at.origin.y + px(view.y_of(*value, height)));
        if index == 0 {
            path.move_to(here);
        } else {
            path.line_to(here);
        }
    }
    if let Ok(path) = path.build() {
        window.paint_path(path, super::super::set::line_color(drawn, colors));
    }
}
