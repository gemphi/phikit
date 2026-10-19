//! Orbit, pan and dolly over the graph.
//!
//! Spatial is a view, so the left button — the editor's selection and drag
//! button in flat — has nothing to grab here and turns the camera instead; the
//! middle button pans and the wheel dollies, which are the flat canvas's own
//! bindings for the same two verbs (`wire/frame.rs`, Blender's `View2D`
//! keymap), so a hand that knows the flat editor knows this one.
//!
//! Only the orbit is written as itself (`NodeKey::Orbit`). A pan writes the
//! FLAT view through `BNodeView::pan_fraction` and a dolly steps it with
//! `zoom_step` about its centre — the camera's pivot and distance are that
//! view's (`camera.rs`), so the flat editor is already looking at the same
//! place when the mode is switched back. Every write is an `Adjust`, as every
//! gesture's is: a `Set` per pixel would take each open overlay down.

use super::camera::BOrbit;
use super::set::TURN_PER_WIDTH;
use super::super::super::super::ctx::{BActionHandler, BRenderCtx};
use super::super::view::{BNodeView, ZOOM_STEP, store as view_store};
use crate::v2::host::BHostType;
use crate::v2::state::{BKey as _, NodeKey};
use crate::v2::view::prim::BAction;
use gpui::{App, Bounds, Pixels, Point, Window};
use std::cell::Cell;
use std::rc::Rc;

thread_local! {
    /// Where the left button last was, in canvas fractions, while orbiting.
    static ORBITING: Cell<Option<(f32, f32)>> = const { Cell::new(None) };
    /// Where the middle button last was, in canvas fractions, while panning.
    static PANNING: Cell<Option<(f32, f32)>> = const { Cell::new(None) };
}

/// One render's worth of camera gesturing.
pub struct BSpatialGesture {
    pub host: BHostType,
    pub handler: BActionHandler,
    pub bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
    pub view: BNodeView,
    pub orbit: BOrbit,
}

impl BSpatialGesture {
    /// The cursor as a fraction of the canvas, and the canvas's size — `None`
    /// before the first paint.
    fn locate(&self, at: Point<Pixels>) -> Option<((f32, f32), (f32, f32))> {
        let bounds = self.bounds.get()?;
        let (w, h) = (f32::from(bounds.size.width), f32::from(bounds.size.height));
        if w <= 0.0 || h <= 0.0 {
            return None;
        }
        let x = (f32::from(at.x) - f32::from(bounds.origin.x)) / w;
        let y = (f32::from(at.y) - f32::from(bounds.origin.y)) / h;
        Some(((x, y), (w, h)))
    }

    fn write(&self, key: NodeKey, value: String, window: &mut Window, app: &mut App) {
        (self.handler)(&BAction::Adjust(key.path(), value), window, app);
    }

    pub fn orbit_start(&self, at: Point<Pixels>) {
        if let Some((cursor, _)) = self.locate(at) {
            ORBITING.set(Some(cursor));
        }
    }

    pub fn pan_start(&self, at: Point<Pixels>) {
        if let Some((cursor, _)) = self.locate(at) {
            PANNING.set(Some(cursor));
        }
    }

    /// Either button up, wherever it was: both gestures end. Cleared
    /// unconditionally so a release off the canvas cannot leave one running.
    pub fn end(&self) {
        ORBITING.set(None);
        PANNING.set(None);
    }

    /// Whether a gesture is in flight — what decides if the move handler is hung.
    pub fn moving() -> bool {
        ORBITING.get().is_some() || PANNING.get().is_some()
    }

    /// The pointer moved: turn the camera, or carry the view.
    pub fn moved(&self, at: Point<Pixels>, window: &mut Window, app: &mut App) {
        let Some((now, _)) = self.locate(at) else { return };
        if let Some(from) = ORBITING.get() {
            ORBITING.set(Some(now));
            // Dragging right turns the eye round to the right; dragging down
            // brings it down toward the floor.
            let by = ((now.0 - from.0) * TURN_PER_WIDTH, (now.1 - from.1) * TURN_PER_WIDTH);
            self.write(NodeKey::Orbit(self.host), self.orbit.turned(by).encode(), window, app);
        }
        if let Some(from) = PANNING.get() {
            PANNING.set(Some(now));
            let next = self.view.pan_fraction((now.0 - from.0, now.1 - from.1));
            self.write(NodeKey::View(self.host), view_store::encode(&next), window, app);
        }
    }

    /// A wheel notch: one step of the flat view's zoom about its centre, which
    /// moves the eye in or out along its line of sight.
    pub fn dolly(&self, event: &gpui::ScrollWheelEvent, window: &mut Window, app: &mut App) {
        let notches = match event.delta {
            gpui::ScrollDelta::Lines(p) => p.y,
            gpui::ScrollDelta::Pixels(p) => f32::from(p.y) / 20.0,
        };
        if notches.abs() < 1e-3 {
            return;
        }
        let Some((_, region)) = self.locate(event.position) else { return };
        let next = self.view.zoom_step(ZOOM_STEP * notches.signum(), None, region);
        if next == self.view {
            return;
        }
        self.write(NodeKey::View(self.host), view_store::encode(&next), window, app);
    }
}

/// Hang the handlers on the canvas.
pub fn listen(canvas: gpui::Div, gesture: Rc<BSpatialGesture>, _ctx: &BRenderCtx) -> gpui::Div {
    use gpui::prelude::FluentBuilder as _;
    use gpui::{InteractiveElement, MouseButton};
    let g = |g: &Rc<BSpatialGesture>| Rc::clone(g);
    let (orbit, pan, up_l, up_m, out_l, out_m, wheel) =
        (g(&gesture), g(&gesture), g(&gesture), g(&gesture), g(&gesture), g(&gesture), g(&gesture));
    canvas
        .on_mouse_down(MouseButton::Left, move |e: &gpui::MouseDownEvent, _w, _a| orbit.orbit_start(e.position))
        .on_mouse_down(MouseButton::Middle, move |e: &gpui::MouseDownEvent, _w, _a| pan.pan_start(e.position))
        .on_mouse_up(MouseButton::Left, move |_e: &gpui::MouseUpEvent, _w, _a| up_l.end())
        .on_mouse_up(MouseButton::Middle, move |_e: &gpui::MouseUpEvent, _w, _a| up_m.end())
        .on_mouse_up_out(MouseButton::Left, move |_e: &gpui::MouseUpEvent, _w, _a| out_l.end())
        .on_mouse_up_out(MouseButton::Middle, move |_e: &gpui::MouseUpEvent, _w, _a| out_m.end())
        .on_scroll_wheel(move |e: &gpui::ScrollWheelEvent, w, a| wheel.dolly(e, w, a))
        .when(BSpatialGesture::moving(), |element| {
            element.on_mouse_move(move |e: &gpui::MouseMoveEvent, w, a| gesture.moved(e.position, w, a))
        })
}
