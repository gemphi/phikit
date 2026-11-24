//! The viewer as a view: where a press does something, what is forgotten
//! once it is put away, a tool told as a word, and what a press, a key,
//! the pointer and the wheel do: with a pen, a press on the picture
//! begins a mark, a line follows the pointer while it is held, and words
//! being set have the keys.

use appkit::pui::core::{PEvent, PPoint, PRect, PView};

use super::{Act, Mark, Pen, Viewer};

impl PView for Viewer {
    /// Where a press on it does something.
    fn hand(&self) -> Vec<PRect> {
        self.spot.borrow().iter().map(|spot| spot.rect).collect()
    }

    fn frame(&self) -> PRect {
        self.frame
    }

    fn set_frame(&mut self, frame: PRect) {
        self.frame = frame;
    }

    /// A viewer that was put away forgets what was true of its picture:
    /// opened again, the picture is whole and was not shared.
    fn tick(&mut self) -> bool {
        if !self.live.replace(false) {
            self.seen.set(None);
            self.over.set(None);
        }
        false
    }

    /// A tool's name told as a word does what a press on it does.
    fn tell(&mut self, word: &str) -> bool {
        let tool = Act::named(word).filter(|act| *act != Act::Close && self.desk.view.get().is_some());
        // What a tool leaves so is true of the picture shown now, and
        // holds through the first frame.
        if tool.is_some() {
            self.seen.set(self.desk.view.get());
            self.live.set(true);
        }
        tool.into_iter().for_each(|act| self.act(act));
        tool.is_some()
    }

    fn respond(&mut self, event: &PEvent) -> bool {
        // What was drawn is in the window's coordinates.
        let under = |at: PPoint| self.spot.borrow().iter().find(|spot| spot.rect.contains(at.plus(self.origin.get()))).map(|spot| spot.act);
        match *event {
            // Words being set on the picture have the keys.
            PEvent::KeyDown(key) if self.writing.get() => self.write(key),
            PEvent::KeyDown(PEvent::LEFT) => self.step(-1),
            PEvent::KeyDown(PEvent::RIGHT) => self.step(1),
            // A line being sketched follows the pointer while it is held.
            PEvent::PointerMove(at) if self.drawing.get() => {
                let point = self.share(at.plus(self.origin.get()));
                if let Some(Mark::Line { point: line, .. }) = self.marks().last() {
                    line.push(point);
                }
                true
            }
            PEvent::PointerUp(_) => self.drawing.replace(false),
            // The window is drawn again when the pointer comes over
            // another control, or leaves one.
            PEvent::PointerMove(at) => self.over.replace(under(at)) != under(at),
            // The wheel moves a picture shown larger, as far as it goes.
            PEvent::Scroll(_, by) if self.large.get() => {
                let (now, most) = (self.shift.get(), self.reach.get());
                self.shift.set(PPoint::new((now.x - by.x).clamp(0.0, most.x), (now.y - by.y).clamp(0.0, most.y)));
                true
            }
            PEvent::PointerDown(at) => {
                let place = at.plus(self.origin.get());
                self.writing.set(false);
                match (under(at), self.shown.get().contains(place), self.pen.get()) {
                    (Some(act), ..) => self.act(act),
                    // With a pen, a press on the picture begins a mark.
                    (None, true, Some(Pen::Sketch)) => {
                        self.marks().add(Mark::Line { ink: self.ink.get(), point: vec![self.share(place)] });
                        self.drawing.set(true);
                    }
                    (None, true, Some(Pen::Text)) => {
                        self.marks().add(Mark::Word { ink: self.ink.get(), at: self.share(place), text: String::new() });
                        self.writing.set(true);
                    }
                    (None, true, None) => self.zoom(),
                    // A press beside the picture puts the viewer away.
                    (None, false, _) => self.act(Act::Close),
                }
                true
            }
            _ => false,
        }
    }
}
