//! Which socket the cursor is on.
//!
//! Ported from Blender 5.2.0, `editors/space_node/node_edit.cc:791`,
//! `node_find_indicated_socket`.
//!
//! ```text
//! view2d_scale = view2d_scale_get_x(&region.v2d)
//! max_distance = NODE_SOCKSIZE + clamp(20.0 / view2d_scale, 5.0, 30.0)
//! ```
//!
//! A socket's own radius plus a slack, and the slack is the interesting half.
//! The `20.0 / view2d_scale` converts twenty *screen* pixels into view units, so
//! that the grab area stays the same size under the pointer however far you have
//! zoomed out — and it is clamped so that at extreme zooms it stops growing or
//! shrinking without bound. Twenty screen pixels at Blender's default scale is
//! one `U.widget_unit`, which is why the slack is expressed here as **one unit**
//! and the clamp in units too: phiber's canvas has no zoom yet, so the scale is
//! 1 and the division is the identity — but writing it in units means the day
//! the canvas zooms, the one line that changes is the scale.
//!
//! Three rules beyond the distance, all from the same function:
//!
//! * **Nearest wins.** Blender keeps a `best_distance` rather than taking the
//!   first hit, so between two sockets a cursor lands between, it picks the
//!   closer one.
//! * **Not over the header.** `draw_bounds.ymax - cursor.y < NODE_DY` skips a
//!   node's sockets entirely while the cursor is within one row of its top, with
//!   the reason stated: *"This allows the user to always resize by dragging on
//!   the left and right side of the header."* An input socket sits ON the card's
//!   left edge, so without this the top-most input would swallow every grab
//!   aimed at the header's corner.
//! * **Not occluded.** `socket_is_occluded` drops a socket that another card is
//!   drawn over, walking `tree_draw_order_calc_nodes_reversed` — the same order
//!   `draw_order` produces, reversed — so the card on top is the one that
//!   answers. Without it, dragging a card over another lets you grab sockets
//!   through it.
//!
//! Collapsed nodes ARE handled now: `node_update_collapsed` lays their sockets
//! out (`socket/collapsed.rs`), and the header exclusion below is skipped for
//! them exactly as Blender skips it (`!node_collapsed`). What is still not here
//! is the multi-input hit rectangle (`cursor_isect_multi_input_socket`, a wide
//! box rather than a radius): phiber draws no multi-input sockets yet.

use super::paint::BNodeCard;
use super::socket::{BSocketRow, BSocketSide, DOT, ROW};

/// `clamp(20.0 / view2d_scale, 5.0, 30.0)`, in widget units rather than pixels.
///
/// Twenty screen pixels is one widget unit at Blender's default UI scale, and
/// its five-to-thirty clamp is a quarter of a unit to one and a half.
const SLACK: f32 = 1.0;
const SLACK_MIN: f32 = 0.25;
const SLACK_MAX: f32 = 1.5;

/// How far from a socket's centre the cursor still counts as on it, in pixels.
///
/// `scale` is the canvas zoom; phiber has none, so callers pass 1.
pub fn max_distance(unit: f32, scale: f32) -> f32 {
    let slack = (SLACK / scale).clamp(SLACK_MIN, SLACK_MAX);
    (DOT + slack) * unit
}

/// The socket under `cursor`, if any — card index and the row.
///
/// `cards` is in draw order, back to front, which is what `draw_order` gives and
/// what occlusion needs. `want` filters to one side: `node_link_init` searches
/// for an output first and then an input, so a cursor near both picks the
/// output, and `node_link_find_socket` searches only for the side opposite the
/// one being dragged from.
pub fn socket_at(
    cards: &[BNodeCard],
    cursor: (f32, f32),
    want: Option<BSocketSide>,
    scale: f32,
) -> Option<(usize, BSocketRow)> {
    let mut best: Option<(usize, BSocketRow, f32)> = None;
    for (index, card) in cards.iter().enumerate() {
        // "Don't pick socket when cursor is over node header."
        //
        // Blender's test is `draw_bounds.ymax - cursor.y < NODE_DY`, and
        // `ymax` is the card's TOP because Blender's Y grows upward. The
        // canvas here grows downward, so the top is `card.y` and the
        // subtraction flips. Ported the other way round it reads perfectly
        // well and guards the card's BOTTOM edge, which is why this carries
        // the axis note: the formula alone is not the fact.
        // `if (!node->is_reroute() && !node_collapsed && draw_bounds.ymax -
        // cursor.y < NODE_DY) continue;` — the exclusion is skipped for a
        // collapsed node, which has no header to resize by. `header` is zero on
        // a collapsed card, which is how that is spelt here.
        if card.header > 0.0 && cursor.1 - card.y < ROW * card.unit {
            continue;
        }
        // A card drawn later covers this one, so its sockets do not answer.
        let occluded = cards[index + 1..].iter().any(|over| {
            cursor.0 >= over.x
                && cursor.0 <= over.x + over.width
                && cursor.1 >= over.y
                && cursor.1 <= over.y + over.height
        });
        if occluded {
            continue;
        }
        let reach = max_distance(card.unit, scale);
        for row in &card.sockets {
            if want.is_some_and(|side| side != row.side) {
                continue;
            }
            let (sx, sy) = card.socket_at(row);
            let distance = ((sx - cursor.0).powi(2) + (sy - cursor.1).powi(2)).sqrt();
            if distance >= reach {
                continue;
            }
            // Nearest wins, not first found.
            if best.as_ref().is_none_or(|(_, _, d)| distance < *d) {
                best = Some((index, row.clone(), distance));
            }
        }
    }
    best.map(|(index, row, _)| (index, row))
}

#[cfg(test)]
mod tests;
