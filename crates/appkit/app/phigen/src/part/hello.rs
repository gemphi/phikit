//! The hello: what a chat nothing was said in yet shows over the field,
//! in the middle of the window: one heading that asks what to work on.

use appkit::pui::core::{PDraw, PPaint, PPoint, PRect, PView};

use super::{Desk, GAP, INK};

/// One heading, in the middle, at the foot of its band.
pub struct Hello {
    frame: PRect,
    desk: Desk,
}

impl Hello {
    pub fn new(desk: Desk) -> Hello {
        Hello { frame: PRect::default(), desk }
    }
}

impl PDraw for Hello {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        let (gap, line, ink) = (paint.size(GAP), paint.size("head.line"), paint.color(INK));
        let text = self.desk.text.get("hello");
        let left = origin.x + (self.frame.size.width - paint.measure_head(text)) / 2.0;
        paint.head(PPoint::new(left, origin.y + self.frame.size.height - gap - line), text, ink);
    }
}

impl PView for Hello {
    fn frame(&self) -> PRect {
        self.frame
    }

    fn set_frame(&mut self, frame: PRect) {
        self.frame = frame;
    }
}
