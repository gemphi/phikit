//! The room: what a page of the kind `chat` shows. What was said in it
//! (`stream`), and who is on its line and who its people are
//! (`present`). At a desk the people stand beside the stream, as on the
//! web; where the part is narrow they stand over it, and give their
//! place to a call this window is in (`call`). A press on a
//! face puts that person (`person`) in the people's place and, where
//! the part is narrow, in the room's. A voice room has who is in it
//! and its way in (`voice`) where another has what was said.
//!
//! A page is made of blocks, as the app's own file writes them. A block
//! that is a room is drawn here; one of a kind this window does not
//! draw yet is named, so a page added to the file is never empty.

use appkit::pui::core::{PDraw, PEvent, PLight, PPaint, PPoint, PRect, PView, light, send, show};

use super::{Calling, DIM, Desk, GAP, LINE, Person, Present, RADIUS, Stream, Voice};

/// The theme's tokens: how wide the part must be for the people to
/// stand beside the stream, how wide they stand there, and the share of
/// its height they have where they stand over it.
pub const SPLIT: &str = "room.split";
pub const SIDE: &str = "room.side";
pub const SHARE: &str = "room.share";

/// A room: its stream and its people.
pub struct Room {
    frame: PRect,
    desk: Desk,
    stream: Stream,
    /// The call this window is in, over the stream.
    calling: Calling,
    /// What a voice room shows in the stream's place.
    voice: Voice,
    present: Present,
    /// The person whose face was pressed, in the people's place.
    person: Person,
}

impl Room {
    pub fn new(desk: Desk) -> Room {
        Room { frame: PRect::default(), stream: Stream::new(desk.clone()), calling: Calling::new(desk.clone()), voice: Voice::new(desk.clone()), present: Present::new(desk.clone()), person: Person::new(desk.clone()), desk }
    }

    /// Whether a person stands beside the room.
    fn picked(&self) -> bool {
        self.desk.now.who.borrow().as_ref().is_some_and(|id| self.desk.site.member(id).is_some())
    }

    /// Whether the part is so narrow that the people stand over the
    /// stream, and a person in its place.
    fn narrow(&self) -> bool {
        self.frame.size.width < self.desk.size(SPLIT)
    }

    /// How high the lines that name blocks not drawn yet are.
    fn band(&self) -> f32 {
        self.desk.undrawn().len() as f32 * self.desk.size(LINE)
    }
}

impl PDraw for Room {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        let (gap, line, dim) = (paint.size(GAP), paint.size(LINE), paint.color(DIM));
        for (index, kind) in self.desk.undrawn().iter().enumerate() {
            let said = self.desk.text.fill("room.undrawn", "kind", kind);
            paint.text(origin.plus(PPoint::new(gap, index as f32 * line)), &said, dim);
        }
        // A person takes the people's place; narrow, the whole room.
        if self.picked() && self.narrow() {
            return show(&self.person, paint, origin);
        }
        // Each column of the room stands on a ground of its own,
        // against the room's.
        let radius = paint.size(RADIUS);
        for column in [self.voice.frame(), self.present.frame()].into_iter().filter(|column| column.size.width > 0.0 && column.size.height > 0.0) {
            light::label(paint, PRect { origin: origin.plus(column.origin), size: column.size }.inset(gap / 4.0), radius, PLight::Soft, true, "");
        }
        match self.desk.voice() {
            true => show(&self.voice, paint, origin),
            false => {
                show(&self.calling, paint, origin);
                show(&self.stream, paint, origin);
            }
        }
        match self.picked() {
            true => show(&self.person, paint, origin),
            false => show(&self.present, paint, origin),
        }
    }
}

impl PView for Room {
    /// Where a press on it does something.
    fn hand(&self) -> Vec<PRect> {
        let room = match self.desk.voice() {
            true => self.voice.hand(),
            false => [self.calling.hand(), self.stream.hand()].concat(),
        };
        let side = match self.picked() {
            true => self.person.hand(),
            false => self.present.hand(),
        };
        [room, side].concat()
    }

    fn frame(&self) -> PRect {
        self.frame
    }

    fn set_frame(&mut self, frame: PRect) {
        self.frame = frame;
        let (top, size) = (self.band(), frame.size);
        let high = (size.height - top).max(0.0);
        let (stream, present) = match size.width >= self.desk.size(SPLIT) {
            true => {
                let side = self.desk.size(SIDE).min(size.width / 2.0);
                (PRect::new(0.0, top, size.width - side, high), PRect::new(size.width - side, top, side, high))
            }
            false => {
                // In a call its tiles are the people: they take the
                // list's place.
                let over = match self.desk.calling() {
                    Some(_) => 0.0,
                    None => high * self.desk.size(SHARE),
                };
                (PRect::new(0.0, top + over, size.width, high - over), PRect::new(0.0, top, size.width, over))
            }
        };
        // A call this window is in stands over the stream, and takes
        // its height from it.
        self.voice.set_frame(stream);
        let call = self.calling.high(stream.size.width, stream.size.height);
        self.calling.set_frame(PRect::new(stream.origin.x, stream.origin.y, stream.size.width, call));
        self.stream.set_frame(PRect::new(stream.origin.x, stream.origin.y + call, stream.size.width, stream.size.height - call));
        self.present.set_frame(present);
        // A person has the people's place beside the stream, and all of
        // the room where it is narrow.
        self.person.set_frame(match size.width >= self.desk.size(SPLIT) {
            true => present,
            false => PRect::new(0.0, top, size.width, high),
        });
    }

    fn tick(&mut self) -> bool {
        // On another's line the open room's is listened to, so a call
        // somebody began there is seen.
        self.desk.listen();
        self.stream.tick() | self.present.tick()
    }

    /// A word for the room is the stream's: a picture to post.
    fn tell(&mut self, word: &str) -> bool {
        self.stream.tell(word)
    }

    fn respond(&mut self, event: &PEvent) -> bool {
        let room = |room: &mut Room| match room.desk.voice() {
            true => send(&mut room.voice, event),
            false => send(&mut room.calling, event) || send(&mut room.stream, event),
        };
        match (self.picked(), self.narrow()) {
            (true, true) => send(&mut self.person, event),
            (true, false) => send(&mut self.person, event) || room(self),
            (false, _) => send(&mut self.present, event) || room(self),
        }
    }
}
