//! The ask: a field to type in and a button that sends. The Enter key
//! sends as the button does. What is sent is said in the open chat, and
//! the field is emptied. While an answer is being written the button
//! stops it instead, and so does the Escape key.

use appkit::pui::control::{PButton, PControl, PField, inside};
use appkit::pui::core::{PDraw, PEvent, PPaint, PPoint, PRect, PView, send, show};

use super::{DIM, Desk, GAP};

/// The theme's token for how wide the button is.
pub const BUTTON: &str = "button.width";

/// A field to type in and a button that sends.
pub struct Ask {
    frame: PRect,
    desk: Desk,
    field: PField,
    send: PButton,
}

impl Ask {
    /// The ask, its field taking the keys from the start.
    pub fn new(desk: Desk) -> Ask {
        let mut field = PField::new(PRect::default());
        field.focused = true;
        let send = PButton::new(desk.text.get("ask.send"), PRect::default());
        Ask { frame: PRect::default(), desk, field, send }
    }

    /// Say what the field holds, and empty it. `false` when it is blank,
    /// or an answer is still being written.
    fn say(&mut self) -> bool {
        let said = self.desk.say(&self.field.value());
        if said {
            self.field.store(String::new());
        }
        said
    }

    /// Stop the answer that is being written. `false` when none is.
    fn stop(&mut self) -> bool {
        self.desk.history.borrow_mut().stop()
    }
}

impl PDraw for Ask {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        show(&self.field, paint, origin);
        show(&self.send, paint, origin);
        // An empty field says what it is for.
        if self.field.value().is_empty() {
            let (frame, dim) = (self.field.frame(), paint.color(DIM));
            let at = origin.plus(frame.origin).plus(inside(paint, frame.size.height));
            paint.text(at, self.desk.text.get("ask.hint"), dim);
        }
    }
}

impl PView for Ask {
    fn frame(&self) -> PRect {
        self.frame
    }

    fn set_frame(&mut self, frame: PRect) {
        let (gap, button) = (self.desk.size(GAP), self.desk.size(BUTTON));
        let (start, wide) = self.desk.column(frame.size.width);
        let height = frame.size.height - gap * 2.0;
        self.frame = frame;
        // The field and the button fill the column, a gap between them.
        self.field.set_frame(PRect::new(start, gap, wide - gap - button, height));
        self.send.set_frame(PRect::new(start + wide - button, gap, button, height));
    }

    /// The button says what a press on it does now.
    fn tick(&mut self) -> bool {
        let label = match self.desk.history.borrow().waiting() {
            true => "ask.stop",
            false => "ask.send",
        };
        self.send.label = self.desk.text.get(label).to_string();
        false
    }

    fn respond(&mut self, event: &PEvent) -> bool {
        match *event {
            PEvent::KeyDown(PEvent::ENTER) => self.say(),
            PEvent::KeyDown(PEvent::ESCAPE) => self.stop(),
            PEvent::KeyDown(_) => self.field.respond(event),
            _ => {
                let pressed = self.send.value();
                let taken = send(&mut self.send, event) || send(&mut self.field, event);
                let waiting = self.desk.history.borrow().waiting();
                match (self.send.value() != pressed, waiting) {
                    (true, true) => self.stop(),
                    (true, false) => self.say(),
                    (false, _) => taken,
                }
            }
        }
    }
}
