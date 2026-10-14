//! Viewport navigation from a scroll gesture — Blender's trackpad and wheel
//! bindings for the 3D View (`blender_default.py`, 3D View ▸ Navigation, with
//! `use_v3d_mmb_pan` off, the default):
//!
//! - `TRACKPADPAN` (a two-finger swipe) **rotates** the view; with Shift it
//!   **moves** (pans) it; with Ctrl it **zooms**. `TRACKPADZOOM` (a pinch)
//!   zooms too — gpui does not surface a pinch, so Ctrl-swipe and the wheel
//!   are a trackpad's zoom here.
//! - `WHEELINMOUSE` / `WHEELOUTMOUSE` **zoom**. The 3D View binds no modifier
//!   variant of the wheel, so Shift / Ctrl leave it zooming.
//!
//! The canvas (`draw::canvas::view3d`) hands each event's device and modifiers
//! to [`scroll_nav`], accumulates the displacement since its last draw, and
//! fires the sets [`scroll_sets`] returns. So a swipe orbits with the same
//! degrees per pixel as a pointer drag ([`BTurntable`]), a pan follows the
//! fingers one pixel for one, and a zoom steps as the wheel always has — one
//! notch per [`PX_PER_LINE`], smoothly across a swipe.
//!
//! A pointer drag over the canvas follows Blender's middle-mouse scheme
//! ([`drag_nav`]): `MIDDLEMOUSE` **rotates**, Shift+`MIDDLEMOUSE` **moves**,
//! Ctrl+`MIDDLEMOUSE` **zooms** (Shift+Ctrl dollies — a zoom here). Phiber
//! orbits on the *left* button so a laptop can, so the modifiers ride that
//! drag instead; the maths is the scroll's ([`drag_sets`]).
//!
//! Deliberate divergences, recorded: Shift+Ctrl+swipe (bound to nothing in
//! Blender) pans here rather than doing nothing; the drag scheme sits on the
//! left button, not the middle.

use super::orbit::BTurntable;
use super::{pan_x_key, pan_y_key, pitch_key, yaw_key, zoom_key, zoom_step};
use crate::v2::state::BPath;

/// Where a scroll came from, as gpui reports its delta: a trackpad's pixel
/// deltas (`ScrollDelta::Pixels`) or a wheel's line steps (`ScrollDelta::Lines`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BScrollSource {
    Trackpad,
    Wheel,
}

/// What a scroll does to the view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BScrollNav {
    Orbit,
    Pan,
    Zoom,
}

/// Blender's binding for a scroll of `source` under these modifiers.
pub fn scroll_nav(source: BScrollSource, shift: bool, ctrl: bool) -> BScrollNav {
    match (source, shift, ctrl) {
        (BScrollSource::Trackpad, true, _) => BScrollNav::Pan,
        (BScrollSource::Trackpad, false, true) => BScrollNav::Zoom,
        (BScrollSource::Trackpad, false, false) => BScrollNav::Orbit,
        (BScrollSource::Wheel, _, _) => BScrollNav::Zoom,
    }
}

/// One wheel notch, in the pixels a trackpad would have swiped — the zoom's
/// step per notch and a pan's distance per notch both follow from it.
pub const PX_PER_LINE: f32 = 20.0;
/// The zoom factor per [`PX_PER_LINE`] of scroll — one wheel notch.
pub const ZOOM_PER_LINE: f32 = 1.1;

/// The view a gesture starts from, read when the canvas drew.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BNavBase {
    pub yaw: f32,
    pub pitch: f32,
    pub pan: (f32, f32),
    pub zoom: f32,
}

/// The view keys to set for `nav` after `acc` pixels of scroll `(dx, dy)`
/// (`dy` down) since the base, over a canvas `width` pixels wide.
pub fn scroll_sets(nav: BScrollNav, base: BNavBase, acc: (f32, f32), width: f32) -> Vec<(BPath, String)> {
    match nav {
        // The same turntable as a pointer drag: the swipe is the drag.
        BScrollNav::Orbit => {
            let (yaw, pitch) = BTurntable::start(base.yaw, base.pitch).at(acc.0, acc.1);
            vec![(yaw_key(), yaw.to_string()), (pitch_key(), pitch.to_string())]
        }
        // Pan is in canvas widths (`camera::view_project`), so a pixel of
        // scroll moves the view one pixel — it follows the fingers.
        BScrollNav::Pan => {
            let w = width.max(1.0);
            vec![
                (pan_x_key(), (base.pan.0 + acc.0 / w).to_string()),
                (pan_y_key(), (base.pan.1 + acc.1 / w).to_string()),
            ]
        }
        // A notch is one factor; a swipe steps fractionally, so it is smooth.
        BScrollNav::Zoom => {
            let factor = ZOOM_PER_LINE.powf(acc.1 / PX_PER_LINE);
            vec![(zoom_key(), zoom_step(base.zoom, factor).to_string())]
        }
    }
}

/// What a pointer drag does to the view — Blender's `MIDDLEMOUSE` bindings,
/// on the drag that orbits here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BDragNav {
    Orbit,
    Pan,
    Zoom,
}

/// Blender's binding for a drag under these modifiers: `view3d.rotate`,
/// Shift → `view3d.move`, Ctrl → `view3d.zoom`, Shift+Ctrl → `view3d.dolly`
/// (a zoom here, the view having no separate dolly).
pub fn drag_nav(shift: bool, ctrl: bool) -> BDragNav {
    match (shift, ctrl) {
        (true, false) => BDragNav::Pan,
        (_, true) => BDragNav::Zoom,
        (false, false) => BDragNav::Orbit,
    }
}

/// The view keys to set for a drag of `nav` that has travelled `total`
/// pixels `(dx, dy)` (`dy` down) from where it began, from the base view,
/// over a canvas `width` pixels wide. Orbit and pan are the scroll's; a
/// zoom drag goes Blender's way — the pointer up zooms in (`viewzoom_apply`,
/// the default vertical style), so the sign is turned.
pub fn drag_sets(nav: BDragNav, base: BNavBase, total: (f32, f32), width: f32) -> Vec<(BPath, String)> {
    match nav {
        BDragNav::Orbit => scroll_sets(BScrollNav::Orbit, base, total, width),
        BDragNav::Pan => scroll_sets(BScrollNav::Pan, base, total, width),
        BDragNav::Zoom => scroll_sets(BScrollNav::Zoom, base, (0.0, -total.1), width),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: BNavBase = BNavBase { yaw: 30.0, pitch: 30.0, pan: (0.0, 0.0), zoom: 1.0 };

    fn value(sets: &[(BPath, String)], key: &BPath) -> f32 {
        sets.iter().find(|(k, _)| k == key).map(|(_, v)| v.parse().unwrap()).unwrap_or_else(|| panic!("no set for {key:?} in {sets:?}"))
    }

    #[test]
    fn a_swipe_orbits_shift_pans_ctrl_zooms_and_the_wheel_always_zooms() {
        use BScrollNav::*;
        use BScrollSource::*;
        assert_eq!(scroll_nav(Trackpad, false, false), Orbit, "TRACKPADPAN → view3d.rotate");
        assert_eq!(scroll_nav(Trackpad, true, false), Pan, "Shift TRACKPADPAN → view3d.move");
        assert_eq!(scroll_nav(Trackpad, false, true), Zoom, "Ctrl TRACKPADPAN → view3d.zoom");
        assert_eq!(scroll_nav(Trackpad, true, true), Pan, "Shift wins where Blender binds nothing");
        for (shift, ctrl) in [(false, false), (true, false), (false, true), (true, true)] {
            assert_eq!(scroll_nav(Wheel, shift, ctrl), Zoom, "the wheel zooms, modifiers or not");
        }
    }

    #[test]
    fn an_orbit_turns_with_the_drag_s_degrees_per_pixel() {
        // 40 px right: yaw turns 40 × 0.5°, as a pointer drag would; pitch holds.
        let sets = scroll_sets(BScrollNav::Orbit, BASE, (40.0, 0.0), 1000.0);
        assert!((value(&sets, &yaw_key()) - (30.0 + 40.0 * BTurntable::DEG_PER_PX)).abs() < 1e-4);
        assert!((value(&sets, &pitch_key()) - 30.0).abs() < 1e-4);
        // 20 px down tips the pitch.
        let sets = scroll_sets(BScrollNav::Orbit, BASE, (0.0, 20.0), 1000.0);
        assert!((value(&sets, &pitch_key()) - (30.0 + 20.0 * BTurntable::DEG_PER_PX)).abs() < 1e-4);
    }

    #[test]
    fn a_pan_follows_the_fingers_one_pixel_for_one() {
        // Pan is in canvas widths: 100 px across a 1000 px canvas is 0.1.
        let sets = scroll_sets(BScrollNav::Pan, BASE, (100.0, 50.0), 1000.0);
        assert!((value(&sets, &pan_x_key()) - 0.1).abs() < 1e-6);
        assert!((value(&sets, &pan_y_key()) - 0.05).abs() < 1e-6);
        // From a view already panned, it adds.
        let panned = BNavBase { pan: (0.2, -0.1), ..BASE };
        let sets = scroll_sets(BScrollNav::Pan, panned, (100.0, 0.0), 1000.0);
        assert!((value(&sets, &pan_x_key()) - 0.3).abs() < 1e-6);
        assert!((value(&sets, &pan_y_key()) - -0.1).abs() < 1e-6);
    }

    #[test]
    fn a_drag_orbits_shift_drag_pans_ctrl_drag_zooms() {
        use BDragNav::*;
        assert_eq!(drag_nav(false, false), Orbit, "MIDDLEMOUSE → view3d.rotate");
        assert_eq!(drag_nav(true, false), Pan, "Shift MIDDLEMOUSE → view3d.move");
        assert_eq!(drag_nav(false, true), Zoom, "Ctrl MIDDLEMOUSE → view3d.zoom");
        assert_eq!(drag_nav(true, true), Zoom, "Shift Ctrl MIDDLEMOUSE → view3d.dolly, a zoom here");
        // The drag's orbit and pan are the scroll's, pixel for pixel.
        assert_eq!(drag_sets(Orbit, BASE, (40.0, 20.0), 1000.0), scroll_sets(BScrollNav::Orbit, BASE, (40.0, 20.0), 1000.0));
        assert_eq!(drag_sets(Pan, BASE, (100.0, 50.0), 1000.0), scroll_sets(BScrollNav::Pan, BASE, (100.0, 50.0), 1000.0));
        // A zoom drag: the pointer moving UP (dy < 0) zooms IN, as Blender's does.
        let up = drag_sets(Zoom, BASE, (0.0, -PX_PER_LINE), 1000.0);
        assert!((value(&up, &zoom_key()) - zoom_step(1.0, ZOOM_PER_LINE)).abs() < 1e-5, "up = in");
        let down = drag_sets(Zoom, BASE, (0.0, PX_PER_LINE), 1000.0);
        assert!((value(&down, &zoom_key()) - zoom_step(1.0, 1.0 / ZOOM_PER_LINE)).abs() < 1e-5, "down = out");
        // Sideways travel does not zoom.
        let side = drag_sets(Zoom, BASE, (300.0, 0.0), 1000.0);
        assert!((value(&side, &zoom_key()) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn a_zoom_steps_one_factor_per_wheel_notch_and_smoothly_between() {
        // One notch (PX_PER_LINE) is the wheel's factor, either way.
        let in_ = scroll_sets(BScrollNav::Zoom, BASE, (0.0, PX_PER_LINE), 1000.0);
        assert!((value(&in_, &zoom_key()) - zoom_step(1.0, ZOOM_PER_LINE)).abs() < 1e-5);
        let out = scroll_sets(BScrollNav::Zoom, BASE, (0.0, -PX_PER_LINE), 1000.0);
        assert!((value(&out, &zoom_key()) - zoom_step(1.0, 1.0 / ZOOM_PER_LINE)).abs() < 1e-5);
        // Half a notch is the square root of the factor — a swipe is continuous.
        let half = scroll_sets(BScrollNav::Zoom, BASE, (0.0, PX_PER_LINE / 2.0), 1000.0);
        assert!((value(&half, &zoom_key()) - zoom_step(1.0, ZOOM_PER_LINE.sqrt())).abs() < 1e-5);
        // Zoom only ever sets the zoom key.
        assert_eq!(in_.len(), 1);
    }
}
