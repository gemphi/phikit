//! The title: what is over a chat. The pick, which names the model that
//! answers, and after it what the chat is called, over a line. A chat
//! nothing was said in has no name yet, and no line: the pick alone.

use appkit::pui::core::{PDraw, PEvent, PPaint, PPoint, PRect, PSize, PView};

use super::{Desk, GAP, INK, LINE, Pick, fit};

/// The theme's tokens for the line under the title.
pub const RULE: &str = "line.ink";
pub const WIDTH: &str = "line.width";

/// What is over a chat.
pub struct Title {
    frame: PRect,
    desk: Desk,
    pick: Pick,
}

impl Title {
    /// The title, its pick opening the modal of that name.
    pub fn new(desk: Desk, modal: &str) -> Title {
        Title { frame: PRect::default(), pick: Pick::new(desk.clone(), modal), desk }
    }
}

impl PDraw for Title {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        let (gap, line, rule, size) = (paint.size(GAP), paint.size(LINE), paint.size(WIDTH), self.frame.size);
        let (ink, ruled) = (paint.color(INK), paint.color(RULE));
        // The pick starts where the turns under it do; the name follows.
        let (start, wide) = self.desk.column(size.width);
        let used = self.pick.draw(paint, origin.plus(PPoint::new(start, size.height / 2.0)));
        let history = self.desk.history.borrow();
        let Some(title) = history.current().title() else { return };
        let shown = fit(title, wide - used - gap, paint);
        paint.text(origin.plus(PPoint::new(start + used + gap, (size.height - line) / 2.0)), &shown, ink);
        paint.fill(PRect { origin: origin.plus(PPoint::new(0.0, size.height - rule)), size: PSize { width: size.width, height: rule } }, ruled);
    }
}

impl PView for Title {
    fn frame(&self) -> PRect {
        self.frame
    }

    fn set_frame(&mut self, frame: PRect) {
        self.frame = frame;
    }

    fn respond(&mut self, event: &PEvent) -> bool {
        match *event {
            // The pick keeps where it was drawn in the window.
            PEvent::PointerDown(at) => self.pick.press(at.plus(self.frame.origin)),
            _ => false,
        }
    }
}
