//! What the Graph editor paints for its cursor — `graph_main_region_draw`
//! under `SIPO_NODRAWCURSOR` off: a two-pixel line across the view at
//! `cursorVal`, and in the Drivers editor a second one up it at
//! `cursorTime`, in `TH_CFRAME` shaded and faded so neither reads as the
//! playhead.

use super::super::set::{hsla, line_x, rect};
use super::super::BTimeView;
use crate::v2::theme::BAnimColors;
use gpui::{Bounds, Pixels, Window, fill};

/// `GPU_line_width(2.0)`.
const CURSOR_WIDTH: f32 = 2.0;
/// `SpaceGraph.cursorTime / cursorVal` as `graph_create` leaves them.
pub const CURSOR: (f32, f32) = (0.0, 0.0);

/// The value line, and the input line in the Drivers editor. Nothing
/// without a value axis: the Dope Sheet has no cursor.
pub fn paint_cursor(view: &BTimeView, colors: &BAnimColors, at: Bounds<Pixels>, window: &mut Window) {
    if view.values.is_none() {
        return;
    }
    let width = f32::from(at.size.width);
    let height = f32::from(at.size.height);
    let (time, value) = CURSOR;
    let y = view.y_of(value, height).floor();
    window.paint_quad(fill(rect(0.0, y - CURSOR_WIDTH / 2.0, width, y + CURSOR_WIDTH / 2.0, at), hsla(colors.cursor_value)));
    if view.drivers {
        line_x(view.x_of(time, width) - CURSOR_WIDTH / 2.0, super::super::SCRUB_HEIGHT, height, CURSOR_WIDTH, colors.cursor_time, at, window);
    }
}
