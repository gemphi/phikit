//! The viewer's pens: what a press on the picture does with one, the
//! marks of the picture shown (read from what was kept the first time it
//! is shown), where a place in the window lies on the whole picture, and
//! the keys given to words being set.

use appkit::pui::core::{PEvent, PPoint};

use super::{Mark, Sheet, Viewer};

/// What a press on the picture does: sketch a line, or set words.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pen {
    Sketch,
    Text,
}

impl Viewer {
    /// The key of the picture shown.
    pub(super) fn key(&self) -> Option<String> {
        let list = self.desk.now.album.borrow();
        list.get(self.desk.view.get()?).map(|shot| shot.key.clone())
    }

    /// The marks of the picture shown, read from what was kept the first
    /// time it is shown.
    pub(super) fn marks(&self) -> std::cell::RefMut<'_, Sheet> {
        let key = self.key().unwrap_or_default();
        let mut sheet = self.sheet.borrow_mut();
        if sheet.0 != key {
            *sheet = (key.clone(), Sheet::kept(&self.desk.choice.get(&format!("mark/{key}"))));
        }
        std::cell::RefMut::map(sheet, |(_, sheet)| sheet)
    }

    /// A place in the window as a share of the whole picture's width and
    /// height.
    pub(super) fn share(&self, at: PPoint) -> PPoint {
        let whole = self.whole.get();
        let part = |by: f32, from: f32, wide: f32| ((by - from) / wide.max(1.0)).clamp(0.0, 1.0);
        PPoint::new(part(at.x, whole.origin.x, whole.size.width), part(at.y, whole.origin.y, whole.size.height))
    }

    /// A key given to the words being set: a letter more, one less, or
    /// done (Enter); words left empty are no mark.
    pub(super) fn write(&self, key: char) -> bool {
        let mut sheet = self.marks();
        let Some(Mark::Word { text, .. }) = sheet.last() else { return false };
        match key {
            PEvent::ENTER => self.writing.set(false),
            PEvent::BACKSPACE => _ = text.pop(),
            key if PEvent::silent(key) || key.is_control() => {}
            key => text.push(key),
        }
        let empty = text.is_empty();
        if empty && !self.writing.get() {
            sheet.undo();
        }
        true
    }
}
