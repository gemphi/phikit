//! A button: a label, in its middle, a hand presses. What it holds is how many times it
//! has been pressed, so every press is a change and sends its action.

use super::{FILL, INK, LABEL, PAction, PControl, PRESS, RADIUS, inside};
use crate::pui::core::light::{LIGHT, ink, label};
use crate::pui::core::{PDraw, PEvent, PLight, PPaint, PPoint, PRect, PView};

/// A label a hand presses.
pub struct PButton {
    frame: PRect,
    pub label: String,
    pressed: u32,
    enabled: bool,
    action: Option<PAction<u32>>,
}

impl PButton {
    pub fn new(label: &str, frame: PRect) -> PButton {
        PButton { frame, label: label.to_string(), pressed: 0, enabled: true, action: None }
    }
}

impl PDraw for PButton {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        let (at, radius) = (inside(paint, self.frame.size.height), paint.size(RADIUS));
        // A button wears the label's light at its strongest, and the
        // theme's ink; in a flat theme, the fill and ink it names.
        let lit = paint.theme().color(LIGHT).is_some();
        let named = |token: &'static str, other: &'static str| match lit || paint.theme().color(token).is_some() {
            true => token,
            false => other,
        };
        let (fill, text) = (named(PRESS, FILL), named(LABEL, INK));
        let ink = ink(paint, text);
        label(paint, PRect { origin, size: self.frame.size }, radius, PLight::Strong, false, fill);
        // The label is in the middle. One too long for that, or drawn
        // where nothing measures it, starts at the inset.
        let wide = paint.measure(&self.label);
        let left = match wide > 0.0 {
            true => ((self.frame.size.width - wide) / 2.0).max(at.x),
            false => at.x,
        };
        paint.text(origin.plus(PPoint::new(left, at.y)), &self.label, ink);
    }
}

impl PView for PButton {
    fn frame(&self) -> PRect {
        self.frame
    }

    fn set_frame(&mut self, frame: PRect) {
        self.frame = frame;
    }

    fn respond(&mut self, event: &PEvent) -> bool {
        match event {
            PEvent::PointerDown(_) => self.change(self.pressed + 1),
            _ => false,
        }
    }
}

impl PControl for PButton {
    /// How many times it has been pressed.
    type Value = u32;

    fn value(&self) -> u32 {
        self.pressed
    }

    fn store(&mut self, value: u32) {
        self.pressed = value;
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn action(&mut self) -> Option<&mut PAction<u32>> {
        self.action.as_mut()
    }

    fn set_action(&mut self, action: PAction<u32>) {
        self.action = Some(action);
    }
}
