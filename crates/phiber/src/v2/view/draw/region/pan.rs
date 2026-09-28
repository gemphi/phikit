//! Middle-drag panning of a header strip — Blender's `view2d.pan` on a
//! header region: with the middle button held, the strip follows the
//! pointer sideways, as far as it has content to show and no further. The
//! wheel already scrolls it (`BFlex::strip_scroller`); this shares that
//! offset through one [`ScrollHandle`] per strip, kept across frames so a
//! re-render does not put the strip back at its start.

use gpui::prelude::*;
use super::super::ctx::BRenderCtx;
use super::super::flex::{BFlex, BPlace};
use gpui::{DispatchPhase, Div, MouseButton, MouseDownEvent, MouseMoveEvent, ScrollHandle, Stateful, canvas, point, px};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;

thread_local! {
    /// One handle per strip, by its element id.
    static HANDLES: RefCell<HashMap<String, ScrollHandle>> = RefCell::new(HashMap::new());
    /// Where a middle-drag began: the pointer's x and the strip's offset.
    static GRAB: Cell<Option<(f32, f32)>> = const { Cell::new(None) };
}

/// A strip's pan: its handle and the gesture on it.
pub(super) struct BStripPan {
    handle: ScrollHandle,
}

impl BStripPan {
    pub fn of(id: &str) -> Self {
        let handle = HANDLES.with(|h| h.borrow_mut().entry(id.to_string()).or_default().clone());
        Self { handle }
    }

    /// The offset `start` pans to after the pointer moved `dx`, held inside
    /// the strip's content: `0` is its start, `-max` its end.
    pub fn clamp(start: f32, dx: f32, max: f32) -> f32 {
        (start + dx).clamp(-max.max(0.0), 0.0)
    }

    /// The strip, tracking the shared offset and listening for the drag.
    /// The press is heard in the capture phase — a button under the pointer
    /// must not keep the strip from seeing it — and the drag window-wide,
    /// so the pan goes on when the pointer leaves the strip, as Blender's
    /// does (an unpainted overlay registers the listener each frame).
    pub fn attach(self, strip: Stateful<Div>, ctx: &BRenderCtx) -> Stateful<Div> {
        let (down, moved) = (self.handle.clone(), self.handle.clone());
        let listener = canvas(
            |_, _, _| (),
            move |_, (), window, _| {
                let moved = moved.clone();
                window.on_mouse_event(move |e: &MouseMoveEvent, phase, window, _| {
                    let Some((x0, start)) = GRAB.get() else { return };
                    if phase != DispatchPhase::Bubble {
                        return;
                    }
                    if e.pressed_button != Some(MouseButton::Middle) {
                        GRAB.set(None);
                        return;
                    }
                    let x = Self::clamp(start, f32::from(e.position.x) - x0, f32::from(moved.max_offset().x));
                    moved.set_offset(point(px(x), moved.offset().y));
                    window.refresh();
                });
            },
        );
        strip
            .track_scroll(&self.handle)
            .capture_any_mouse_down(move |e: &MouseDownEvent, _, _| {
                if e.button == MouseButton::Middle {
                    GRAB.set(Some((f32::from(e.position.x), f32::from(down.offset().x))));
                }
            })
            .child(BFlex::column().place(BPlace::Overlay).styled(listener, ctx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pan_stays_inside_the_strip() {
        assert_eq!(BStripPan::clamp(0.0, -30.0, 100.0), -30.0);
        assert_eq!(BStripPan::clamp(-90.0, -30.0, 100.0), -100.0, "no further than the end");
        assert_eq!(BStripPan::clamp(-10.0, 50.0, 100.0), 0.0, "no further than the start");
        assert_eq!(BStripPan::clamp(0.0, -30.0, 0.0), 0.0, "a strip that fits does not move");
    }
}
