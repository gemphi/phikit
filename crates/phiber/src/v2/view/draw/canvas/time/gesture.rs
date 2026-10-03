//! What the pointer does in an animation window — one owner for every
//! gesture there: a press scrubs (`anim.change_frame`); in the marker band a
//! press selects the marker under it (Shift extends) and a drag begun on one
//! moves it (`marker.move`), while a drag begun on the band's empty part
//! sweeps a box and selects the markers inside it (`marker.select_box`);
//! and while a marker grab (G) is on, the pointer carries the markers and a
//! click keeps them. In the NLA editor a press below the scrub band picks
//! instead (`nla.click_select`: the strip there, or nothing; on a track's
//! name, `anim.channels_click`). In the Dope Sheet a press there picks the
//! key column under it, a drag begun on a key moves the picked keys, and a
//! drag begun on nothing sweeps a box (`scene::action::dope`); only the
//! scrub band scrubs, as in Blender. A scrub snaps the playhead when the
//! magnet says so. The wheel is the `View2D` keymap: a notch zooms the time
//! axis (`view2d.zoom_in` / `zoom_out`), with Ctrl it scrolls it
//! (`view2d.scroll_left` / `scroll_right`); Shift's channel scroll is not
//! here yet (`view/op.rs`).

use super::BTimeView;
use super::row::{BRowKind, BTimeRow};
use crate::v2::scene::action::nla_editor::BNlaOp;
use super::super::super::ctx::BActionHandler;
use crate::v2::part::playback::snap::{BPlayheadSnap, BSnapTargets};
use crate::v2::state::{BKey as _, CtxKey, OpKey};
use crate::v2::view::prim::BAction;
use gpui::{App, Bounds, Pixels, Point, Window};
use std::cell::Cell;
use std::rc::Rc;

/// What a drag began on: a marker (its frame, as it moves), the band's
/// empty part (the frame the box began at), or neither — a scrub.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BTimeDrag {
    #[default]
    Scrub,
    Marker(i32),
    Box(i32),
    /// A pick in the NLA editor: a drag does nothing more.
    Held,
    /// A drag begun on a Dope Sheet key at a frame; whether its move began.
    Keys(i32, bool),
    /// A drag begun on the Dope Sheet's empty keys: the frame (in
    /// thousandths) and the row it began at.
    KeyBox(i32, usize),
}

thread_local! {
    /// What the last press landed on. The press's own action re-renders the
    /// window — a new element, new closures — before the drag begins, so
    /// this outlives the element that set it.
    static PRESSED: Cell<BTimeDrag> = const { Cell::new(BTimeDrag::Scrub) };
}

/// The window's gesture handling, for one render of it.
pub struct BTimeGesture {
    pub view: BTimeView,
    pub handler: BActionHandler,
    /// The canvas's bounds, set by the paint pass.
    pub bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
    /// A marker grab (G) is under way.
    pub grabbing: bool,
    /// Playhead snapping and what it can snap to (`playback::snap`).
    pub snap: (BPlayheadSnap, BSnapTargets),
}

impl BTimeGesture {
    /// The wheel over `host`'s window: the `View2D` keymap (`view/op.rs`).
    pub fn wheel(&self, host: crate::v2::host::BHostType, event: &gpui::ScrollWheelEvent, window: &mut Window, app: &mut App) {
        let notches = match event.delta {
            gpui::ScrollDelta::Lines(p) => p.y,
            gpui::ScrollDelta::Pixels(p) => f32::from(p.y) / 20.0,
        };
        if let Some(op) = super::view::op::wheel_op(host, notches, event.modifiers.control, event.modifiers.shift) {
            (self.handler)(&BAction::Set(OpKey::TimeView.path(), op), window, app);
        }
    }

    fn marker(&self, op: String, window: &mut Window, app: &mut App) {
        (self.handler)(&BAction::Set(OpKey::Marker.path(), op), window, app);
    }

    /// The pointer in the window's pixels: `(x, y, width, height)`.
    fn locate(&self, position: Point<Pixels>) -> Option<(f32, f32, f32, f32)> {
        let b = self.bounds.get().map(|b| self.view.window(b))?;
        let x = f32::from(position.x) - f32::from(b.origin.x);
        (x >= 0.0).then_some((x, f32::from(position.y) - f32::from(b.origin.y), f32::from(b.size.width), f32::from(b.size.height)))
    }

    /// The frame under the pointer.
    fn frame(&self, position: Point<Pixels>) -> Option<i32> {
        self.locate(position).map(|(x, _, width, _)| self.view.frame_at(x, width).round() as i32)
    }

    fn scrub(&self, position: Point<Pixels>, window: &mut Window, app: &mut App) {
        // "Driver Editor's X axis is not time": nothing to scrub.
        if self.view.drivers {
            return;
        }
        let Some((x, _, width, _)) = self.locate(position) else { return };
        let mut frame = self.view.frame_at(x, width);
        // The magnet, flipped while Ctrl is held; its reach is pixels at
        // the window's zoom.
        let (snap, targets) = &self.snap;
        if snap.active(window.modifiers().control) {
            frame = snap.snap(frame, snap.distance as f32 / self.view.scale(width).max(1e-3), targets);
        }
        (self.handler)(&BAction::Set(CtxKey::Frame.path(), (frame.round() as i32).to_string()), window, app);
    }

    /// The NLA editor's press: on a track's name, that track; in the
    /// strips, the strip within 7 px (`nlaedit_strip_at_region_position`)
    /// or nothing; `""` for a name that is no track's. `None` for the scrub
    /// and marker bands.
    fn nla_press(&self, event: &gpui::MouseDownEvent) -> Option<String> {
        let b = self.bounds.get()?;
        let (x, y) = (f32::from(event.position.x - b.origin.x), f32::from(event.position.y - b.origin.y));
        if y < super::SCRUB_HEIGHT || self.view.in_marker_band(y, f32::from(b.size.height)) {
            return None;
        }
        let extend = event.modifiers.shift;
        let hit = BTimeRow::index_at(y, self.view.row_height).and_then(|i| Some((i, BTimeRow::track_at(&self.view.rows, i)?)));
        let x = x - self.view.channel_width;
        if x < 0.0 {
            return Some(hit.map_or(String::new(), |(_, (object, track))| BNlaOp::channel(&object, track, extend)));
        }
        let width = f32::from(self.view.window(b).size.width);
        let strip = hit.and_then(|(i, (object, track))| {
            let BRowKind::Track(row) = &self.view.rows[i].kind else { return None };
            let reach = (self.view.frame_at(x - 7.0, width), self.view.frame_at(x + 7.0, width));
            Some((object, track, row.strip_at(self.view.frame_at(x, width), reach)?))
        });
        Some(BNlaOp::click(strip.as_ref().map(|(o, t, k)| (o.as_str(), *t, *k)), extend))
    }

    pub fn down(&self, event: &gpui::MouseDownEvent, window: &mut Window, app: &mut App) {
        PRESSED.set(BTimeDrag::Scrub);
        if self.view.key_grab {
            return self.keys("grab:confirm".into(), window, app);
        }
        if self.grabbing {
            return self.marker("grab:confirm".into(), window, app);
        }
        if let Some(drag) = self.sheet_press(event, window, app) {
            return PRESSED.set(drag);
        }
        if let Some(op) = self.view.nla.then(|| self.nla_press(event)).flatten() {
            PRESSED.set(BTimeDrag::Held);
            if !op.is_empty() {
                (self.handler)(&BAction::Set(OpKey::Nla.path(), op), window, app);
            }
            return;
        }
        let Some((x, y, width, height)) = self.locate(event.position) else { return self.scrub(event.position, window, app) };
        if !self.view.in_marker_band(y, height) {
            return self.scrub(event.position, window, app);
        }
        let hit = self.view.marker_hit(x, y, width, height);
        let at = self.view.frame_at(x, width).round() as i32;
        PRESSED.set(hit.as_ref().map_or(BTimeDrag::Box(at), |hit| BTimeDrag::Marker(hit.frame as i32)));
        let op = match hit {
            Some(hit) if event.modifiers.shift => format!("select+:{}", hit.frame as i64),
            Some(hit) => format!("select:{}", hit.frame as i64),
            None if event.modifiers.shift => return,
            None => "select:none".to_string(),
        };
        self.marker(op, window, app);
    }

    /// A drag begins: on what the press landed on.
    pub fn drag_start() -> BTimeDrag {
        PRESSED.replace(BTimeDrag::Scrub)
    }

    /// The pointer moved with the button held: the drag as it now stands.
    pub fn drag_move(&self, drag: BTimeDrag, position: Point<Pixels>, window: &mut Window, app: &mut App) -> BTimeDrag {
        match (drag, self.frame(position)) {
            (BTimeDrag::Scrub, _) => {
                self.scrub(position, window, app);
                drag
            }
            (BTimeDrag::Marker(from), Some(to)) if to != from => {
                self.marker(format!("move:{from}:{to}"), window, app);
                BTimeDrag::Marker(to)
            }
            (BTimeDrag::Box(from), Some(to)) => {
                self.marker(format!("box:{from}:{to}"), window, app);
                drag
            }
            (BTimeDrag::Keys(from, started), Some(to)) => {
                if !started {
                    self.keys(format!("grab:start:{from}"), window, app);
                }
                self.keys(format!("grab:at:{to}"), window, app);
                BTimeDrag::Keys(from, true)
            }
            (BTimeDrag::KeyBox(from, row), _) => {
                let Some((x, y, width, _)) = self.locate(position) else { return drag };
                let to = BTimeRow::index_at(y, self.view.row_height).unwrap_or(0).min(self.view.rows.len().saturating_sub(1));
                self.keys(format!("box:{},{},{row},{to}", from as f32 / 1000.0, self.view.frame_at(x, width)), window, app);
                drag
            }
            (BTimeDrag::Held, _) => drag,
            _ => drag,
        }
    }

    /// The button came up: a box ends (the markers in it stay selected).
    pub fn up(&self, window: &mut Window, app: &mut App) {
        if self.view.key_box.is_some() {
            self.keys("box:end".into(), window, app);
        }
        if self.view.key_grab {
            self.keys("grab:confirm".into(), window, app);
        }
        if self.view.marker_box.is_some() {
            self.marker("box:end".into(), window, app);
        }
    }

    /// While grabbing: the markers follow the pointer.
    pub fn grab_move(&self, position: Point<Pixels>, window: &mut Window, app: &mut App) {
        if let (true, Some(at)) = (self.view.key_grab, self.frame(position)) {
            return self.keys(format!("grab:at:{at}"), window, app);
        }
        if let Some(at) = self.frame(position) {
            self.marker(format!("grab:at:{at}"), window, app);
        }
    }

    pub fn grab_cancel(&self, window: &mut Window, app: &mut App) {
        if self.view.key_grab {
            return self.keys("grab:cancel".into(), window, app);
        }
        self.marker("grab:cancel".into(), window, app);
    }
}
