//! One message of the stream: the face of who said it, a press on
//! which opens them beside the room; over it what it answers; who and
//! when, with a mark where it is pinned; its words, broken to fit; its
//! pictures, each covering a frame of its own; the marks it was given,
//! each with its count; and, for the message under the pointer or the
//! one pressed last, its controls at the end of its first line. A
//! message that follows one of the same person has no face or name of
//! its own.

use appkit::pui::block::{PBand, PPill};
use appkit::pui::core::{PFit, PPaint, PPoint, PRect};
use appkit::pui::layout::wrap;

use super::{Act, FACE, HIGH, LIKE, PIN, REPLY, Spot, Stream, WIDE};
use crate::part::{DIM, GAP, ICON, INK, LINE, RADIUS, fit};
use crate::site::Message;

impl Stream {
    /// The controls of a message, the kit's band, ending at a place:
    /// answer it, like it, pin it. Each is lit when it is so.
    fn control(&self, paint: &mut dyn PPaint, end: PPoint, one: &Message, spot: &mut Vec<Spot>) {
        let text = &self.desk.text;
        let like = self.desk.like();
        let answered = *self.desk.now.answer.borrow() == one.id;
        let list = [(REPLY, Act::Reply(one.id.clone()), answered), (LIKE, Act::Mark(one.id.clone(), like.clone()), self.desk.gave(&one.id, &like)), (PIN, Act::Pin(one.id.clone()), self.desk.pinned(one))];
        let list: Vec<(&str, &str, bool, Act)> = list.into_iter().filter_map(|(name, act, on)| self.tool.of(name, on, text).map(|(icon, word)| (icon, word, on, act))).collect();
        let pill: Vec<(&str, &str, bool)> = list.iter().map(|(icon, _, on, _)| (*icon, "", *on)).collect();
        let rect = pill.draw_end(paint, end);
        spot.extend(rect.into_iter().zip(list).map(|(rect, (_, word, _, act))| Spot { rect, act, word: word.to_string() }));
    }

    /// One message in `at`, which gives its left, its top and its
    /// width. How far down the next begins.
    pub(super) fn message(&self, paint: &mut dyn PPaint, at: PRect, one: &Message, follows: bool, all: &[Message], spot: &mut Vec<Spot>) -> f32 {
        let (gap, line, icon, face) = (paint.size(GAP), paint.size(LINE), paint.size(ICON), paint.size(FACE));
        let (ink, dim, text) = (paint.color(INK), paint.color(DIM), &self.desk.text);
        let (left, wide) = (at.origin.x + face + gap / 2.0, (at.size.width - face - gap / 2.0).max(0.0));
        let mut top = at.origin.y;
        let pinned = self.desk.pinned(one);
        // Whom it answers, and how that began: a press leads there.
        if let Some(to) = all.iter().find(|other| !one.reply.is_empty() && other.id == one.reply) {
            let said = fit(&self.desk.text.fill("stream.reply", "name", &to.by).replace("{text}", &to.text), wide - gap, paint);
            paint.text(PPoint::new(left + gap, top), &said, dim);
            spot.push(Spot { rect: PRect::new(left + gap, top, paint.measure(&said), line), act: Act::Seek(to.id.clone()), word: text.fill("stream.seek", "name", &to.by) });
            top += line;
        }
        let first = top;
        if !follows {
            // The face of who said it, then who and when.
            let rect = PRect::new(at.origin.x, top, face, face);
            if let Some(cell) = self.desk.site.face(&one.who) {
                paint.picture(&cell.sheet, rect, face / 2.0, PFit::Cell { column: cell.column, index: cell.index });
            }
            spot.push(Spot { rect, act: Act::Face(one.who.clone()), word: text.fill("stream.open", "name", &one.by) });
            paint.text(PPoint::new(left, top), &one.by, ink);
            let when = format!("{} {} {}", one.day, text.get("between"), one.at);
            let after = left + paint.measure(&one.by) + gap / 2.0;
            paint.text(PPoint::new(after, top), &when, dim);
            if let Some((name, _)) = self.tool.of(PIN, false, text).filter(|_| pinned) {
                paint.icon(name, PRect::new(after + paint.measure(&when) + gap / 2.0, top + (line - icon) / 2.0, icon, icon), dim);
            }
            top += line;
        }
        for words in wrap(&one.text, wide, &|text| paint.measure(text)) {
            paint.text(PPoint::new(left, top), &words, ink);
            top += line;
        }
        // The pictures posted with it, each covering its frame: its
        // number is its place among all the room's.
        let (across, high, corner) = (paint.size(WIDE).min(wide), paint.size(HIGH), paint.size(RADIUS));
        let before: usize = all.iter().take_while(|other| other.id != one.id).map(|other| other.image.len()).sum();
        for (index, key) in one.image.iter().enumerate() {
            let rect = PRect::new(left, top + gap / 4.0, across, high);
            paint.picture(key, rect, corner, PFit::Cover);
            spot.push(Spot { rect, act: Act::Shot(before + index), word: text.fill("stream.see", "name", &one.by) });
            top += high + gap / 2.0;
        }
        // What it was marked with: a pill a mark, lit when the person
        // gave it; a press gives it or takes it back.
        let marks = self.desk.mark(one);
        let mut across = left;
        for (mark, count, on) in &marks {
            // A mark and how many gave it: the kit's pill.
            let said = count.to_string();
            let pill = (self.desk.icon(mark), said.as_str(), *on).draw(paint, PPoint::new(across, top + gap / 4.0), false);
            spot.push(Spot { rect: pill, act: Act::Mark(one.id.clone(), mark.clone()), word: String::new() });
            across += pill.size.width + gap / 2.0;
        }
        // Its controls, where the pointer is or the last press was.
        let over = *self.over.borrow() == one.id;
        over.then(|| self.control(paint, PPoint::new(at.origin.x + at.size.width, first - gap / 4.0), one, spot));
        top + match marks.is_empty() {
            true => gap / 2.0,
            false => line + gap,
        }
    }
}
