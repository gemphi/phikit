//! The animation window — Blender's Dope Sheet in Timeline and Action mode
//! (`space_action.cc::action_main_region_draw`) and the Graph editor
//! (`space_graph.cc::graph_main_region_draw`), which share one `View2D`
//! over frames, one scrub strip and one playhead.
//!
//! Structure: the element that owns the View2D ([`BTime`]) and the scrub.
//! The view itself is `view/`, painting is `set.rs`; the grid maths
//! `grid/`, the rows `row/`, the F-curves `curve/`, the NLA strips
//! `strip/`, the drivers `driver/`, the markers `marker/`.

pub mod curve;
pub mod driver;
pub mod gesture;
pub mod grid;
pub mod marker;
pub mod row;
pub mod set;
pub mod sheet;
pub mod show;
pub mod expander;
pub mod strip;
pub mod view;

pub use view::BTimeView;

use super::super::ctx::BRenderCtx;
use crate::v2::host::BHostType;
use super::super::flex::{BFill, BFlex, BPlace};
use gpui::prelude::*;
use gpui::{AnyElement, Bounds, Context, Pixels, Render, SharedString, Window, canvas, div};
use std::cell::Cell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

/// `UI_TIME_SCRUB_MARGIN_Y` at scale 1: the frame-number strip's height.
pub const SCRUB_HEIGHT: f32 = 23.0;
/// `U.widget_unit`.
pub const WIDGET_UNIT: f32 = 20.0;
/// `U.v2d_min_gridsize`: the fewest pixels between major grid lines.
pub const MIN_GRID: f32 = 45.0;
/// `action_create`: the view opens ten frames either side of the range.
pub const VIEW_MARGIN: f32 = 10.0;

struct BGhost;

impl Render for BGhost {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

/// The animation window of `host`: the Timeline, the Dope Sheet with its
/// channel region, the Graph editor with its curves, the NLA editor with
/// its tracks, or the Drivers editor with its driver curves.
pub struct BTime(pub BHostType);

impl BTime {
    pub fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        let colors = ctx.style.anim();
        let mut view = BTimeView::now(ctx.state, self.0, curve::BAxisColors::of(&colors));
        // The one place the theme meets this window's geometry: `now` builds
        // from the scene and Blender's own constants and has no style to read.
        view.scrub_height = ctx.style.dimensions.header_height;
        // The paint pass knows the canvas's bounds; the gestures, which need
        // them to turn a pointer into a frame, read them from here.
        let bounds: Rc<Cell<Option<Bounds<Pixels>>>> = Rc::new(Cell::new(None));
        let seen = Rc::clone(&bounds);
        let painted = view.clone();
        let playing = crate::v2::part::playback::set::BTransportButton::playing(ctx.state);
        // A grab is live while either a marker or a key is being moved.
        // `BTimeGesture::grab_move` and `grab_cancel` both branch on
        // `view.key_grab` and then fall through to the marker — but the
        // pointer handlers below were attached only when a **marker** grab
        // was active, so in a key grab nothing ever delivered the move and
        // G left the key where it was (`dope_sheet_edits_keys`: "G: the key
        // follows the pointer").
        let grabbing = crate::v2::scene::grab::BMarkerGrab::active(ctx.state) || view.key_grab;
        let gesture = Rc::new(gesture::BTimeGesture { view: view.clone(), handler: ctx.handler.clone(), bounds, grabbing, snap: (crate::v2::part::playback::snap::BPlayheadSnap::now(ctx.state), crate::v2::part::playback::snap::BSnapTargets::now(ctx.state)) });
        let (down, moved, dragged, up, cancel, wheeled) =
            (gesture.clone(), gesture.clone(), gesture.clone(), gesture.clone(), gesture.clone(), gesture);
        let host = self.0;
        // The window covers the host edge to edge (`BPlace::Overlay`) in
        // the theme's `TH_BACK`, and the canvas covers the window.
        let window = BFlex::column().place(BPlace::Overlay).fill(BFill::Swatch(crate::v2::theme::BAnimColors::rgba(colors.back)));
        let painter = canvas(|_, _, _| (), move |b, (), window, cx| {
            seen.set(Some(b));
            if painted.channel_width > 0.0 {
                row::set::paint_channels(&painted.rows, painted.row_height, painted.scrub_height, &colors, b, window, cx);
            }
            painted.paint(&colors, painted.window(b), window, cx);
            if playing {
                window.request_animation_frame();
            }
        });
        super::base(ctx)
            .id(SharedString::from("time-window"))
            .debug_selector(|| "time-window".to_string())
            .child(window.div(ctx).child(BFlex::column().place(BPlace::Overlay).styled(painter, ctx)))
            // The `View2D` keymap over the time axis: a notch zooms, Ctrl
            // scrolls. `BTimeGesture::wheel` and the `wheel_op` behind it
            // were written and unit-tested, and nothing ever called them —
            // every other canvas (view3d, image, the node wire) wires its
            // own, so the animation editors alone did not answer the wheel.
            .on_scroll_wheel(move |event: &gpui::ScrollWheelEvent, window, app| wheeled.wheel(host, event, window, app))
            .on_mouse_down(gpui::MouseButton::Left, move |event, window, app| down.down(event, window, app))
            .on_mouse_up(gpui::MouseButton::Left, { let up = up.clone(); move |_, window, app| up.up(window, app) })
            .on_mouse_up_out(gpui::MouseButton::Left, move |_, window, app| up.up(window, app))
            .when(grabbing, |element| {
                element
                    .on_mouse_move(move |event: &gpui::MouseMoveEvent, window, app| moved.grab_move(event.position, window, app))
                    .on_mouse_down(gpui::MouseButton::Right, move |_, window, app| cancel.grab_cancel(window, app))
            })
            .on_drag(Arc::new(Mutex::new(gesture::BTimeDrag::default())), move |state, _, _, cx| {
                if let Ok(mut state) = state.lock() {
                    *state = gesture::BTimeGesture::drag_start();
                }
                cx.new(|_| BGhost)
            })
            .on_drag_move::<Arc<Mutex<gesture::BTimeDrag>>>(move |event, window, app| {
                let state = Arc::clone(event.drag(app));
                let now = state.lock().map(|s| *s).unwrap_or_default();
                let next = dragged.drag_move(now, window.mouse_position(), window, app);
                if let Ok(mut s) = state.lock() {
                    *s = next;
                }
            })
            .into_any_element()
    }
}
