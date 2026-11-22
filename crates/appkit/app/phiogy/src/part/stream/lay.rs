//! How the stream is laid out: over it the search field while the room
//! is searched and its pinned messages when they are asked for; at its
//! foot what is being answered and the field a message is written in;
//! between them the messages, scrolled with the wheel.

use appkit::pui::control::{PControl, inside};
use appkit::pui::core::{PColor, PDraw, PLight, PPaint, PPoint, PRect, PView, light, show};
use appkit::theme::PTheme;

use super::{Act, POST, SEND, Spot, Stream, UNANSWER};
use crate::part::{DIM, GAP, ICON, INK, LINE, RADIUS, ROW, fit, tool};
use crate::site::Message;

/// A painter that draws nothing and measures as another does: with it
/// a room is laid out to learn how long it is.
struct Dry<'a>(&'a dyn PPaint);

impl PPaint for Dry<'_> {
    fn theme(&self) -> &dyn PTheme {
        self.0.theme()
    }

    fn measure(&self, text: &str) -> f32 {
        self.0.measure(text)
    }

    fn fill(&mut self, _: PRect, _: PColor) {}

    fn text(&mut self, _: PPoint, _: &str, _: PColor) {}

    fn icon(&mut self, _: &str, _: PRect, _: PColor) {}
}

impl PDraw for Stream {
    /// Another room, and a room with one more message, are shown from
    /// the foot, where the newest is; a message that was asked for is
    /// brought into sight. Either is laid out once with nothing drawn,
    /// to learn where things are.
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        let now = (self.desk.room(), self.desk.said().len());
        let sought = self.desk.now.seek.borrow_mut().take();
        let fresh = *self.seen.borrow() != now;
        if fresh || sought.is_some() {
            self.lay(&mut Dry(&*paint), origin);
            let top = sought.and_then(|id| self.place.borrow().iter().find(|(known, ..)| *known == id).map(|(_, top, _)| *top));
            let by = match top {
                Some(top) => top - self.body.get().origin.y - self.body.get().size.height / 3.0,
                None => f32::MAX,
            };
            let mut scroll = self.scroll.get();
            scroll.turn(by);
            self.scroll.set(scroll);
            *self.seen.borrow_mut() = now;
        }
        self.lay(paint, origin);
    }
}

impl Stream {
    /// A line of a message: who said it and how it began.
    fn line(&self, one: &Message) -> String {
        let words = match one.text.is_empty() {
            true => self.desk.text.get("stream.media"),
            false => one.text.as_str(),
        };
        self.desk.text.fill("stream.reply", "name", &one.by).replace("{text}", words)
    }

    /// The head: the search field, and the pinned messages when they
    /// are asked for. How high it is.
    fn head(&self, paint: &mut dyn PPaint, origin: PPoint, all: &[Message], spot: &mut Vec<Spot>) -> f32 {
        let (gap, line, row, radius) = (paint.size(GAP), paint.size(LINE), paint.size(ROW), paint.size(RADIUS));
        let (dim, text) = (paint.color(DIM), &self.desk.text);
        let (start, wide) = self.desk.column(self.frame.size.width);
        let label = self.desk.plate(&self.desk.room()).map(|plate| plate.label).unwrap_or_default();
        let mut high = 0.0;
        if self.desk.now.finding.get() {
            show(&self.find, paint, origin);
            if self.find.value().is_empty() {
                let frame = self.find.frame();
                paint.text(origin.plus(frame.origin).plus(inside(paint, frame.size.height)), &text.fill("stream.find", "room", &label), dim);
            }
            high += row + gap;
        }
        if self.desk.now.pins.get() {
            let pinned: Vec<&Message> = all.iter().filter(|one| self.desk.pinned(one)).collect();
            let rect = PRect::new(origin.x + start, origin.y + high + gap / 2.0, wide, (pinned.len().max(1) + 1) as f32 * line + gap);
            light::surface(paint, rect, radius, PLight::Rest, "");
            let left = rect.origin.x + gap / 2.0;
            paint.text(PPoint::new(left, rect.origin.y + gap / 2.0), text.get("stream.pins"), dim);
            if pinned.is_empty() {
                paint.text(PPoint::new(left, rect.origin.y + gap / 2.0 + line), text.get("stream.unpinned"), dim);
            }
            let ink = paint.color(INK);
            for (index, one) in pinned.iter().enumerate() {
                let at = PPoint::new(left, rect.origin.y + gap / 2.0 + (index + 1) as f32 * line);
                paint.text(at, &fit(&self.line(one), wide - gap, paint), ink);
                spot.push(Spot { rect: PRect::new(rect.origin.x, at.y, wide, line), act: Act::Seek(one.id.clone()), word: text.fill("stream.seek", "name", &one.by) });
            }
            high += rect.size.height + gap;
        }
        high
    }

    /// The foot: what is being answered, and the field with the control
    /// that says what it holds. How high it is.
    fn foot(&self, paint: &mut dyn PPaint, origin: PPoint, all: &[Message], spot: &mut Vec<Spot>) -> f32 {
        let (gap, line, row, icon) = (paint.size(GAP), paint.size(LINE), paint.size(ROW), paint.size(ICON));
        let (dim, ink, text) = (paint.color(DIM), paint.color(INK), &self.desk.text);
        let frame = self.write.frame();
        let label = self.desk.plate(&self.desk.room()).map(|plate| plate.label).unwrap_or_default();
        show(&self.write, paint, origin);
        if self.write.value().is_empty() {
            paint.text(origin.plus(frame.origin).plus(inside(paint, frame.size.height)), &fit(&text.fill("stream.write", "room", &label), frame.size.width - gap, paint), dim);
        }
        // After the field: post a picture, then say what was written.
        let mut left = origin.x + frame.origin.x + frame.size.width + gap / 2.0;
        for (name, act) in [(POST, Act::Post), (SEND, Act::Send)] {
            let rect = PRect::new(left, origin.y + frame.origin.y, row, row);
            if let Some((icon, word)) = self.tool.of(name, false, text) {
                tool::button(paint, rect, icon, Some(self.lit(&act, false)));
                spot.push(Spot { rect, act, word: word.to_string() });
            }
            left += row + gap / 2.0;
        }
        let mut high = row + gap;
        let answer = self.desk.now.answer.borrow().clone();
        if let Some(one) = all.iter().find(|one| !answer.is_empty() && one.id == answer) {
            // A tag over the field: whom, and the way to answer nobody.
            let said = text.fill("stream.answer", "name", &one.by);
            let tag = PRect::new(origin.x + frame.origin.x, origin.y + frame.origin.y - line - gap / 2.0, paint.measure(&said) + gap * 1.5 + icon, line + gap / 4.0);
            let radius = paint.size(RADIUS);
            light::label(paint, tag, radius, PLight::Rest, false, "");
            paint.text(PPoint::new(tag.origin.x + gap / 2.0, tag.origin.y + gap / 8.0), &said, ink);
            if let Some((name, word)) = self.tool.of(UNANSWER, false, text) {
                let rect = PRect::new(tag.origin.x + tag.size.width - icon - gap / 2.0, tag.center().y - icon / 2.0, icon, icon);
                paint.icon(name, rect, ink);
                spot.push(Spot { rect, act: Act::Unanswer, word: word.to_string() });
            }
            high += line + gap / 2.0;
        }
        high
    }

    /// How strong the ground of a control is: more under the pointer,
    /// and when it is on.
    pub(super) fn lit(&self, act: &Act, on: bool) -> PLight {
        match (self.hand.borrow().as_ref() == Some(act), on) {
            (true, _) | (_, true) => PLight::Strong,
            (false, false) => PLight::Rest,
        }
    }

    /// Draw the stream with that painter, and keep where things are.
    pub(super) fn lay(&self, paint: &mut dyn PPaint, origin: PPoint) {
        let (gap, line, dim) = (paint.size(GAP), paint.size(LINE), paint.color(DIM));
        let (start, wide) = self.desk.column(self.frame.size.width);
        let all = self.desk.said();
        let mut spot = Vec::new();
        self.origin.set(origin);
        // The messages are between the head and the foot. They are laid
        // out first, with a painter that the other two cover.
        let over = self.band(&*paint, &all);
        let body = PRect::new(origin.x, origin.y + over.0, self.frame.size.width, (self.frame.size.height - over.0 - over.1).max(0.0));
        self.body.set(body);
        let shown = match self.desk.now.finding.get() {
            true => self.desk.sought(&self.find.value()),
            false => all.clone(),
        };
        paint.clip(body);
        let left = origin.x + start;
        let begin = body.origin.y + gap - self.scroll.get().at();
        let mut next = begin;
        let none = match (all.is_empty(), shown.is_empty()) {
            (true, _) => Some("stream.none"),
            (false, true) => Some("stream.unfound"),
            (false, false) => None,
        };
        if let Some(none) = none {
            paint.text(PPoint::new(left, next), self.desk.text.get(none), dim);
            next += line;
        }
        let mut place = Vec::new();
        for (index, one) in shown.iter().enumerate() {
            let before = index.checked_sub(1).and_then(|index| shown.get(index));
            // A new day is named between the messages.
            if before.is_none_or(|before| before.day != one.day) {
                paint.text(PPoint::new(left + (wide - paint.measure(&one.day)) / 2.0, next), &one.day, dim);
                next += line + gap / 2.0;
            }
            // What a muted person said is not shown: a line says so
            // where a row of theirs begins.
            if self.desk.muted(&one.who) {
                if before.is_none_or(|before| before.who != one.who) {
                    paint.text(PPoint::new(left, next), &self.desk.text.fill("stream.muted", "name", &one.by), dim);
                    next += line + gap / 2.0;
                }
                continue;
            }
            let follows = before.is_some_and(|before| before.day == one.day && before.who == one.who && one.reply.is_empty());
            // One person's messages stand close; another's a little apart.
            if !follows && index > 0 {
                next += gap / 2.0;
            }
            let foot = self.message(paint, PRect::new(left, next, wide, 0.0), one, follows, &all, &mut spot);
            place.push((one.id.clone(), next, foot));
            next = foot;
        }
        paint.unclip();
        // A place outside where the messages show is none.
        spot.retain(|spot| body.contains(spot.rect.center()));
        self.head(paint, origin, &all, &mut spot);
        self.foot(paint, origin, &all, &mut spot);
        // What the control under the pointer is called, by it.
        let hand = self.hand.borrow().clone();
        let said = hand.and_then(|act| spot.iter().find(|spot| spot.act == act).cloned());
        said.into_iter().for_each(|spot| tool::say(paint, spot.rect, &spot.word, PRect { origin, size: self.frame.size }));
        // Keep how long it is, for the wheel.
        let mut scroll = self.scroll.get();
        scroll.fit(next - begin + gap, body.size.height);
        self.scroll.set(scroll);
        (*self.spot.borrow_mut(), *self.place.borrow_mut()) = (spot, place);
    }

    /// How high the head and the foot are, learned by laying them out
    /// with nothing drawn.
    fn band(&self, paint: &dyn PPaint, all: &[Message]) -> (f32, f32) {
        let (mut dry, mut none) = (Dry(paint), Vec::new());
        (self.head(&mut dry, PPoint::default(), all, &mut none), self.foot(&mut dry, PPoint::default(), all, &mut none))
    }
}
