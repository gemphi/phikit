//! The animation window's `View2D` — the frames (or, in the Drivers
//! editor, the driver values) across the window and the values up it, and
//! the maps between view units and pixels (`view2d_view_to_region_x` /
//! `_y`, `view2d_region_to_view_x`). Structure here; reading the view out
//! of the state is `set.rs`.

pub mod cur;
pub mod op;
pub mod set;

use super::row::BTimeRow;
use super::curve::BDrawnCurve;
use gpui::{Bounds, Pixels, point, px, size};

/// `graph_create`: the value axis opens at `sfra − 10 .. 10`.
pub const GRAPH_VALUE_TOP: f32 = 10.0;
/// `ED_drivers_editor_init`: the Drivers editor opens on `−2 .. 2` both ways.
pub const DRIVER_VIEW: (f32, f32) = (-2.0, 2.0);
/// `graph_main_region_draw`'s `driver_step`: the Drivers editor's axes step
/// in tens, whatever the frame rate.
pub const DRIVER_STEP: i32 = 10;

/// The window's `View2D`: the units on screen and the scene they belong to.
#[derive(Clone, Debug, PartialEq)]
pub struct BTimeView {
    pub start: i32,
    pub end: i32,
    pub current: f32,
    pub fps: f32,
    /// `Scene.r.psfra..pefra` while `use_preview_range` is on.
    pub preview: Option<(i32, i32)>,
    /// `v2d.cur.xmin / xmax`.
    pub first: f32,
    pub last: f32,
    /// `v2d.cur.ymin / ymax` — the Graph editor's value axis. The Dope
    /// Sheet has none: its rows are laid out in pixels.
    pub values: Option<(f32, f32)>,
    /// `SIPO_MODE_DRIVERS`: the x axis is the driver's input value, not
    /// time — no frame range, no playhead, a base-ten grid, and the cursor
    /// crosses at `cursorTime / cursorVal`.
    pub drivers: bool,
    /// The NLA editor: a press in its rows picks strips and tracks.
    pub nla: bool,
    /// The channel rows, summary first where the editor lists one, and how
    /// tall each is (`NLATRACK_HEIGHT` in the NLA editor).
    pub rows: Vec<BTimeRow>,
    pub row_height: f32,
    /// The frame-number strip's height, in pixels.
    ///
    /// `UI_TIME_SCRUB_MARGIN_Y` is where the number comes from, but it is the
    /// THEME's header height that it has to equal: the scrub strip reads as a
    /// header for the window below it, and a strip 23px tall against 36px
    /// headers either side of it is the one thing in an animation editor that
    /// does not line up with anything.
    ///
    /// It is a field and not the `SCRUB_HEIGHT` const it used to be because
    /// every painter and hit-test here is a method of this view, so the view is
    /// the one place the theme can reach them all. A const could not be told
    /// what the theme says — which is why changing the theme's own
    /// `scrubbing_height` had no effect on the painted band at all.
    pub scrub_height: f32,
    /// The F-curves the Graph editor draws.
    pub curves: Vec<BDrawnCurve>,
    /// The channel region's width: the Dope Sheet and Graph editor show it,
    /// the Timeline hides it (`RGN_FLAG_HIDDEN` in `action_create`).
    pub channel_width: f32,
    /// `Scene.markers` as `(frame, name, selected)`, in frame order; the
    /// Drivers editor, whose x axis is not time, carries none.
    pub markers: Vec<(f32, String, bool)>,
    /// A box being swept over the marker band: its two frames.
    pub marker_box: Option<(f32, f32)>,
    /// The Dope Sheet's picked key columns, the box being swept over its
    /// keys (two frames, two rows), and whether a key move is under way
    /// (`scene::action::dope`).
    pub key_picks: crate::v2::scene::action::dope::BKeySelection,
    pub key_box: Option<(f32, f32, usize, usize)>,
    pub key_grab: bool,
}

impl BTimeView {
    /// The view `action_create` opens on a scene range: ten frames either
    /// side, the current frame on the first, no rows and no value axis.
    pub fn of_range(start: i32, end: i32, fps: f32) -> Self {
        Self {
            start,
            end,
            current: start as f32,
            fps,
            preview: None,
            first: start as f32 - super::VIEW_MARGIN,
            last: end as f32 + super::VIEW_MARGIN,
            values: None,
            drivers: false,
            nla: false,
            rows: Vec::new(),
            row_height: super::row::ROW_HEIGHT,
            scrub_height: super::SCRUB_HEIGHT,
            curves: Vec::new(),
            channel_width: 0.0,
            markers: Vec::new(),
            marker_box: None,
            key_picks: Default::default(),
            key_box: None,
            key_grab: false,
        }
    }

    /// The x axis's grid base: the frame rate, or `driver_step` where x is
    /// a value (`graph_main_region_draw`).
    pub fn base(&self) -> i32 {
        if self.drivers { DRIVER_STEP } else { self.fps.round() as i32 }
    }

    /// The window: the canvas less the channel region on its left.
    pub fn window(&self, at: Bounds<Pixels>) -> Bounds<Pixels> {
        let shift = px(self.channel_width.min(f32::from(at.size.width)));
        Bounds::new(point(at.origin.x + shift, at.origin.y), size(at.size.width - shift, at.size.height))
    }

    /// Whether row `row`'s key at `frame` is picked: an object's own
    /// column, or for the summary any listed object's.
    pub fn picked(&self, row: usize, frame: f32) -> bool {
        use super::row::BRowKind;
        match self.rows.get(row).map(|r| &r.kind) {
            Some(BRowKind::Object { name, .. } | BRowKind::Slot { object: name, .. }) => self.key_picks.has(name, frame),
            // An unfolded object's F-curve rows answer with their object's
            // picks: a key column is picked per object, and a curve under it
            // shows the same column (`row::curves`).
            Some(BRowKind::Curve { .. } | BRowKind::Group { .. }) => self
                .rows
                .get(..row)
                .and_then(|above| above.iter().rev().find_map(|r| match &r.kind {
                    BRowKind::Object { name, .. } | BRowKind::Slot { object: name, .. } => Some(self.key_picks.has(name, frame)),
                    _ => None,
                }))
                .unwrap_or(false),
            Some(BRowKind::Summary { .. }) => self.key_picks.0.iter().any(|p| (p.frame - frame).abs() < 5e-4),
            _ => false,
        }
    }

    /// `view2d_view_to_region_x`.
    pub fn x_of(&self, frame: f32, width: f32) -> f32 {
        (frame - self.first) / (self.last - self.first) * width
    }

    /// `view2d_region_to_view_x`.
    pub fn frame_at(&self, x: f32, width: f32) -> f32 {
        self.first + x / width.max(1.0) * (self.last - self.first)
    }

    /// Pixels per frame.
    pub fn scale(&self, width: f32) -> f32 {
        width / (self.last - self.first)
    }

    /// `view2d_view_to_region_y`: values run up the window, `cur.ymin` at
    /// its foot. Without a value axis every value is the foot.
    pub fn y_of(&self, value: f32, height: f32) -> f32 {
        let Some((low, high)) = self.values else { return height };
        height - (value - low) / (high - low).max(f32::EPSILON) * height
    }

    /// Pixels per value unit.
    pub fn value_scale(&self, height: f32) -> f32 {
        let (low, high) = self.values.unwrap_or((0.0, 1.0));
        height / (high - low).max(f32::EPSILON)
    }
}

#[cfg(test)]
mod tests {
    use super::super::row;
    use super::*;
    use gpui::Pixels;

    #[test]
    fn frames_and_pixels_round_trip_and_the_sheet_keeps_a_channel_region() {
        let v = BTimeView::of_range(1, 140, 24.0);
        let width = 1500.0;
        let x = v.x_of(70.0, width);
        assert!((v.frame_at(x, width) - 70.0).abs() < 1e-3);
        let sheet = BTimeView { channel_width: row::CHANNEL_WIDTH, ..v };
        let at = |x: f32, w: f32| Bounds::new(point(px(x), Pixels::ZERO), size(px(w), px(width)));
        assert_eq!(sheet.window(at(10.0, 1000.0)), at(10.0 + row::CHANNEL_WIDTH, 1000.0 - row::CHANNEL_WIDTH));
    }

    #[test]
    fn values_run_up_the_window_and_the_base_follows_the_mode() {
        let mut v = BTimeView::of_range(1, 250, 24.0);
        assert_eq!(v.y_of(0.5, 400.0), 400.0, "without a value axis everything sits on the foot");
        v.values = Some((-9.0, 10.0));
        assert_eq!(v.y_of(10.0, 400.0), 0.0);
        assert_eq!(v.y_of(-9.0, 400.0), 400.0);
        assert!((v.value_scale(380.0) - 20.0).abs() < 1e-6);
        assert_eq!(v.base(), 24, "time editors step by the frame rate");
        v.drivers = true;
        assert_eq!(v.base(), DRIVER_STEP, "the Drivers editor steps in tens");
    }
}
