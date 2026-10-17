//! `View2D.cur` along time, as Blender moves it (`view2d_ops.cc`,
//! `action_edit.cc`, `anim_draw.cc`): a wheel step zooms about the
//! view's centre, Ctrl wheel scrolls by forty pixels' worth of frames,
//! Frame All stretches the keys across the window with a margin of four
//! diamonds, and the zoom is held between `minzoom` and `maxzoom`.
//! Where the view is kept is `TimeViewKey`; what applies these is `op.rs`.

use crate::v2::host::BHostType;
use crate::v2::state::{BState, TimeViewKey};
use std::cell::RefCell;
use std::collections::HashMap;

/// `view_zoomfac_default`: one wheel step takes this share of the view off
/// each end (and gives back the same, symmetrically, zooming out).
pub const ZOOM_STEP: f32 = 0.0375;
/// `VIEW2D_OT_scroll_left / _right`: `deltax = 40 × UI_SCALE_FAC` pixels.
pub const SCROLL_PX: f32 = 40.0;
/// `action_create`: `v2d.minzoom = 0.01`, `v2d.maxzoom = 50` pixels a frame.
pub const MIN_ZOOM: f32 = 0.01;
pub const MAX_ZOOM: f32 = 50.0;
/// `ANIM_frame_range_view2d_add_xmargin`: four diamonds of ten pixels,
/// capped at an eighth of the view.
const MARGIN_PX: f32 = 4.0 * 10.0;
const MARGIN_MAX: f32 = 0.125;

/// The visible range `first..last` of one editor.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BTimeCur {
    pub first: f32,
    pub last: f32,
}

thread_local! {
    /// Each editor's window width at its last paint, in pixels: what the
    /// operators scale by (Blender reads `v2d.mask` of the region in hand;
    /// a key or a menu entry here has no region, only the editor kind).
    static WIDTHS: RefCell<HashMap<BHostType, f32>> = RefCell::new(HashMap::new());
}

impl BTimeCur {
    /// The range kept for `host`, if it has been moved from its default.
    pub fn stored(s: &BState, host: BHostType) -> Option<Self> {
        let raw = s.get(TimeViewKey(host))?;
        let (a, b) = raw.split_once(',')?;
        let (first, last) = (a.parse().ok()?, b.parse().ok()?);
        (last > first).then_some(Self { first, last })
    }

    pub fn encode(self) -> String {
        format!("{:.4},{:.4}", self.first, self.last)
    }

    pub fn width(self) -> f32 {
        self.last - self.first
    }

    /// `view_zoomstep_apply_ex` for `steps` wheel notches (negative out),
    /// about the centre (`USER_ZOOM_TO_MOUSEPOS` is off by default): in,
    /// `dx = size × fac` off each end; out, `dx = size / (1 + 2 fac) × fac`
    /// — so a step out undoes a step in exactly. Then held to the zoom
    /// limits over `px` pixels of window.
    pub fn zoomed(self, steps: f32, px: f32) -> Self {
        let fac = ZOOM_STEP * steps;
        let size = self.width();
        let dx = if fac >= 0.0 { size * fac } else { size / (1.0 + 2.0 * fac) * fac };
        Self { first: self.first + dx, last: self.last - dx }.limited(px)
    }

    /// `view_pan_apply_ex` with `deltax` pixels: the view moves by that many
    /// pixels' worth of frames (`facx = size / winx`).
    pub fn scrolled(self, pixels: f32, px: f32) -> Self {
        let dx = pixels * self.width() / px.max(1.0);
        Self { first: self.first + dx, last: self.last + dx }
    }

    /// `ANIM_center_frame` (`anim_ops.cc`): pan the view so `frame` is at
    /// the centre, keeping the current width. Called by `ACTION_OT_view_frame`
    /// (Numpad 0, Go to Current Frame) and its graph / NLA / sequencer kin.
    pub fn centered(self, frame: f32) -> Self {
        let half = self.width() / 2.0;
        Self { first: frame - half, last: frame + half }
    }

    /// `actkeys_viewall` over keys spanning `min..max`: a lone key (or
    /// none apart) is centred at the width in hand; a span fills the
    /// window with the margin.
    pub fn framed(self, min: f32, max: f32, px: f32) -> Self {
        if (max - min).abs() < 1.0 {
            let half = self.width() / 2.0;
            return Self { first: min - half, last: max + half };
        }
        Self { first: min, last: max }.with_margin(px)
    }

    /// `ANIM_frame_range_view2d_add_xmargin`.
    pub fn with_margin(self, px: f32) -> Self {
        let size = self.width();
        let margin = (MARGIN_PX * size / px.max(1.0)).min(MARGIN_MAX * size);
        Self { first: self.first - margin, last: self.last + margin }
    }

    /// Held between `minzoom` and `maxzoom` pixels a frame, about the centre.
    fn limited(self, px: f32) -> Self {
        let px = px.max(1.0);
        let size = self.width().clamp(px / MAX_ZOOM, px / MIN_ZOOM);
        let mid = (self.first + self.last) / 2.0;
        Self { first: mid - size / 2.0, last: mid + size / 2.0 }
    }

    /// The painter says how wide `host`'s window is.
    pub fn note_width(host: BHostType, px: f32) {
        WIDTHS.with(|w| {
            w.borrow_mut().insert(host, px);
        });
    }

    /// The width the painter last saw for `host` (a thousand pixels before
    /// any paint).
    pub fn width_of(host: BHostType) -> f32 {
        WIDTHS.with(|w| w.borrow().get(&host).copied()).unwrap_or(1000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PX: f32 = 1000.0;

    #[test]
    fn a_step_in_and_a_step_out_meet_where_they_began() {
        let cur = BTimeCur { first: -9.0, last: 260.0 };
        let zoomed = cur.zoomed(1.0, PX);
        assert!((zoomed.first - (-9.0 + 269.0 * 0.0375)).abs() < 1e-3, "{zoomed:?}: 3.75% off each end");
        let back = zoomed.zoomed(-1.0, PX);
        assert!((back.first - cur.first).abs() < 1e-3 && (back.last - cur.last).abs() < 1e-3, "{back:?}");
        let tight = BTimeCur { first: 0.0, last: 10.0 }.zoomed(3.0, PX);
        assert!((tight.width() - PX / MAX_ZOOM).abs() < 1e-3, "no closer than fifty pixels a frame: {tight:?}");
    }

    #[test]
    fn ctrl_wheel_scrolls_forty_pixels_of_frames() {
        let cur = BTimeCur { first: 0.0, last: 500.0 };
        let right = cur.scrolled(SCROLL_PX, PX);
        assert_eq!((right.first, right.last), (20.0, 520.0), "500 frames over 1000 px: 40 px is 20 frames");
    }

    #[test]
    fn frame_all_fills_the_window_with_four_diamonds_to_spare() {
        let cur = BTimeCur { first: -9.0, last: 260.0 };
        let framed = cur.framed(10.0, 90.0, PX);
        // 80 frames over 1000 px → 40 px is 3.2 frames, under the 10-frame cap.
        assert!((framed.first - 6.8).abs() < 1e-3 && (framed.last - 93.2).abs() < 1e-3, "{framed:?}");
        let wide = cur.framed(0.0, 100.0, 100.0);
        assert!((wide.first + 12.5).abs() < 1e-3, "a narrow window: the margin caps at an eighth: {wide:?}");
        let lone = cur.framed(40.0, 40.0, PX);
        assert_eq!((lone.first, lone.last), (40.0 - 134.5, 40.0 + 134.5), "one key: centred at the width in hand");
        let mut s = BState::default();
        assert_eq!(BTimeCur::stored(&s, BHostType::Time), None);
        s.set(TimeViewKey(BHostType::Time), framed.encode());
        assert_eq!(BTimeCur::stored(&s, BHostType::Time).map(BTimeCur::encode), Some(framed.encode()));
        assert_eq!(BTimeCur::stored(&s, BHostType::Action), None, "each editor keeps its own");
    }
}
