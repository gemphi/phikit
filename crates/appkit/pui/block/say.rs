//! What somebody said: a message of a conversation. Over it, where it
//! answers something, what that was, in one line; then their face, who
//! they are and when, with a mark after it where the app has one (a
//! pin); the words, broken to fit; the pictures posted with it, one
//! under the other; and what it was marked with, a pill a mark. One
//! that goes on from the one before has no head of its own.
//!
//! It is drawn in place and says where each of its things stands, so
//! the page that holds it knows where a press does what.

use super::super::core::{PFit, PPaint, PPoint, PRect};
use super::super::layout::wrap;
use super::{DIM, GAP, ICON, INK, LINE, PFace, PPill, PText, RADIUS};

/// The theme's tokens: how large the face of who said it is, and how
/// wide and how high a picture posted with it is at most.
pub const FACE: &str = "say.face";
pub const WIDE: &str = "say.wide";
pub const HIGH: &str = "say.high";

/// Where the things of a message stand, once it is drawn.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PSaid {
    /// The line over it that says what it answers.
    pub quote: Option<PRect>,
    /// The face of who said it; none on one that goes on from another.
    pub face: Option<PRect>,
    /// Its pictures, in order, and its marks, in order.
    pub image: Vec<PRect>,
    pub mark: Vec<PRect>,
    /// How far down its own first line is, under what it answers.
    pub head: f32,
    /// How far down it ends: where the next thing begins.
    pub end: f32,
}

/// A message.
pub trait PSay {
    /// The picture of who said it, as a cell of a sheet.
    fn cell(&self) -> Option<(&str, u32, u32)> {
        None
    }

    /// Who said it, and when.
    fn by(&self) -> &str;
    fn when(&self) -> &str {
        ""
    }

    /// What was said.
    fn text(&self) -> &str {
        ""
    }

    /// What it answers, as one line; nothing for one that answers none.
    fn quote(&self) -> &str {
        ""
    }

    /// The icon after when it was said: a pin on one that is pinned.
    fn flag(&self) -> &str {
        ""
    }

    /// The pictures posted with it, by their keys.
    fn image(&self) -> Vec<&str> {
        Vec::new()
    }

    /// What it was marked with: an icon, how many gave it, and whether
    /// the person did.
    fn mark(&self) -> Vec<(&str, &str, bool)> {
        Vec::new()
    }

    /// Whether it goes on from the one before, by the same person.
    fn follows(&self) -> bool {
        false
    }

    /// Draw it in a place, as wide as the place and as high as it
    /// needs: where its things stand.
    fn say(&self, paint: &mut dyn PPaint, at: PRect) -> PSaid {
        let (gap, line, icon, face) = (paint.size(GAP), paint.size(LINE), paint.size(ICON), paint.size(FACE));
        let (ink, dim, corner) = (paint.color(INK), paint.color(DIM), paint.size(RADIUS));
        let (left, wide) = (at.origin.x + face + gap / 2.0, (at.size.width - face - gap / 2.0).max(0.0));
        let mut top = at.origin.y;
        // What it answers, over it.
        let quote = Some(self.quote()).filter(|quote| !quote.is_empty()).map(|quote| {
            let said = quote.fit(wide - gap, paint);
            paint.text(PPoint::new(left + gap, top), &said, dim);
            let rect = PRect::new(left + gap, top, paint.measure(&said), line);
            top += line;
            rect
        });
        let head = top;
        // The face of who said it, then who and when.
        let seat = (!self.follows()).then(|| {
            let rect = PRect::new(at.origin.x, top, face, face);
            (self.cell(), self.by()).face(paint, rect);
            paint.text(PPoint::new(left, top), self.by(), ink);
            let after = left + paint.measure(self.by()) + gap / 2.0;
            paint.text(PPoint::new(after, top), self.when(), dim);
            let flag = PRect::new(after + paint.measure(self.when()) + gap / 2.0, top + (line - icon) / 2.0, icon, icon);
            Some(self.flag()).filter(|flag| !flag.is_empty()).into_iter().for_each(|name| paint.icon(name, flag, dim));
            top += line;
            rect
        });
        let gauge = |text: &str| paint.measure(text);
        let broken = wrap(self.text(), wide, &gauge);
        broken.iter().for_each(|words| {
            paint.text(PPoint::new(left, top), words, ink);
            top += line;
        });
        // Its pictures, each covering its frame.
        let (across, high) = (paint.size(WIDE).min(wide), paint.size(HIGH));
        let image: Vec<PRect> = self.image().into_iter().map(|key| {
            let rect = PRect::new(left, top + gap / 4.0, across, high);
            paint.picture(key, rect, corner, PFit::Cover);
            top += high + gap / 2.0;
            rect
        }).collect();
        // What it was marked with: a pill a mark, lit when the person gave it.
        let mut next = left;
        let mark: Vec<PRect> = self.mark().into_iter().map(|pill| {
            let rect = pill.draw(paint, PPoint::new(next, top + gap / 4.0), false);
            next += rect.size.width + gap / 2.0;
            rect
        }).collect();
        let end = top + match mark.is_empty() {
            true => gap / 2.0,
            false => line + gap,
        };
        PSaid { quote, face: seat, image, mark, head, end }
    }
}
