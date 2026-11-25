//! A form's `field` block: a line a hand types into, as a comment box
//! has one. A press gives it the keys; Enter keeps what was typed at the
//! block's path (the app adds it where it keeps such things) and empties
//! the line; Escape lets the keys go. A field whose variation says `live`
//! keeps what is typed at its path as it is typed (a search). Its words before anything is
//! typed are the block's first choice's.

use super::PChoice;
use crate::pui::core::light::surface;
use crate::pui::core::{PEvent, PLight, PPaint, PPoint, PRect};

const GAP: &str = "gap";
const LINE: &str = "text.line";
const INK: &str = "text.ink";
const DIM: &str = "text.dim";
const RADIUS: &str = "row.radius";

/// What a key did to a line being typed.
#[derive(Debug, Clone, PartialEq)]
pub enum PTyped {
    /// The line changed, or not, and is still being typed.
    Edit,
    /// Enter: what was typed, to keep.
    Kept(String),
    /// Escape: the keys are let go.
    Left,
}

/// Draw the line: its ground, what was typed (or the hint), and a caret
/// while it has the keys. Where it ends, and where a press gives it them.
pub fn draw(choice: &[PChoice], draft: &str, typing: bool, paint: &mut dyn PPaint, at: PPoint, wide: f32) -> (f32, Vec<PRect>) {
    let (gap, line, ink, dim, radius) = (paint.size(GAP), paint.size(LINE), paint.color(INK), paint.color(DIM), paint.size(RADIUS));
    let r = PRect::new(at.x, at.y, wide, line + gap);
    let light = match typing {
        true => PLight::Strong,
        false => PLight::Soft,
    };
    surface(paint, r, radius, light, "");
    let hint = choice.first().map(|c| c.label.as_str()).unwrap_or_default();
    let (said, colour) = match draft.is_empty() {
        true => (hint, dim),
        false => (draft, ink),
    };
    let place = PPoint::new(r.origin.x + gap / 2.0, r.center().y - line / 2.0);
    paint.text(place, said, colour);
    if typing {
        let x = place.x + paint.measure(draft) + 1.0;
        paint.fill(PRect::new(x, place.y, 1.0, line), ink);
    }
    (r.origin.y + r.size.height, vec![r])
}

/// A key given to the line being typed.
pub fn key(draft: &mut String, event: &PEvent) -> Option<PTyped> {
    match *event {
        PEvent::KeyDown(PEvent::ENTER) => Some(PTyped::Kept(std::mem::take(draft))),
        PEvent::KeyDown(PEvent::ESCAPE) => Some(PTyped::Left),
        PEvent::KeyDown(PEvent::BACKSPACE) => {
            draft.pop();
            Some(PTyped::Edit)
        }
        PEvent::KeyDown(letter) if !letter.is_control() && !('\u{F700}'..='\u{F8FF}').contains(&letter) => {
            draft.push(letter);
            Some(PTyped::Edit)
        }
        _ => None,
    }
}
