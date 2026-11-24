//! A form's toolbar: the blocks whose variation has the word `bar` stand
//! together under its heading, with no heading or ground of their own, as
//! Finder's, Reality Composer Pro's and Blender's file browsers have one
//! over what they show. They lie left to right on a line; `end` puts a
//! block at the line's right end, `next` begins a new line, and a line
//! too narrow for both its ends gives its right end a line of its own.
//!
//! In a bar a block can be, beside `tools` and `band`:
//!
//! * `crumb`: the way to where the form is, each step pressed to go
//!   there, the mark of what comes next between them (`PCrumb`), the
//!   last step, where it is, in the ink.
//! * `layout`: the switch between a grid and a list, kept at its path.

use super::head::LIST;
use super::{Block, PChoice, PForm, Spot, kind};
use crate::pui::block::{PBand, PCrumb};
use crate::pui::core::{PPaint, PPoint, PRect};
use crate::pui::layout::PDry;

/// A block of a bar: how wide and high it is, and how it is drawn there.
struct Piece<'a> {
    block: &'a Block,
    choice: Vec<PChoice>,
    now: String,
    wide: f32,
    high: f32,
    end: bool,
    next: bool,
}

/// Whether a block's variation has a word.
fn says(block: &Block, word: &str) -> bool {
    block.variant.split_whitespace().any(|one| one == word)
}

/// Whether a block stands in the bar.
pub fn barred(block: &Block) -> bool {
    says(block, "bar")
}

/// The two ways a layout switch keeps: a grid (nothing) and a list.
fn layout() -> Vec<PChoice> {
    [("grid", ""), ("list", LIST)].map(|(icon, value)| PChoice { icon: icon.to_string(), value: value.to_string(), ..PChoice::default() }).to_vec()
}

/// The pills of a band, a tools block or a layout switch.
fn pills<'a>(piece: &'a Piece) -> Vec<(&'a str, &'a str, bool)> {
    let label = |c: &'a PChoice| match piece.block.kind.as_str() {
        "band" => c.label.as_str(),
        _ => "",
    };
    piece.choice.iter().map(|c| (c.icon.as_str(), label(c), piece.block.kind != "tools" && c.value == piece.now)).collect()
}

/// Draw a piece from a place: where each of its choices stands.
fn draw(piece: &Piece, paint: &mut dyn PPaint, at: PPoint) -> Vec<PRect> {
    match piece.block.kind.as_str() {
        "crumb" => {
            let step: Vec<&str> = piece.choice.iter().map(|c| c.label.as_str()).collect();
            let Some((last, way)) = step.split_last() else { return Vec::new() };
            let (mut rect, across) = way.lead(paint, at);
            let wide = paint.measure(last);
            paint.text(PPoint::new(across, at.y), last, paint.color(kind::INK));
            rect.push(PRect::new(across, at.y, wide, paint.size(kind::LINE)));
            rect
        }
        _ => pills(piece).draw(paint, at),
    }
}

impl PForm {
    /// What a bar block is, measured with nothing drawn.
    fn piece<'a>(&self, block: &'a Block, paint: &dyn PPaint) -> Option<Piece<'a>> {
        let choice = match block.kind.as_str() {
            "layout" => layout(),
            _ => self.source.list(&block.list),
        };
        let mut piece = Piece { block, choice, now: self.source.get(&block.path), wide: 0.0, high: 0.0, end: says(block, "end"), next: says(block, "next") };
        if piece.choice.is_empty() {
            return None;
        }
        (piece.wide, piece.high) = match block.kind.as_str() {
            "crumb" => {
                let gap = paint.size(kind::GAP);
                let across: f32 = piece.choice.iter().map(|c| paint.measure(&c.label) + gap * 1.5).sum();
                (across - gap * 1.5, paint.size(kind::LINE))
            }
            _ => {
                let pill = pills(&piece);
                (pill.span(paint), pill.high(paint))
            }
        };
        Some(piece)
    }

    /// Draw the bar from the top of a place across a width: where it
    /// ends, and what a press on each place keeps.
    pub(super) fn bar(&self, paint: &mut dyn PPaint, left: f32, top: f32, wide: f32) -> (f32, Vec<Spot>) {
        let gap = paint.size(kind::GAP);
        let piece: Vec<Piece> = self.block.iter().filter(|b| barred(b)).filter_map(|b| self.piece(b, &PDry(&*paint))).collect();
        // The lines: each begins where a piece says `next`.
        let mut line: Vec<Vec<&Piece>> = Vec::new();
        for one in &piece {
            match (one.next, line.last_mut()) {
                (false, Some(last)) => last.push(one),
                _ => line.push(vec![one]),
            }
        }
        // A line whose ends do not both fit gives its right end its own.
        let lines: Vec<Vec<&Piece>> = line.into_iter().flat_map(|one| {
            let (lead, end): (Vec<&Piece>, Vec<&Piece>) = one.into_iter().partition(|p| !p.end);
            let span = |list: &[&Piece]| list.iter().map(|p| p.wide + gap / 2.0).sum::<f32>();
            match span(&lead) + span(&end) <= wide + gap / 2.0 || lead.is_empty() {
                true => vec![lead.into_iter().chain(end).collect::<Vec<_>>()],
                false => vec![lead, end],
            }
        }).collect();
        let mut spot = Vec::new();
        let mut y = top;
        for one in &lines {
            let high = one.iter().map(|p| p.high).fold(0.0, f32::max);
            let end: f32 = one.iter().filter(|p| p.end).map(|p| p.wide + gap / 2.0).sum();
            let (mut x, mut right) = (left, left + wide - (end - gap / 2.0).max(0.0));
            for p in one {
                let place = match p.end {
                    true => &mut right,
                    false => &mut x,
                };
                let rect = draw(p, paint, PPoint::new(*place, y + (high - p.high) / 2.0));
                *place += p.wide + gap / 2.0;
                spot.extend(rect.into_iter().zip(&p.choice).map(|(r, c)| (r, p.block.path.clone(), c.value.clone())));
            }
            y += high + gap / 2.0;
        }
        (y, spot)
    }
}
