//! What the animation window paints, in the order Blender paints it
//! (`action_main_region_draw` / `graph_main_region_draw`, then their
//! overlays): the grid, the darkened frames outside the scene range, the
//! preview range, the key columns — or, in the Graph editor, the curves
//! and the cursor — the number strip, the markers in their band along the
//! foot, and the playhead on top. The Drivers editor's x axis is not time:
//! no range, no preview, no markers, no playhead.
//!
//! Colours are the theme's (`BAnimColors`, `theme/style/anim.rs`): each
//! stands in for the Blender slot it names, so the geometry is Blender's
//! and the palette is phiber's.

use super::BTimeView;
use crate::v2::theme::{BAnimColors, BColor};
use crate::v2::view::draw::color::to_hsla;
use gpui::{App, BorderStyle, Bounds, Corners, Edges, Hsla, PathBuilder, Pixels, Point, SharedString, TextRun, Window, fill, point, px, quad, size};

pub(super) fn hsla(color: BColor) -> Hsla {
    to_hsla(color)
}

pub(super) fn rect(x0: f32, y0: f32, x1: f32, y1: f32, at: Bounds<Pixels>) -> Bounds<Pixels> {
    let (x0, x1) = (x0.min(x1), x0.max(x1));
    let (y0, y1) = (y0.min(y1), y0.max(y1));
    Bounds::new(point(at.origin.x + px(x0), at.origin.y + px(y0)), size(px(x1 - x0), px(y1 - y0)))
}

pub(super) fn line_x(x: f32, y0: f32, y1: f32, width: f32, color: BColor, at: Bounds<Pixels>, window: &mut Window) {
    let x = x.floor();
    window.paint_quad(fill(rect(x, y0, x + width, y1, at), hsla(color)));
}

pub(super) fn triangle(a: Point<Pixels>, b: Point<Pixels>, c: Point<Pixels>, color: BColor, window: &mut Window) {
    let mut path = PathBuilder::fill();
    path.move_to(a);
    path.line_to(b);
    path.line_to(c);
    path.close();
    if let Ok(path) = path.build() {
        window.paint_path(path, hsla(color));
    }
}

/// `UI_FSTYLE_WIDGET` points at scale 1.
pub(super) const FONT: f32 = 11.0;
/// `get_label_width`: the padding past the widest label.
const LABEL_PADDING: f32 = 6.0;

/// `frame_to_string`: whole at a step of one or more, then one, two or
/// three decimals as the step shrinks.
pub(super) fn frame_label(frame: f32, step: f32) -> String {
    if step >= 1.0 {
        format!("{}", frame as i32)
    } else if step >= 0.5 {
        format!("{frame:.1}")
    } else if step >= 0.01 {
        format!("{frame:.2}")
    } else {
        format!("{frame:.3}")
    }
}
/// `draw_playhead_box`: the corner radius of the frame-number box.
const BOX_RADIUS: f32 = 4.0;

pub(super) fn shape(text: &str, color: BColor, window: &Window) -> gpui::ShapedLine {
    let run = TextRun {
        len: text.len(),
        font: window.text_style().font(),
        color: hsla(color),
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    window.text_system().shape_line(SharedString::from(text.to_string()), px(FONT), &[run], None)
}

pub(super) fn text_at(line: &gpui::ShapedLine, x: f32, center_y: f32, at: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
    let line_height = px(FONT * 1.3);
    let origin = point(at.origin.x + px(x), at.origin.y + px(center_y) - line_height / 2.0);
    let _ = line.paint(origin, line_height, gpui::TextAlign::Left, None, window, cx);
}

impl BTimeView {
    pub fn paint(&self, colors: &BAnimColors, at: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
        let width = f32::from(at.size.width);
        let x = |frame: f32| self.x_of(frame, width);
        // `get_min_line_distance_x`: the wider of the edge labels — measured
        // at step 0, so with three decimals — sets the floor on the line
        // distance, so numbers never overlap.
        let label_width = |value: f32| f32::from(shape(&frame_label(value.abs(), 0.0), colors.scrub_text, window).width());
        let min_distance = super::MIN_GRID.max(label_width(self.first).max(label_width(self.last)) + LABEL_PADDING);
        let step = self.grid_step(width, min_distance);
        self.paint_frame_lines(colors, step, at, window);
        self.paint_value_lines(colors, at, window, cx);
        if !self.drivers {
            self.paint_ranges(colors, at, window);
        }
        // The Dope Sheet lays its keys out in rows; the NLA editor lays its
        // strips out in taller ones; the Graph editor draws the curves
        // themselves over its value axis.
        match self.values {
            None if self.row_height > super::row::ROW_HEIGHT => self.paint_nla_rows(colors, at, window, cx),
            None => super::row::set::paint_rows(&self.rows, colors, at, &x, &|row, frame| self.picked(row, frame), self.scrub_height, window),
            Some(_) => self.paint_curves(&self.curves, colors, at, window),
        }
        self.paint_cursor(colors, at, window);
        self.paint_scrub(colors, step, at, window, cx);
        if !self.drivers {
            self.paint_markers(colors, at, window, cx);
            self.paint_playhead(colors, at, window, cx);
        }
    }
}

impl BTimeView {
    /// `ANIM_draw_framerange` and `ANIM_draw_previewrange`: the frames outside
    /// the scene range darkened behind their edge lines, and the frames
    /// outside the preview range over that.
    fn paint_ranges(&self, colors: &BAnimColors, at: Bounds<Pixels>, window: &mut Window) {
        let width = f32::from(at.size.width);
        let height = f32::from(at.size.height);
        let x = |frame: f32| self.x_of(frame, width);
        let (start, end) = (self.start as f32, self.end as f32);
        window.paint_quad(fill(rect(0.0, self.scrub_height, x(start), height, at), hsla(colors.out_of_range)));
        window.paint_quad(fill(rect(x(end), self.scrub_height, width, height, at), hsla(colors.out_of_range)));
        line_x(x(start), self.scrub_height, height, 1.0, colors.range_line, at, window);
        line_x(x(end), self.scrub_height, height, 1.0, colors.range_line, at, window);
        if let Some((first, last)) = self.preview {
            window.paint_quad(fill(rect(0.0, self.scrub_height, x(first as f32), height, at), hsla(colors.preview)));
            window.paint_quad(fill(rect(x(last as f32), self.scrub_height, width, height, at), hsla(colors.preview)));
        }
    }

    /// `ED_time_scrub_draw`: the strip and its numbers, one per major line,
    /// spelt as `frame_to_string` spells them for the step.
    fn paint_scrub(&self, colors: &BAnimColors, step: f32, at: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
        let width = f32::from(at.size.width);
        window.paint_quad(fill(rect(0.0, 0.0, width, self.scrub_height, at), hsla(colors.scrub_back)));
        for frame in self.lines(step) {
            let line = shape(&frame_label(frame, step), colors.scrub_text, window);
            let x = self.x_of(frame, width) - (f32::from(line.width()) / 2.0).trunc();
            text_at(&line, x, self.scrub_height / 2.0, at, window, cx);
        }
    }

    /// `ED_time_scrub_draw_current_frame`: the stalk down the window, the
    /// rounded box with the frame number, and the tip under it.
    fn paint_playhead(&self, colors: &BAnimColors, at: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
        let width = f32::from(at.size.width);
        let height = f32::from(at.size.height);
        let x = self.x_of(self.current, width);
        let label = format!("{}", self.current.round() as i64);
        let line = shape(&label, colors.playhead_text, window);
        let text_width = f32::from(line.width());
        let box_width = (text_width + 8.0).max(24.0);
        let (margin, shadow) = (2.0, 1.0);
        // Stalk: `pixelsize` either side of the frame, a shadow either side.
        let stalk = rect(x.floor() - 1.0 - shadow, self.scrub_height, x.floor() + 2.0 + shadow, height, at);
        window.paint_quad(quad(stalk, Corners::all(Pixels::ZERO), hsla(colors.playhead), Edges::all(px(shadow)), hsla(colors.playhead_shadow), BorderStyle::Solid));
        // Tip: a triangle under the box, its shadow first.
        let tip_top = self.scrub_height - margin;
        let tip = |half: f32, drop: f32, dy: f32| {
            (
                point(at.origin.x + px(x - half), at.origin.y + px(tip_top + dy)),
                point(at.origin.x + px(x + half + 1.0), at.origin.y + px(tip_top + dy)),
                point(at.origin.x + px(x + 0.5), at.origin.y + px(tip_top + dy + drop)),
            )
        };
        let (a, b, c) = tip(6.0 + shadow + 0.4, 6.0 + shadow + 0.4, 0.0);
        triangle(a, b, c, colors.playhead_shadow, window);
        // Box: `draw_roundbox_4fv_ex` with a one-pixel shadow outline.
        let boxed = rect(x - box_width / 2.0, margin - shadow, x + box_width / 2.0 + 1.0, self.scrub_height - margin + shadow, at);
        window.paint_quad(quad(boxed, Corners::all(px(BOX_RADIUS)), hsla(colors.playhead), Edges::all(px(shadow)), hsla(colors.playhead_shadow), BorderStyle::Solid));
        let (a, b, c) = tip(6.0, 6.0, 0.0);
        triangle(a, b, c, colors.playhead, window);
        text_at(&line, x - text_width / 2.0, self.scrub_height / 2.0, at, window, cx);
    }
}



#[cfg(test)]
mod tests {
    use super::frame_label;

    #[test]
    fn labels_carry_the_decimals_their_step_needs() {
        assert_eq!(frame_label(24.0, 12.0), "24");
        assert_eq!(frame_label(0.5, 0.5), "0.5");
        assert_eq!(frame_label(-1.25, 0.25), "-1.25");
        assert_eq!(frame_label(0.005, 0.005), "0.005");
        assert_eq!(frame_label(2.0, 0.0), "2.000", "`get_label_width` measures at step 0");
    }
}
