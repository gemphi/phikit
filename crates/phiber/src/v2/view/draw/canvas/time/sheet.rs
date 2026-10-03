//! The Dope Sheet's part of the animation window's gestures
//! (`gesture.rs`): a press among its keys picks, and says what a drag from
//! there does — move the picked keys, or sweep a box
//! (`scene::action::dope`).

use super::gesture::{BTimeDrag, BTimeGesture};
use super::row::BTimeRow;
use crate::v2::state::{BKey as _, OpKey};
use crate::v2::view::prim::BAction;
use gpui::{App, Window};

impl BTimeGesture {
    pub(super) fn keys(&self, op: String, window: &mut Window, app: &mut App) {
        (self.handler)(&BAction::Set(OpKey::Keys.path(), op), window, app);
    }

    /// The Dope Sheet's press below the scrub band: the drag it begins,
    /// having picked (`dope::BKeyOpRule`). `None` for the scrub and marker
    /// bands, and outside the Dope Sheet.
    pub(super) fn sheet_press(&self, event: &gpui::MouseDownEvent, window: &mut Window, app: &mut App) -> Option<BTimeDrag> {
        let v = &self.view;
        if v.channel_width <= 0.0 || v.values.is_some() || v.nla {
            return None;
        }
        let b = self.bounds.get()?;
        let (x, y) = (f32::from(event.position.x - b.origin.x) - v.channel_width, f32::from(event.position.y - b.origin.y));
        if y < super::SCRUB_HEIGHT || v.in_marker_band(y, f32::from(b.size.height)) {
            return None;
        }
        if x < 0.0 || v.rows.is_empty() {
            return Some(BTimeDrag::Held);
        }
        let width = f32::from(v.window(b).size.width);
        let row = BTimeRow::index_at(y, v.row_height).unwrap_or(0).min(v.rows.len() - 1);
        let hit = v.rows[row].keys.iter().copied().filter(|&k| (v.x_of(k, width) - x).abs() <= 7.0).min_by(|a, b| (v.x_of(*a, width) - x).abs().total_cmp(&(v.x_of(*b, width) - x).abs()));
        let shift = event.modifiers.shift;
        Some(match hit {
            Some(frame) => {
                if shift {
                    self.keys(format!("click+:{row}:{frame}"), window, app);
                } else if !v.picked(row, frame) {
                    self.keys(format!("click:{row}:{frame}"), window, app);
                }
                BTimeDrag::Keys(frame.round() as i32, false)
            }
            None => {
                if !shift {
                    self.keys("click:none".into(), window, app);
                }
                BTimeDrag::KeyBox((v.frame_at(x, width) * 1000.0).round() as i32, row)
            }
        })
    }
}
