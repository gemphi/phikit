//! Painting one curve's handles — `draw_fcurve_handles`: each segment a
//! one-pixel line in its type's `TH_HANDLE_*` colour at the curve's
//! display alpha; then `draw_fcurve_handle_vertices`: a ring on each
//! handle's end, `1.4 × TH_HANDLE_VERTEX_SIZE` across with a 1.5 px
//! outline in `TH_HANDLE_VERTEX` or `_SELECT`, its fill all but invisible.

use super::super::super::set::rect;
use super::super::super::BTimeView;
use super::super::BDrawnCurve;
use super::BCurveHandles;
use crate::v2::theme::{BAnimColors, BColor};
use crate::v2::view::draw::color::to_hsla;
use gpui::{BorderStyle, Bounds, Corners, Edges, PathBuilder, Pixels, Window, point, px, quad};

/// `GPU_line_width(1.0)` for the handle lines.
const LINE_WIDTH: f32 = 1.0;
/// `1.4 × TH_HANDLE_VERTEX_SIZE` (5) and `outlineWidth` 1.5.
const RING_SIZE: f32 = 7.0;
const RING_OUTLINE: f32 = 1.5;
/// `immUniformColor3fvAlpha(hcolor, 0.01f)`: the ring's fill.
const RING_FILL_ALPHA: f32 = 0.01;

/// The `TH_HANDLE_*` colour of a handle type, selected or not.
pub fn handle_color(kind: u8, selected: bool, colors: &BAnimColors) -> BColor {
    let table = if selected { &colors.handle_select } else { &colors.handle };
    table[(kind as usize).min(table.len() - 1)]
}

/// `draw_fcurve_handles` then `draw_fcurve_handle_vertices` for `drawn`,
/// when `draw_fcurve_handles_check` allows them.
pub fn paint_handles(view: &BTimeView, drawn: &BDrawnCurve, colors: &BAnimColors, at: Bounds<Pixels>, window: &mut Window) {
    let width = f32::from(at.size.width);
    let height = f32::from(at.size.height);
    let handles = BCurveHandles::of(&drawn.curve, view.first, view.last, false);
    let place = |[frame, value]: [f32; 2]| point(at.origin.x + px(view.x_of(frame, width)), at.origin.y + px(view.y_of(value, height)));
    for segment in &handles.segments {
        let mut path = PathBuilder::stroke(px(LINE_WIDTH));
        path.move_to(place(segment.handle));
        path.line_to(place(segment.key));
        let mut color = to_hsla(handle_color(segment.kind, segment.selected, colors));
        color.a = drawn.alpha;
        if let Ok(path) = path.build() {
            window.paint_path(path, color);
        }
    }
    let half = RING_SIZE / 2.0;
    for ring in &handles.rings {
        let (x, y) = (view.x_of(ring.at[0], width), view.y_of(ring.at[1], height));
        if x < -half || x > width + half {
            continue;
        }
        let outline = to_hsla(if ring.selected { colors.handle_vertex_select } else { colors.handle_vertex });
        let mut fill = outline;
        fill.a = RING_FILL_ALPHA;
        window.paint_quad(quad(rect(x - half, y - half, x + half, y + half, at), Corners::all(px(half)), fill, Edges::all(px(RING_OUTLINE)), outline, BorderStyle::Solid));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phiphx::blend::{HANDLE_AUTO, HANDLE_AUTO_ANIM, HANDLE_FREE};

    #[test]
    fn handle_colours_index_blenders_theme_slots() {
        let state = crate::v2::state::BState::new();
        let colors = crate::v2::theme::resolve_ui_style(crate::v2::theme::style::ui_style_defaults(crate::v2::theme::BPaletteName::Sphere), &state).anim();
        assert_eq!(handle_color(HANDLE_FREE, true, &colors), colors.handle_select[0]);
        assert_eq!(handle_color(HANDLE_AUTO, false, &colors), colors.handle[1]);
        assert_eq!(handle_color(HANDLE_AUTO_ANIM, true, &colors), colors.handle_select[4]);
        assert_eq!(handle_color(9, false, &colors), colors.handle[4], "an unknown type is clamped to the last slot");
    }
}
