//! Dragging a value along a track.
//!
//! `BCtrlKind::Slider` drew a bar filled to its value with a step either side, and
//! the bar was scenery: you could not grab it. So a slider was a number field
//! wearing a progress bar, and the one thing the shape promises — sweep across it
//! and the value follows — was the one thing it did not do. Nothing in this crate
//! turned a pointer's place inside an element into a value, which is the same gap
//! that left the colour picker four steppers rather than a wheel.
//!
//! This does. A track takes the key it writes, the range it spans and the axis it
//! runs along; while the pointer is down, where it sits IS the value. Every write
//! is an `Adjust`: a `Set` would dismiss the picker the slider sits in on the
//! first pixel of movement.
//!
//! The mechanics are `resize.rs`'s and `fold.rs`'s generalised: `DragMoveEvent`
//! carries the element's bounds and the pointer, which between them are the whole
//! of what a track needs. What a track looks like is `bar.rs`.

mod bar;
mod plane;
mod scale;

pub use bar::{share_of, view};
pub use plane::{BPlane, BPlaneAxis, view as plane_view};

use super::ctx::BRenderCtx;
use crate::v2::state::BPath;
use crate::v2::view::layout::BAxis;
use crate::v2::view::prim::BAction;
use gpui::prelude::*;
use gpui::{App, Bounds, Context, Div, DragMoveEvent, Pixels, Point, Render, Stateful, Window};
use std::rc::Rc;

/// What the dragged number becomes when it is written.
///
/// A slider writes the number itself. A colour's channel writes the whole colour
/// with one component replaced, because a colour is one key holding four numbers
/// — so the track has to know the other three to say anything at all. Both are
/// plain data, so the payload stays `Clone` and comparable and the gesture holds
/// no closure over the render pass.
#[derive(Clone, Debug, PartialEq)]
pub enum BTrackWrite {
    /// The number, formatted as the control's value.
    Value,
    /// Channel `index` of an HSLA colour, the rest of `color` carried along.
    Channel { index: usize, color: crate::v2::theme::BColor },
}

/// The drag in flight. A payload is cloned into the gesture, so it holds what the
/// value is computed from and nothing that changes under it.
#[derive(Clone, Debug, PartialEq)]
pub struct BTrack {
    pub key: BPath,
    pub min: f32,
    pub max: f32,
    /// What the number means at the key.
    pub write: BTrackWrite,
    /// Which way along the element the value grows. A `Row` track grows
    /// rightwards; a `Column` track grows UPWARDS, because a vertical slider
    /// reads from the bottom, where more is higher.
    pub axis: BAxis,
    /// Whole numbers only — an `Int` control's track.
    pub whole: bool,
}

impl BTrack {
    /// A horizontal track over `min..max`.
    pub fn across(key: BPath, min: f32, max: f32) -> Self {
        Self { key, min, max, write: BTrackWrite::Value, axis: BAxis::Row, whole: false }
    }

    /// A horizontal track over one channel of `color`.
    pub fn channel(key: BPath, index: usize, color: crate::v2::theme::BColor, min: f32, max: f32) -> Self {
        Self { write: BTrackWrite::Channel { index, color }, ..Self::across(key, min, max) }
    }

    /// Whole numbers only.
    pub fn whole(mut self) -> Self {
        self.whole = true;
        self
    }

    /// The write for the pointer's current place.
    pub fn adjust_at(&self, bounds: Bounds<Pixels>, pointer: Point<Pixels>) -> BAction {
        BAction::Adjust(self.key.clone(), self.value_at(bounds, pointer))
    }
}


/// `color` written the way a `BStyleDescriptor` formats and parses it back.
///
/// Stated once: a track, a plane and the picker's step buttons all write colours,
/// and a second spelling anywhere is a value the field beside them would reject.
pub fn encode_hsla(color: crate::v2::theme::BColor) -> String {
    let (h, s, l, a) = color;
    format!("{h}, {s}, {l}, {a}")
}

/// Nothing is dragged out of a track — the value moves, not the element — so the
/// gesture's drag image is empty.
pub(super) struct BTrackGhost;

impl Render for BTrackGhost {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        gpui::Empty
    }
}

/// Make `element` a draggable track for `track`.
///
/// The element must already be stateful (`.id(…)`), as every gesture target here
/// is.
///
/// The value follows the pointer from the first movement. A press with no
/// movement writes nothing: `on_drag`'s start callback is handed the pointer but
/// not the element, and a value needs both, so there is nothing honest to compute
/// there. Blender does jump to a clicked point, and getting that will take the
/// bounds reaching the press — not a cell outside the gesture holding whatever
/// the last track to move happened to be, which is what the first draft of this
/// did and would have crossed two sliders over each other.
pub fn draggable(element: Stateful<Div>, track: BTrack, ctx: &BRenderCtx) -> Stateful<Div> {
    let start = move |_: &BTrack, _: Point<Pixels>, _window: &mut Window, app: &mut App| {
        app.new(|_| BTrackGhost)
    };
    let over = {
        let handler = ctx.handler.clone();
        move |event: &DragMoveEvent<BTrack>, window: &mut Window, app: &mut App| {
            let track = event.drag(app).clone();
            (Rc::clone(&handler))(&track.adjust_at(event.bounds, event.event.position), window, app);
        }
    };
    element
        .cursor(gpui::CursorStyle::ResizeLeftRight)
        .on_drag(track, start)
        .on_drag_move::<BTrack>(over)
}
