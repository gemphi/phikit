//! A field: a line of text a hand types into. A press gives it the keys.
//! A key adds its character where the caret stands; backspace takes away
//! the one before it and delete the one after; the arrows, Home and End
//! move it. Nothing is selected yet, and a press does not place it.
//!
//! `send` gives a key to the view it is called on, so a field inside
//! another view gets its keys only when the app sends them to it: who is
//! listening is not kept anywhere yet.

use super::{ACCENT, CARET, FILL, INK, LINE, PAction, PControl, RADIUS, ROW, inside};
use crate::pui::core::light::surface;
use crate::pui::core::{PDraw, PEvent, PLight, PPaint, PPoint, PRect, PSize, PView};

/// The theme's word for the mark drawn in place of each character of
/// a secret.
pub const MASK: &str = "control.mask";

/// A line of text a hand types into.
pub struct PField {
    frame: PRect,
    text: String,
    /// How many characters stand before the caret.
    caret: usize,
    enabled: bool,
    /// Whether it takes the keys: set by a press on it.
    pub focused: bool,
    /// Whether it draws no ground of its own: it stands in a box that
    /// something else drew.
    pub bare: bool,
    /// Whether what is typed is kept from sight: a mark is drawn for
    /// each character, the theme's `control.mask`.
    pub secret: bool,
    action: Option<PAction<String>>,
}

impl PField {
    pub fn new(frame: PRect) -> PField {
        PField { frame, text: String::new(), caret: 0, enabled: true, focused: false, bare: false, secret: false, action: None }
    }

    /// How many characters stand before the caret.
    pub fn caret(&self) -> usize {
        self.caret
    }

    /// The text with one character put in or taken out at a place, and
    /// the caret moved there after it. `false` when nothing changed.
    fn edit(&mut self, at: usize, put: Option<char>, caret: usize) -> bool {
        let byte = self.text.char_indices().nth(at).map_or(self.text.len(), |(byte, _)| byte);
        let mut text = self.text.clone();
        match put {
            Some(letter) => text.insert(byte, letter),
            None => _ = text.remove(byte),
        }
        let changed = self.change(text);
        self.caret = caret;
        changed
    }

    /// One key: `true` when the field did something with it.
    fn key(&mut self, key: char) -> bool {
        let count = self.text.chars().count();
        let at = self.caret.min(count);
        match key {
            PEvent::LEFT => {
                self.caret = at.saturating_sub(1);
                at > 0
            }
            PEvent::RIGHT => {
                self.caret = (at + 1).min(count);
                at < count
            }
            PEvent::HOME => {
                self.caret = 0;
                true
            }
            PEvent::END => {
                self.caret = count;
                true
            }
            PEvent::BACKSPACE => at > 0 && self.edit(at - 1, None, at - 1),
            PEvent::DELETE => at < count && self.edit(at, None, at),
            // A key that writes nothing, and is none of the above.
            key if key.is_control() || PEvent::silent(key) => false,
            key => self.edit(at, Some(key), at + 1),
        }
    }
}

impl PDraw for PField {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        let (at, line, size, radius) = (inside(paint, self.frame.size.height), paint.size(LINE), self.frame.size, paint.size(RADIUS));
        let (ink, accent) = (paint.color(INK), paint.color(ACCENT));
        // A field wears the surface, as a card does.
        if !self.bare {
            surface(paint, PRect { origin, size }, radius, PLight::Rest, FILL);
        }
        let mark = paint.theme().word(MASK).unwrap_or("*").to_string();
        // A secret is drawn as a mark for each of its characters.
        let seen = |text: &str| match self.secret {
            true => mark.repeat(text.chars().count()),
            false => text.to_string(),
        };
        paint.text(origin.plus(at), &seen(&self.text), ink);
        // The field that takes the keys is underlined, between its corners.
        if self.focused {
            let under = PRect { origin: origin.plus(PPoint::new(radius, size.height - line)), size: PSize { width: size.width - radius * 2.0, height: line } };
            paint.fill(under, accent);
            // And has a caret where the next character goes, as high as
            // a line of text, in the room just before that character: so
            // it never lies over a letter.
            let before: String = seen(&self.text.chars().take(self.caret).collect::<String>());
            let high = paint.theme().size(ROW).unwrap_or(size.height - at.y * 2.0);
            let wide = paint.theme().size(CARET).unwrap_or(line);
            let caret = PRect::new(origin.x + at.x + paint.measure(&before) - wide, origin.y + at.y, wide, high);
            paint.fill(caret, accent);
        }
    }
}

impl PView for PField {
    fn frame(&self) -> PRect {
        self.frame
    }

    fn set_frame(&mut self, frame: PRect) {
        self.frame = frame;
    }

    fn respond(&mut self, event: &PEvent) -> bool {
        match (event, self.focused) {
            (PEvent::PointerDown(_), _) => {
                self.focused = self.enabled;
                self.enabled
            }
            (PEvent::KeyDown(key), true) => self.key(*key),
            _ => false,
        }
    }
}

impl PControl for PField {
    type Value = String;

    fn value(&self) -> String {
        self.text.clone()
    }

    /// Keep a text, with the caret after it.
    fn store(&mut self, value: String) {
        self.caret = value.chars().count();
        self.text = value;
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn action(&mut self) -> Option<&mut PAction<String>> {
        self.action.as_mut()
    }

    fn set_action(&mut self, action: PAction<String>) {
        self.action = Some(action);
    }
}
