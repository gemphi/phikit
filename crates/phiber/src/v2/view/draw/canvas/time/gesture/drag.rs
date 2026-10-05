//! A held button: the drag as it proceeds, the release, and the grab.
//!
//! Split from `mod.rs` for the 240-line read budget (`level_structure`), and it
//! is the right seam: `mod.rs` is what a PRESS decides — which of the three
//! things one button means in an animation window — and this is what happens
//! for as long as the button stays down.

use super::{BTimeDrag, BTimeGesture, PRESSED};
use super::super::row::BTimeRow;
use gpui::{App, Pixels, Point, Window};

impl BTimeGesture {
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
                let to = BTimeRow::index_at(y, self.view.row_height, self.view.scrub_height).unwrap_or(0).min(self.view.rows.len().saturating_sub(1));
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
