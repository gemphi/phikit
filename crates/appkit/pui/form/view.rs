//! A form as a view: where a press does something, its frame, what moves
//! by itself, and what a press, a key, the pointer and the wheel do.

use super::group::FOCUS;
use super::head::DROP;
use super::PForm;
use crate::pui::core::{PEvent, PPoint, PRect, PView};

impl PView for PForm {
    fn hand(&self) -> Vec<PRect> {
        self.spot.borrow().iter().map(|(rect, ..)| *rect).collect()
    }

    fn frame(&self) -> PRect {
        self.frame
    }

    fn set_frame(&mut self, frame: PRect) {
        self.frame = frame;
    }

    /// The tabs' light moves while it is on its way; what is live is
    /// drawn every frame.
    fn tick(&mut self) -> bool {
        self.motion.tick(self.theme.as_ref()) || self.source.live()
    }

    fn beat(&self) -> Option<u64> {
        self.source.beat()
    }

    /// A press keeps the value of what it pressed at its block's path;
    /// the wheel scrolls it.
    fn respond(&mut self, event: &PEvent) -> bool {
        match *event {
            PEvent::PointerDown(at) => {
                let place = at.plus(self.origin.get());
                let hit = self.spot.borrow().iter().find(|(rect, ..)| rect.contains(place)).map(|(_, path, value)| (path.clone(), value.clone()));
                // A press anywhere puts an open drop away, but on its switch.
                let open = !self.source.get(DROP).is_empty();
                if open && hit.as_ref().is_none_or(|(path, _)| path != DROP) {
                    self.source.set(DROP, "");
                }
                match hit {
                    Some((path, key)) if path == FOCUS => *self.typing.borrow_mut() = Some(key),
                    Some((path, value)) => {
                        *self.typing.borrow_mut() = None;
                        self.source.set(&path, &value);
                    }
                    None => return open,
                }
                true
            }
            // The window is drawn again when the pointer comes over
            // another place to press, or leaves one.
            PEvent::PointerMove(at) => {
                let place = at.plus(self.origin.get());
                let under = |point: Option<PPoint>| point.and_then(|point| self.spot.borrow().iter().position(|(rect, ..)| rect.contains(point)));
                let before = under(self.over.replace(Some(place)));
                before != under(Some(place))
            }
            PEvent::KeyDown(_) => self.typed(event),
            PEvent::Scroll(_, by) => {
                let mut scroll = self.scroll.get();
                let any = scroll.turn(-by.y);
                self.scroll.set(scroll);
                any
            }
            _ => false,
        }
    }
}
