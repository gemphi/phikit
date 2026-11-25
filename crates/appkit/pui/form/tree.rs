//! Three more kinds of a form's block, for a browser and an inspector:
//!
//! * `tree`: each choice a line, its icon and its words, set in by its
//!   depth, the chosen one lit: a navigator, as Reality Composer Pro's
//!   Project Browser and Blender's file browser have one.
//! * `facts`: each choice a row, its words at the left on a ground of
//!   their own and its note at the right: an item's attributes.
//! * `picture`: the first choice's picture across the whole width,
//!   standing at the top: an item seen in the inspector.

use super::PChoice;
use crate::pui::block::PFacts;
use crate::pui::core::light::label;
use crate::pui::core::{PFit, PLight, PPaint, PPoint, PRect};
use crate::pui::layout::level;

const GAP: &str = "gap";
const LINE: &str = "text.line";
const INK: &str = "text.ink";
const DIM: &str = "text.dim";
const ICON: &str = "icon.size";
const RADIUS: &str = "row.radius";

/// A tree's lines: where it ends, and where each line stands. Each depth
/// is a step in, as wide as an icon; at the step a line's arrow, down
/// where it is open and to the right where it can open. From the middle
/// of an open arrow a thin line runs down the gap at its depth to the
/// foot of the tree, so the depths read as Finder's and Reality
/// Composer Pro's outlines do. The chosen line is lit, and the one under
/// the pointer softly.
pub fn tree(choice: &[PChoice], now: &str, paint: &mut dyn PPaint, at: PPoint, wide: f32, over: Option<PPoint>) -> (f32, Vec<PRect>) {
    let (gap, line, ink, icon, radius) = (paint.size(GAP), paint.size(LINE), paint.color(INK), paint.size(ICON), paint.size(RADIUS));
    let dim = paint.color(DIM);
    let high = line + gap / 2.0;
    let step = icon;
    let arrow = |depth: u32| at.x + gap / 4.0 + depth as f32 * step;
    let row = |index: usize| PRect::new(at.x, at.y + index as f32 * high, wide, high);
    // The lines down the gaps, first, under everything.
    for (index, c) in choice.iter().enumerate().filter(|(_, c)| c.fold == Some(true)) {
        // It runs to the foot of the tree, past everything under it.
        let under = choice.get(index + 1).is_some_and(|one| one.depth > c.depth);
        if under {
            level(paint, arrow(c.depth) + step / 2.0, row(index).center().y + step / 2.0, at.y + choice.len() as f32 * high);
        }
    }
    let mut rect = Vec::new();
    for (index, c) in choice.iter().enumerate() {
        let r = row(index);
        // The chosen line lit, and the one under the pointer softly.
        match (c.value == now && !now.is_empty(), over.is_some_and(|over| r.contains(over))) {
            (true, _) => label(paint, r, radius, PLight::Strong, false, ""),
            (false, true) => label(paint, r, radius, PLight::Soft, false, ""),
            (false, false) => {}
        }
        let left = arrow(c.depth);
        let mark = match c.fold {
            Some(true) => Some("chevron-down"),
            Some(false) => Some("chevron-right"),
            None => None,
        };
        let shrink = step / 4.0;
        mark.into_iter().for_each(|mark| paint.icon(mark, PRect::new(left + shrink, r.center().y - step / 2.0 + shrink, step - shrink * 2.0, step - shrink * 2.0), dim));
        let left = left + step;
        let words = match c.icon.is_empty() {
            true => left,
            false => {
                paint.icon(&c.icon, PRect::new(left, r.center().y - icon / 2.0, icon, icon), ink);
                left + icon + gap / 4.0
            }
        };
        let room = (r.origin.x + r.size.width - words - gap / 4.0).max(0.0);
        paint.text(PPoint::new(words, r.center().y - line / 2.0), &fitted(&c.label, room, paint), ink);
        rect.push(r);
    }
    (at.y + choice.len() as f32 * high, rect)
}

/// Facts: a row each, the choice's words and its note.
pub fn facts(choice: &[PChoice], paint: &mut dyn PPaint, at: PPoint, wide: f32) -> (f32, Vec<PRect>) {
    let row: Vec<(String, String)> = choice.iter().map(|c| (c.label.clone(), c.end.clone())).collect();
    (row.table(paint, at, wide), Vec::new())
}

/// The theme's token for how many times as wide as high a banner is.
pub const BANNER: &str = "form.banner";

/// The first choice's picture, as wide as the block and as high, at the
/// top of its frame; as a banner, as wide as the theme says times its
/// height, covering it. Or its face, the cell of a sheet, square in the
/// middle. A press on it keeps its value.
pub fn picture(choice: &[PChoice], paint: &mut dyn PPaint, at: PPoint, wide: f32, banner: bool) -> (f32, Vec<PRect>) {
    let Some(first) = choice.first() else { return (at.y, Vec::new()) };
    let (high, fit) = match banner {
        true => (wide / paint.theme().size(BANNER).unwrap_or(2.0).max(1.0), PFit::Cover),
        false => (wide, PFit::Top),
    };
    let (r, radius) = (PRect::new(at.x, at.y, wide, high), paint.size(RADIUS));
    match &first.face {
        Some((sheet, column, index)) => paint.picture(sheet, PRect::new(at.x + (wide - high) / 2.0, at.y, high, high), radius, PFit::Cell { column: *column, index: *index }),
        None => paint.picture(&first.picture, r, radius, fit),
    }
    // What can be done with it: round icons down its top left corner,
    // those that are on lit.
    let (gap, icon) = (paint.size(GAP), paint.size(ICON));
    let side = icon + gap * 0.75;
    let deed = first.deed.iter().enumerate().map(|(index, deed)| {
        let place = PRect::new(at.x + gap / 2.0, at.y + gap / 2.0 + index as f32 * (side + gap / 4.0), side, side);
        let light = match deed.on {
            true => PLight::Strong,
            false => PLight::Rest,
        };
        label(paint, place, side / 2.0, light, false, "");
        paint.icon(&deed.icon, PRect::new(place.center().x - icon / 2.0, place.center().y - icon / 2.0, icon, icon), paint.color(INK));
        place
    });
    (at.y + high, std::iter::once(r).chain(deed).collect())
}

/// Words cut to a width, the cut shown by an ellipsis.
fn fitted(said: &str, room: f32, paint: &dyn PPaint) -> String {
    match paint.measure(said) <= room {
        true => said.to_string(),
        false => {
            let mut cut: String = said.to_string();
            while !cut.is_empty() && paint.measure(&format!("{cut}…")) > room {
                cut.pop();
            }
            format!("{cut}…")
        }
    }
}
