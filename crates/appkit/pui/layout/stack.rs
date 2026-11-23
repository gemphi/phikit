//! A stack: groups one under another, each on a section's ground under
//! its heading, and the room a pane has left shared out between them so
//! they fill it. And tabs over a stack: the tabs on one ground with the
//! groups they choose, so it is plain that they choose what is under
//! them, the light on the chosen tab moving to the next one chosen.
//!
//! The room round every group, its roundness and the least room between
//! two are the theme's, the same for every app (`section.*`).

use super::section::{ROUND, SIDE, TOP, ground};
use super::PDry;
use crate::pui::block::{PBand, PTabMove};
use crate::pui::core::light::label;
use crate::pui::core::{PLight, PPaint, PPoint, PRect};

/// The theme's tokens: the least room between two groups, a group's
/// heading's ink, and the room under a heading.
pub const SPACE: &str = "section.space";
pub const INK: &str = "text.ink";
pub const LINE: &str = "text.line";
pub const GAP: &str = "gap";

/// What a group draws under its heading, from a place across a width:
/// where it ends, and what can be pressed in it.
pub type PDrawn<'a> = Box<dyn FnMut(&mut dyn PPaint, PPoint, f32) -> (f32, Vec<PRect>) + 'a>;

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
/// sharing out the room up to `end` between them. Where the last ends,
/// and what can be pressed in each.
pub fn stack(paint: &mut dyn PPaint, at: PPoint, wide: f32, end: f32, group: &mut [PGrouped]) -> (f32, Vec<Vec<PRect>>) {
    let (side, top) = room(paint);
    let (line, gap, ink) = (paint.size(LINE), paint.size(GAP), paint.color(INK));
    let least = paint.theme().size(SPACE).unwrap_or(gap / 2.0);
    let inner = wide - side * 2.0;
    let head = |g: &PGrouped| match g.head.is_empty() {
        true => 0.0,
        false => line + gap / 2.0,
    };
    let tall: Vec<f32> = group.iter_mut().map(|g| head(g) + (g.draw)(&mut PDry(&*paint), PPoint::new(0.0, 0.0), inner).0).collect();
    let total = tall.iter().sum::<f32>() + tall.len() as f32 * top * 2.0 + tall.len().saturating_sub(1) as f32 * least;
    let space = least + (end - at.y - total).max(0.0) / tall.len().saturating_sub(1).max(1) as f32;
    let (mut y, mut pressed) = (at.y, Vec::new());
    for (g, high) in group.iter_mut().zip(tall) {
        let from = PPoint::new(at.x + side, y + top);
        ground(paint, from, inner, from.y + high);
        let below = head(g);
        (below > 0.0).then(|| paint.text(from, &g.head, ink));
        pressed.push((g.draw)(paint, PPoint::new(from.x, from.y + below), inner).1);
        y = from.y + high + top + space;
    }
    (y - space, pressed)
}

/// Tabs over a stack, all on one ground from a place across a width to
/// `end` (further, where they need more). The ground is lit from the
/// left, the groups' from the right, so each stands against what holds
/// it. What can be pressed: the tabs, and in each group.
pub fn tabbed<B: PBand + ?Sized>(paint: &mut dyn PPaint, at: PPoint, wide: f32, end: f32, tab: &B, motion: &PTabMove, group: &mut [PGrouped]) -> (Vec<PRect>, Vec<Vec<PRect>>) {
    let (side, top) = room(paint);
    let radius = paint.theme().size(ROUND).unwrap_or_default() + side;
    let least = paint.theme().size(SPACE).unwrap_or(paint.size(GAP) / 2.0);
    let inner = wide - side * 2.0;
    let below = at.y + top + tab.high(paint) + least;
    let need = stack(&mut PDry(&*paint), PPoint::new(at.x + side, below), inner, 0.0, group).0;
    let bottom = end.max(need + top);
    label(paint, PRect::new(at.x, at.y, wide, bottom - at.y), radius, PLight::Rest, false, "");
    let (from, to, share) = motion.now();
    let tabs = tab.draw_moving(paint, PPoint::new(at.x + side, at.y + top), inner, from, to, share);
    let (_, pressed) = stack(paint, PPoint::new(at.x + side, below), inner, bottom - top, group);
    (tabs, pressed)
}
