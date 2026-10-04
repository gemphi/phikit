//! Panning and zooming the node canvas.
//!
//! The `View2D` keymap, as the node editor binds it
//! (`editors/interface/view2d/view2d_ops.cc`, registered for every 2D region):
//!
//! * `VIEW2D_OT_pan` — the middle mouse button dragged. Blender also pans on
//!   Ctrl+wheel and the trackpad; this is the button, which is the one every
//!   node editor user has in their hands.
//! * `VIEW2D_OT_zoom_in` / `_out` — the wheel, zooming about the cursor
//!   (`zoom_to_mouse_pos`), so the node under the pointer stays under it.
//!
//! Both write `NodeKey::View` with `Adjust`, for the same reason the drag does:
//! a gesture writes continuously, and a `Set` per wheel notch would take every
//! open overlay down. Nothing commits on release — the value written during the
//! gesture is already the value.
//!
//! The arithmetic is all in `view/mod.rs`, ported and tested there. What lives
//! here is only the wiring: which button, which delta, and the fact that a pan
//! measured in fractions of the canvas needs no pixel size
//! (`BNodeView::pan_fraction`).

use super::BWire;
use crate::v2::state::NodeKey;
use gpui::{App, Point, Pixels, Window};
use std::cell::Cell;

thread_local! {
    /// Where the middle button last was, in canvas fractions.
    ///
    /// A pan is a delta per move, so it needs the previous position — and the
    /// press that began it re-renders the window, which replaces the element and
    /// its closures, so this outlives the element that set it. The animation
    /// window's `PRESSED` is the same arrangement for the same reason
    /// (`canvas/time/gesture.rs`).
    static PANNING: Cell<Option<(f32, f32)>> = const { Cell::new(None) };
}

impl BWire {
    /// The middle button went down: a pan begins here.
    pub fn pan_start(&self, at: Point<Pixels>) {
        let Some((cursor, region)) = self.locate(at) else { return };
        PANNING.set(Some((cursor.0 / region.0, cursor.1 / region.1)));
    }

    /// The middle button came up, wherever it was: the pan ends.
    ///
    /// Cleared unconditionally rather than only when it was panning, so a
    /// release off the canvas cannot leave a pan that resumes on the next move.
    pub fn pan_end(&self) {
        PANNING.set(None);
    }

    /// The pointer moved with the middle button held: the view goes the other
    /// way, so the graph follows the pointer.
    pub fn pan_move(&self, at: Point<Pixels>, window: &mut Window, app: &mut App) {
        let Some(from) = PANNING.get() else { return };
        let Some((cursor, region)) = self.locate(at) else { return };
        let now = (cursor.0 / region.0, cursor.1 / region.1);
        PANNING.set(Some(now));
        let next = self.view.pan_fraction((now.0 - from.0, now.1 - from.1));
        self.write(NodeKey::View(self.host), super::super::view::store::encode(&next), window, app);
    }

    /// Whether a pan is in flight — what decides if the move handler is hung.
    pub fn panning() -> bool {
        PANNING.get().is_some()
    }

    /// A wheel notch: one zoom step about the cursor.
    ///
    /// `view_zoomstep_apply_ex` refuses a step that would leave the zoom limits
    /// rather than clamping it, so a wheel at the far end does nothing — and
    /// then there is nothing to write, which is why the equality check is here
    /// rather than a write every notch.
    pub fn zoom(&self, event: &gpui::ScrollWheelEvent, window: &mut Window, app: &mut App) {
        let notches = match event.delta {
            gpui::ScrollDelta::Lines(p) => p.y,
            // The same twenty pixels a line is worth in the animation window.
            gpui::ScrollDelta::Pixels(p) => f32::from(p.y) / 20.0,
        };
        if notches.abs() < 1e-3 {
            return;
        }
        let Some((cursor, region)) = self.locate(event.position) else { return };
        let step = super::super::view::ZOOM_STEP * notches.signum();
        let next = self.view.zoom_step(step, Some(cursor), region);
        if next == self.view {
            return;
        }
        self.write(NodeKey::View(self.host), super::super::view::store::encode(&next), window, app);
    }
}
