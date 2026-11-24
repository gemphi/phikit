//! Three more kinds of a form's block, for a browser and an inspector:
//!
//! * `tree`: each choice a line, its icon and its words, set in by its
//!   depth, the chosen one lit: a navigator, as Reality Composer Pro's
//!   Project Browser and Blender's file browser have one.
//! * `facts`: each choice a row, its words at the left and its note at
//!   the right, a line under it: an item's attributes.
//! * `picture`: the first choice's picture across the whole width,
//!   standing at the top: an item seen in the inspector.

use super::PChoice;
use crate::pui::block::PFacts;
use crate::pui::core::light::label;
use crate::pui::core::{PFit, PLight, PPaint, PPoint, PRect};

const GAP: &str = "gap";
const LINE: &str = "text.line";
const INK: &str = "text.ink";
const ICON: &str = "icon.size";
const RADIUS: &str = "row.radius";

/// A tree's lines: where it ends, and where each line stands.
pub fn tree(choice: &[PChoice], now: &str, paint: &mut dyn PPaint, at: PPoint, wide: f32) -> (f32, Vec<PRect>) {
    let (gap, line, ink, icon, radius) = (paint.size(GAP), paint.size(LINE), paint.color(INK), paint.size(ICON), paint.size(RADIUS));
    let high = line + gap / 2.0;
    let mut rect = Vec::new();
    for (index, c) in choice.iter().enumerate() {
        let r = PRect::new(at.x, at.y + index as f32 * high, wide, high);
        if c.value == now && !now.is_empty() {
            label(paint, r, radius, PLight::Strong, false, "");
        }
        let left = r.origin.x + gap / 4.0 + c.depth as f32 * gap;
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

/// The first choice's picture, as wide as the block and as high, at the
/// top of its frame; a press on it keeps its value.
pub fn picture(choice: &[PChoice], paint: &mut dyn PPaint, at: PPoint, wide: f32) -> (f32, Vec<PRect>) {
    let Some(first) = choice.first() else { return (at.y, Vec::new()) };
    let r = PRect::new(at.x, at.y, wide, wide);
    paint.picture(&first.picture, r, paint.size(RADIUS), PFit::Top);
    (at.y + wide, vec![r])
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
