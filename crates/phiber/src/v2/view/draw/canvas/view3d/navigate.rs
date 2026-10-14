//! What a wheel, a trackpad or a pinch asks the viewport to do — Blender's
//! 3D view keymap, not a guess.
//!
//! `wm_event_system.cc` turns one GHOST trackpad event into one of four
//! Blender events by its subtype, and
//! `keymap_data/blender_default.py`'s `Navigation` block maps each to an
//! operator:
//!
//! | the finger does | GHOST subtype | Blender event | operator |
//! | --- | --- | --- | --- |
//! | pinch | `Magnify` | `MOUSEZOOM` | `view3d.zoom` |
//! | two-finger drag | `Scroll` | `TRACKPADPAN` | **`view3d.rotate`** |
//! | two-finger drag + shift | `Scroll` | `TRACKPADPAN` | `view3d.move` |
//! | two-finger twist | `Rotate` | `MOUSEROTATE` | `view3d.rotate` |
//! | wheel notch | — | `WHEELUP/DOWNMOUSE` | `view3d.zoom` |
//!
//! The one that surprises people porting this: a two-finger **drag**
//! orbits, it does not zoom. Zoom is the **pinch**, and only the pinch.
//! Treating every vertical scroll as zoom — which is what this viewport did
//! before — means a trackpad can never orbit and a pinch does nothing at
//! all, because `on_pinch` was never wired even though gpui delivers it
//! (`PlatformInput::Pinch`, `gpui_macos/src/events.rs`).
//!
//! Blender also inverts a magnify's delta (`delta[0] = -delta[0]`) and
//! honours the system's own "natural scrolling" through
//! `WM_EVENT_SCROLL_INVERT`; the platform has already applied the user's
//! preference by the time gpui hands us a delta, so there is nothing to
//! invert here beyond the magnify itself.

use crate::v2::scene::{pan_x_key, pan_y_key, pitch_key, pitch_step, view_pan, view_pitch, view_yaw, view_zoom, yaw_key, zoom_key, zoom_step};
use crate::v2::state::BState;
use crate::v2::view::prim::BAction;
use gpui::{Modifiers, ScrollDelta, TouchPhase};

/// One wheel notch, in pixels — what `Lines` is worth. Blender's wheel
/// zoom is a fixed step per notch, not a distance.
const LINE: f32 = 20.0;
/// Below this a delta is noise, not a gesture.
const FLOOR: f32 = 0.5;
/// `view3d.zoom`'s step per notch (`view_zoom_step`).
const ZOOM_STEP: f32 = 1.1;
/// How far a trackpad drag turns the view: Blender's `view3d.rotate` is a
/// trackball over the region, and a pixel of drag is a degree of orbit at
/// the default sensitivity.
const ORBIT_PER_PIXEL: f32 = 0.5;
/// How far a shift-drag slides it, in view units per pixel.
const PAN_PER_PIXEL: f32 = 0.002;

/// What the viewport should do about a gesture.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BViewMove {
    /// `view3d.zoom`: multiply the zoom by this.
    Zoom { factor: f32 },
    /// `view3d.rotate`: turn by these degrees (yaw, pitch).
    Orbit { yaw: f32, pitch: f32 },
    /// `view3d.move`: slide by these view units.
    Pan { x: f32, y: f32 },
}

/// The keymap, as a type — one place that decides what an input means, so
/// the viewport element only has to deliver the answer.
pub struct BViewNavigate;

impl BViewNavigate {
    /// A pinch (`TRACKPADZOOM` → `view3d.zoom`). gpui's delta is a
    /// magnification: positive spreads the fingers, which zooms **in**.
    pub fn pinch(delta: f32) -> Option<BViewMove> {
        (delta.abs() > f32::EPSILON).then(|| BViewMove::Zoom { factor: 1.0 + delta })
    }

    /// A scroll. `Lines` is a mouse wheel and zooms; `Pixels` is a trackpad
    /// drag and orbits, or pans with shift.
    ///
    /// `delta` is taken already axis-filtered — see
    /// [`gpui::OngoingScroll::filter`], which locks a gesture to the axis it
    /// started on so a drag that wanders does not wobble the view.
    pub fn scroll(delta: ScrollDelta, modifiers: Modifiers, phase: TouchPhase) -> Option<BViewMove> {
        if matches!(phase, TouchPhase::Ended | TouchPhase::Cancelled) {
            return None;
        }
        match delta {
            // A wheel notch: `view3d.zoom`, a step per notch either way.
            ScrollDelta::Lines(point) => {
                let notches = point.y;
                if notches.abs() * LINE < FLOOR {
                    return None;
                }
                Some(BViewMove::Zoom { factor: if notches > 0.0 { ZOOM_STEP } else { 1.0 / ZOOM_STEP } })
            }
            // A trackpad drag: `TRACKPADPAN`, which orbits — and pans when
            // shift is held, exactly as the default keymap has it.
            ScrollDelta::Pixels(point) => {
                let (x, y) = (f32::from(point.x), f32::from(point.y));
                if x.abs() < FLOOR && y.abs() < FLOOR {
                    return None;
                }
                if modifiers.shift {
                    return Some(BViewMove::Pan { x: x * PAN_PER_PIXEL, y: y * PAN_PER_PIXEL });
                }
                // Dragging right turns the view right, and dragging down
                // tips it down, which is the direction a trackball gives.
                Some(BViewMove::Orbit { yaw: -x * ORBIT_PER_PIXEL, pitch: -y * ORBIT_PER_PIXEL })
            }
        }
    }

    /// Ctrl with a trackpad drag is Blender's `view3d.zoom` on
    /// `TRACKPADPAN` in the *2D* editors; the 3D view has no such binding,
    /// so a ctrl-drag orbits like any other. Kept as its own question so
    /// the answer is stated rather than implied by a missing branch.
    pub fn ctrl_drag_zooms() -> bool {
        false
    }
}

/// The view's own keys as they stand, and the one place a gesture's answer
/// is written back to them — so a pinch, a notch and a drag that mean the
/// same thing land the same way.
#[derive(Clone, Copy, Debug)]
pub struct BViewWrite {
    pub yaw: f32,
    pub pitch: f32,
    pub pan: (f32, f32),
    pub zoom: f32,
}

impl BViewWrite {
    pub fn now(state: &BState) -> Self {
        Self { yaw: view_yaw(state), pitch: view_pitch(state), pan: view_pan(state), zoom: view_zoom(state) }
    }

    /// Write `move_to` through `handler`. Zoom goes through `zoom_step` and
    /// pitch through `pitch_step`, so a gesture is clamped exactly as the
    /// keys themselves clamp — a drag cannot tip the view past the poles
    /// and a pinch cannot zoom past the limits.
    pub fn apply(
        &self,
        move_to: BViewMove,
        handler: &crate::v2::view::draw::ctx::BActionHandler,
        window: &mut gpui::Window,
        app: &mut gpui::App,
    ) {
        match move_to {
            BViewMove::Zoom { factor } => {
                handler(&BAction::Set(zoom_key(), zoom_step(self.zoom, factor).to_string()), window, app);
            }
            BViewMove::Orbit { yaw, pitch } => {
                handler(&BAction::Set(yaw_key(), (self.yaw + yaw).to_string()), window, app);
                handler(&BAction::Set(pitch_key(), pitch_step(self.pitch, pitch).to_string()), window, app);
            }
            BViewMove::Pan { x, y } => {
                handler(&BAction::Set(pan_x_key(), (self.pan.0 + x).to_string()), window, app);
                handler(&BAction::Set(pan_y_key(), (self.pan.1 + y).to_string()), window, app);
            }
        }
    }
}

impl BViewWrite {
    /// Attach the navigation gestures to the viewport's element: a wheel
    /// notch, a trackpad drag and a pinch, each routed through
    /// [`BViewNavigate`] and written by [`Self::apply`].
    ///
    /// `OngoingScroll` locks a drag to the axis it began on — gpui's own
    /// filter, which this viewport never used, and the reason a wandering
    /// drag used to wobble the view.
    pub fn attach(
        self,
        element: gpui::Stateful<gpui::Div>,
        handler: crate::v2::view::draw::ctx::BActionHandler,
    ) -> gpui::Stateful<gpui::Div> {
        use gpui::prelude::*;
        let scroll = handler.clone();
        let ongoing = std::cell::RefCell::new(gpui::OngoingScroll::default());
        element
            .on_scroll_wheel(move |event: &gpui::ScrollWheelEvent, window, app| {
                let mut delta = event.delta;
                if let ScrollDelta::Pixels(mut point) = delta {
                    ongoing.borrow_mut().filter(&mut point, event.touch_phase);
                    delta = ScrollDelta::Pixels(point);
                }
                if let Some(move_to) = BViewNavigate::scroll(delta, event.modifiers, event.touch_phase) {
                    self.apply(move_to, &scroll, window, app);
                }
            })
            .on_pinch(move |event: &gpui::PinchEvent, window, app| {
                if let Some(move_to) = BViewNavigate::pinch(event.delta) {
                    self.apply(move_to, &handler, window, app);
                }
            })
    }
}

#[cfg(test)]
#[path = "navigate_tests.rs"]
mod navigate_tests;
