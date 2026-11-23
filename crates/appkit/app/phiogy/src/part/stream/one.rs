//! One message of the stream: the face of who said it, a press on
//! which opens them beside the room; over it what it answers; who and
//! when, with a mark where it is pinned; its words, broken to fit; its
//! pictures, each covering a frame of its own; the marks it was given,
//! each with its count; and, for the message under the pointer or the
//! one pressed last, its controls at the end of its first line. A
//! message that follows one of the same person has no face or name of
//! its own.

use appkit::pui::block::{PBand, PSay};
use appkit::pui::core::{PPaint, PPoint, PRect};

use super::{Act, LIKE, PIN, REPLY, Spot, Stream};
use crate::part::GAP;
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
        let text = &self.desk.text;
        // What the kit's message is drawn from: whom it answers and how
        // that began, who said it and when, a pin where it is pinned,
        // and what it was marked with.
        let to = all.iter().find(|other| !one.reply.is_empty() && other.id == one.reply);
        let flag = self.tool.of(PIN, false, text).filter(|_| self.desk.pinned(one)).map_or("", |(name, _)| name);
        let marks = self.desk.mark(one);
        let said = Said {
            one,
            follows,
            cell: self.desk.cell(&one.who),
            quote: to.map(|to| text.fill("stream.reply", "name", &to.by).replace("{text}", &to.text)).unwrap_or_default(),
            when: format!("{} {} {}", one.day, text.get("between"), one.at),
            flag,
            mark: marks.iter().map(|(mark, count, on)| (self.desk.icon(mark), count.to_string(), *on)).collect(),
        };
        let laid = said.say(paint, at);
        // Where a press does something: what it answers leads there, a
        // face opens the person, a picture is seen whole (its number is
        // its place among all the room's), a mark is given or taken back.
        let word = |name: &str, by: &str| text.fill(name, "name", by);
        spot.extend(laid.quote.zip(to).map(|(rect, to)| Spot { rect, act: Act::Seek(to.id.clone()), word: word("stream.seek", &to.by) }));
        spot.extend(laid.face.map(|rect| Spot { rect, act: Act::Face(one.who.clone()), word: word("stream.open", &one.by) }));
        let before: usize = all.iter().take_while(|other| other.id != one.id).map(|other| other.image.len()).sum();
        spot.extend(laid.image.iter().enumerate().map(|(index, rect)| Spot { rect: *rect, act: Act::Shot(before + index), word: word("stream.see", &one.by) }));
        spot.extend(laid.mark.iter().zip(&marks).map(|(rect, (mark, ..))| Spot { rect: *rect, act: Act::Mark(one.id.clone(), mark.clone()), word: String::new() }));
        // Its controls, where the pointer is or the last press was.
        let (gap, over) = (paint.size(GAP), *self.over.borrow() == one.id);
        over.then(|| self.control(paint, PPoint::new(at.origin.x + at.size.width, laid.head - gap / 4.0), one, spot));
        laid.end
    }
}

/// A message as the kit draws one: the site's own, with what the app
/// knows of it.
struct Said<'a> {
    one: &'a Message,
    follows: bool,
    cell: Option<(&'a str, u32, u32)>,
    quote: String,
    when: String,
    flag: &'a str,
    mark: Vec<(&'a str, String, bool)>,
}

impl PSay for Said<'_> {
    fn cell(&self) -> Option<(&str, u32, u32)> {
        self.cell
    }

    fn by(&self) -> &str {
        &self.one.by
    }

    fn when(&self) -> &str {
        &self.when
    }

    fn text(&self) -> &str {
        &self.one.text
    }

    fn quote(&self) -> &str {
        &self.quote
    }

    fn flag(&self) -> &str {
        self.flag
    }

    fn image(&self) -> Vec<&str> {
        self.one.image.iter().map(String::as_str).collect()
    }

    fn mark(&self) -> Vec<(&str, &str, bool)> {
        self.mark.iter().map(|(icon, count, on)| (*icon, count.as_str(), *on)).collect()
    }

    fn follows(&self) -> bool {
        self.follows
    }
}
