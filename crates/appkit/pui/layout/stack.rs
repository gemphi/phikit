//! A stack: groups one under another, each on a section's ground under
//! its heading, and the room a pane has left shared out between them so
//! they fill it. And tabs over a stack: the tabs above, on the pane, and
//! the groups they choose on one ground of their own under them, so the
//! tabs stand over what they choose and do not sit in it; the light on
//! the chosen tab moves to the next one chosen.
//!
//! The room round every group, its roundness and the least room between
//! two are the theme's, the same for every app (`section.*`).

use super::section::{ROUND, SIDE, TOP, lit};
use super::PDry;
use crate::pui::block::{PBand, PTabMove};
use crate::pui::core::light::label;
use crate::pui::core::{PLight, PPaint, PPoint, PRect};

/// The theme's tokens: the least room between two groups, a group's
/// heading's ink, and the room under a heading.
pub const SPACE: &str = "section.space";
/// Whether only the grounds are drawn, what stands on them laid out but
/// not drawn: to see the grounds alone (more than nothing says so).
pub const BARE: &str = "section.bare";

/// Whether a theme has only the grounds drawn.
pub fn bare(paint: &dyn PPaint) -> bool {
    paint.theme().size(BARE).unwrap_or_default() > 0.0
}
pub const INK: &str = "text.ink";
pub const LINE: &str = "text.line";
pub const GAP: &str = "gap";

/// What a group draws under its heading, from a place across a width,
/// lit from the right where told (against its ground): where it ends,
/// and what can be pressed in it.
pub type PDrawn<'a> = Box<dyn FnMut(&mut dyn PPaint, PPoint, f32, bool) -> (f32, Vec<PRect>) + 'a>;

/// A group of a stack: its heading (none for none), and what it draws.
pub struct PGrouped<'a> {
    pub head: String,
    pub draw: PDrawn<'a>,
}

/// The room beside and over what stands on a group's ground.
fn room(paint: &dyn PPaint) -> (f32, f32) {
    let gap = paint.size(GAP);
    (paint.theme().size(SIDE).unwrap_or(gap / 2.0), paint.theme().size(TOP).unwrap_or(gap / 4.0))
}

/// Draw the groups from a place across a width (their grounds' edges),
/// sharing out the room up to `end` between them, their grounds lit
/// from the right (`right`) or the left, and what stands on them the
/// other way. Where the last ends, and what can be pressed in each.
pub fn stack(paint: &mut dyn PPaint, at: PPoint, wide: f32, end: f32, right: bool, group: &mut [PGrouped]) -> (f32, Vec<Vec<PRect>>) {
    let (side, top) = room(paint);
    let (line, gap, ink) = (paint.size(LINE), paint.size(GAP), paint.color(INK));
    let least = paint.theme().size(SPACE).unwrap_or(gap / 2.0);
    let inner = wide - side * 2.0;
    let head = |g: &PGrouped| match g.head.is_empty() {
        true => 0.0,
        false => line + gap / 2.0,
    };
    let tall: Vec<f32> = group.iter_mut().map(|g| head(g) + (g.draw)(&mut PDry(&*paint), PPoint::new(0.0, 0.0), inner, !right).0).collect();
    let total = tall.iter().sum::<f32>() + tall.len() as f32 * top * 2.0 + tall.len().saturating_sub(1) as f32 * least;
    let space = least + (end - at.y - total).max(0.0) / tall.len().saturating_sub(1).max(1) as f32;
    let (mut y, mut pressed) = (at.y, Vec::new());
    for (g, high) in group.iter_mut().zip(tall) {
        let from = PPoint::new(at.x + side, y + top);
        lit(paint, from, inner, from.y + high, right);
        let below = head(g);
        let place = PPoint::new(from.x, from.y + below);
        let drawn = match bare(paint) {
            true => (g.draw)(&mut PDry(&*paint), place, inner, !right).1,
            false => {
                (below > 0.0).then(|| paint.text(from, &g.head, ink));
                (g.draw)(paint, place, inner, !right).1
            }
        };
        pressed.push(drawn);
        y = from.y + high + top + space;
    }
    (y - space, pressed)
}

/// Tabs over a stack, from a place across a width to `end` (further,
/// where they need more): the tabs at the top, and under them, a little
/// apart, one ground holding the groups they choose. The tabs and the
/// ground are lit from the right, against the column they stand in; the
/// groups from the left, against the ground. What can be pressed: the tabs, and in each group; and
/// where the ground ends.
pub fn tabbed<B: PBand + ?Sized>(paint: &mut dyn PPaint, at: PPoint, wide: f32, end: f32, tab: &B, motion: &PTabMove, group: &mut [PGrouped]) -> (Vec<PRect>, Vec<Vec<PRect>>, f32) {
    let (side, top) = room(paint);
    let radius = paint.theme().size(ROUND).unwrap_or_default() + side;
    let least = paint.theme().size(SPACE).unwrap_or(paint.size(GAP) / 2.0);
    let (from, to, share) = motion.now();
    let tabs = match bare(paint) {
        true => tab.draw_moving(&mut PDry(&*paint), at, wide, from, to, share),
        false => tab.draw_moving(paint, at, wide, from, to, share),
    };
    let held = at.y + tab.high(paint) + least;
    let inner = wide - side * 2.0;
    let need = stack(&mut PDry(&*paint), PPoint::new(at.x + side, held + top), inner, 0.0, false, group).0;
    let bottom = end.max(need + top);
    label(paint, PRect::new(at.x, held, wide, bottom - held), radius, PLight::Rest, true, "");
    let (_, pressed) = stack(paint, PPoint::new(at.x + side, held + top), inner, bottom - top, false, group);
    (tabs, pressed, bottom)
}
