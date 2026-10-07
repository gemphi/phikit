//! The sockets and the links, painted.
//!
//! Cards are divs — they are rectangles with text in them. Links are not: a
//! Blender link is a cubic bezier, and no amount of `div()` draws a curve. So
//! everything between the cards is painted, which is also how the animation
//! editors draw their curves (`canvas/time/set.rs`).
//!
//! **The curve**, ported from Blender 5.2.0,
//! `editors/space_node/drawnode.cc:1749`, `calculate_inner_link_bezier_points`:
//!
//! ```text
//! dist_x       = |p0.x - p3.x|
//! dist_y       = |p0.y - p3.y|
//! slope        = dist_y / dist_x            (safe divide: 0 when dist_x is 0)
//! clamp_factor = min(1, slope * (4.5 - 0.25 * curving))
//! handle       = curving * 0.1 * dist_x * clamp_factor
//! p1 = (p0.x + handle, p0.y)
//! p2 = (p3.x - handle, p3.y)
//! ```
//!
//! `curving` is `TH_NODE_CURVING`, which the factory theme sets to **4**
//! (`userdef_default_theme.c:670`, `.noodle_curving = 4`). At 4 the bracket is
//! `3.5`, so the handles reach four tenths of the horizontal distance — but only
//! for a steep link. The `clamp_factor` is the detail worth keeping: Blender's own
//! comment is "Reduce the handle offset when the link endpoints are close to
//! horizontal", and without it two sockets at nearly the same height get a long
//! flat S instead of the almost-straight line Blender draws. A link with no
//! horizontal distance at all gets `slope = 0` out of the safe divide, so its
//! handles collapse and it is drawn as a straight vertical line.
//!
//! `curving == 0` is Blender's straight-line case, where the handles sit at the
//! one-third and two-third interpolations so the cubic degenerates to a line.
//! [`handles`] keeps that branch because the curving value is a theme setting
//! that phiber will expose, and a reader comparing the two files should find both
//! halves of the condition.
//!
//! **The two strokes.** A Blender link is drawn twice, in one instanced draw
//! whose vertices are split in half (`gpu_shader_2D_nodelink.bsl.hh:83`, "Each
//! instance contains both the outline and the main line on top"):
//!
//! * the **outline** pass at the full `LINK_WIDTH` (2.5) in `color_shadow`,
//!   which is `TH_WIRE` — `0x1a1a1a` in the factory theme, the same value as the
//!   node editor's own `back`. An outline the colour of the canvas is what makes
//!   a noodle read as cut out of the background rather than laid on it;
//! * the **main line** on top at `line_thickness *= 0.65f` (line 120) in
//!   `TH_WIRE_INNER`.
//!
//! Still missing: the colour gradient along the link. The main pass mixes
//! `color_start` into `color_end` between uv 0.35 and 0.65
//! (`start_gradient_threshold` / `end_gradient_threshold`), which only shows
//! when a link's two ends differ — a link out of a selected node. Both ends are
//! one colour here. Also missing: `has_back_link`'s `*= 1.7f` widening for two
//! links drawn over each other, and a muted link's `0.65` inner alpha.

use super::socket::{BSocketRow, BSocketSide};
use crate::v2::theme::palette::BColor;
use gpui::{App, Bounds, PathBuilder, Pixels, Window, point, px};

/// `TH_NODE_CURVING`'s factory value (`.noodle_curving = 4`).
pub const CURVING: f32 = 4.0;
/// `LINK_WIDTH` (`drawnode.cc:1959`) — the outline pass's width.
pub const LINK_WIDTH: f32 = 2.5;
/// `line_thickness *= 0.65f` — the main line on top of the outline
/// (`gpu_shader_2D_nodelink.bsl.hh:120`).
pub const INNER_SCALE: f32 = 0.65;

/// The style this pass needs — colours and the one stroke width — resolved by
/// the caller, so no design literal lives in the painter.
#[derive(Clone, Copy, Debug)]
pub struct BNodeInk {
    /// The link. Blender's `TH_WIRE_INNER` — the core of the noodle.
    pub wire: BColor,
    /// The link's outline. Blender's `TH_WIRE`, which the factory theme sets to
    /// the node editor's own background, so a noodle reads as cut out of it.
    pub wire_shadow: BColor,
    /// A socket's label.
    pub label: BColor,
    /// The ring around a socket, so a pale socket reads against a pale card.
    pub outline: BColor,
    /// How thick that ring is — the theme's own border stroke, so a socket's
    /// ring matches every other hairline in the window.
    pub ring: f32,
}

/// One card, placed, in canvas pixels.
#[derive(Clone, Debug)]
pub struct BNodeCard {
    /// Which node this is. Carried rather than left to the caller to work out
    /// from the order the cards were built in: the hit-test has to name the
    /// node it found a socket on, and matching by position would couple it to
    /// `plan`'s iteration order.
    pub name: String,
    /// The card's top-left and its size.
    pub x: f32,
    pub y: f32,
    pub width: f32,
    /// Where the body starts — the header's height, since socket rows are
    /// measured from the bottom of the header.
    pub header: f32,
    /// One widget unit in pixels (the theme's `row_height`), which is what the
    /// layout's units are expressed in.
    pub unit: f32,
    /// The card's full height — header plus `socket::body_height`. Carried
    /// rather than recomputed because hit-testing needs to know what the card
    /// covers (`pick.rs`), not just where its sockets are.
    pub height: f32,
    pub sockets: Vec<BSocketRow>,
}

impl BNodeCard {
    /// Where one socket sits, in canvas pixels.
    ///
    /// `node_update_basis_socket` puts an input at `locx` and an output at
    /// `locx + NODE_WIDTH(node)` — the card's two vertical edges, with the
    /// socket centred ON the edge rather than inset from it. That is why a
    /// socket appears to straddle the border: it does.
    pub fn socket_at(&self, row: &BSocketRow) -> (f32, f32) {
        let x = match row.side {
            BSocketSide::Input => self.x,
            BSocketSide::Output => self.x + self.width,
        };
        (x, self.y + self.header + row.y * self.unit)
    }
}

/// The two control points of a link's cubic, from its ends.
///
/// Pure, so the clamp can be checked against Blender's arithmetic rather than
/// against a screenshot.
pub fn handles(p0: (f32, f32), p3: (f32, f32), curving: f32) -> ((f32, f32), (f32, f32)) {
    if curving == 0.0 {
        // Blender's straight-line branch: the handles at a third and two
        // thirds, which makes the cubic a line.
        let at = |t: f32| (p0.0 + (p3.0 - p0.0) * t, p0.1 + (p3.1 - p0.1) * t);
        return (at(1.0 / 3.0), at(2.0 / 3.0));
    }
    let dist_x = (p0.0 - p3.0).abs();
    let dist_y = (p0.1 - p3.1).abs();
    // `math::safe_divide`: zero rather than infinity, which is what makes a
    // vertical link straight instead of undrawable.
    let slope = if dist_x == 0.0 { 0.0 } else { dist_y / dist_x };
    let clamp_factor = (slope * (4.5 - 0.25 * curving)).min(1.0);
    let handle = curving * 0.1 * dist_x * clamp_factor;
    ((p0.0 + handle, p0.1), (p3.0 - handle, p3.1))
}

/// Paint every link, then every socket over them — a socket sits on a card's
/// edge, so a link drawn after it would cross the dot it ends at.
pub fn paint(
    cards: &[BNodeCard],
    links: &[((f32, f32), (f32, f32))],
    ink: &BNodeInk,
    at: Bounds<Pixels>,
    window: &mut Window,
    cx: &mut App,
) {
    let origin = |x: f32, y: f32| point(at.origin.x + px(x), at.origin.y + px(y));
    // A link being dragged is drawn by exactly the same two strokes as a real
    // one, because in Blender it IS one — `nldrag.links` are `bNodeLink`s and
    // go through `node_draw_link` with the rest. A preview in a different
    // colour or a dashed line would be phiber inventing a distinction.
    for (from, to) in links {
        let (c1, c2) = handles(*from, *to, CURVING);
        // The outline first and the main line over it, which is the order the
        // one instanced draw puts them in.
        for (width, color) in
            [(LINK_WIDTH, ink.wire_shadow), (LINK_WIDTH * INNER_SCALE, ink.wire)]
        {
            let mut path = PathBuilder::stroke(px(width));
            path.move_to(origin(from.0, from.1));
            path.cubic_bezier_to(origin(to.0, to.1), origin(c1.0, c1.1), origin(c2.0, c2.1));
            if let Ok(path) = path.build() {
                window.paint_path(path, crate::v2::view::draw::color::to_hsla(color));
            }
        }
    }
    for card in cards {
        for row in &card.sockets {
            dot::paint_socket(card, row, ink, at, window, cx);
        }
    }
}

/// The socket's own dot and label.
/// The canvas element itself — the one place pixels exist.
pub mod canvas;
mod dot;

#[cfg(test)]
mod tests;
