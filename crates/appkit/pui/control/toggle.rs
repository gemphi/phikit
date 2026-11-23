//! A toggle: on or off, and a press flips it.

use super::{ACCENT, FILL, PAction, PControl};
use crate::pui::core::{PDraw, PEvent, PPaint, PPoint, PRect, PView};

/// On or off; a press flips it.
pub struct PToggle {
    frame: PRect,
    on: bool,
    enabled: bool,
    action: Option<PAction<bool>>,
}

impl PToggle {
    pub fn new(frame: PRect) -> PToggle {
        PToggle { frame, on: false, enabled: true, action: None }
    }
}

impl PDraw for PToggle {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        let token = match self.on {
            true => ACCENT,
            false => FILL,
        };
        let color = paint.color(token);
        paint.fill(PRect { origin, size: self.frame.size }, color);
    }
}

impl PView for PToggle {
    fn frame(&self) -> PRect {
        self.frame
    }

    fn set_frame(&mut self, frame: PRect) {
        self.frame = frame;
    }

    fn respond(&mut self, event: &PEvent) -> bool {
        match event {
            PEvent::PointerDown(_) => self.change(!self.on),
            _ => false,
        }
    }
}

impl PControl for PToggle {
    type Value = bool;

    fn value(&self) -> bool {
        self.on
    }

    fn store(&mut self, value: bool) {
        self.on = value;
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn action(&mut self) -> Option<&mut PAction<bool>> {
        self.action.as_mut()
    }

    fn set_action(&mut self, action: PAction<bool>) {
        self.action = Some(action);
    }
}
