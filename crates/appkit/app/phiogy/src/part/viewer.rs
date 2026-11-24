//! The viewer: a picture that was posted, looked at alone, with the
//! others it was opened among (a room's, a person's). It is the
//! web's (`puikit/src/components/overlays/Lightbox`), over the whole
//! window with nothing of a panel's about it:
//!
//! * who posted it and when at the top left;
//! * its tools at the top right, on one surface, and the way out
//!   beside them: larger and whole again, share, save, open;
//! * the picture whole in the middle, never cut and never larger than
//!   it is; a press on it shows it larger, moved about with the wheel,
//!   and another whole again;
//! * the one before and the one after at its sides, and with the arrow
//!   keys; neither past an end;
//! * at the foot a strip of every picture of the room, the one shown
//!   marked; a press on one shows it.
//!
//! A press beside the picture puts the viewer away, as Escape and its
//! close do. In a narrow window who and when stand over the strip and
//! the picture has the width. What a control is called is shown by it
//! while the pointer is over it.
//!
//! It is the content of the modal `view`, whose shape is the whole
//! window and whose panel is bare (`data/modal.tsv`). Its controls,
//! their icons and their words are `data/tool.tsv`; `lay.rs` draws it.

mod foot;
mod lay;
mod mark;
mod pen;
mod view;

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::path::PathBuf;

use appkit::pui::core::{PPoint, PRect};
use appkit::pui::window::PDeed;

use super::{Desk, Tool};
use crate::data;
use crate::hosted::Hosted;
pub use mark::{Mark, Sheet};
pub use pen::Pen;

/// The theme's tokens: the face of who posted it, a picture of the
/// strip, the square a round control stands in, a tool, how thick the
/// mark around the picture shown is in the strip, and how many times
/// the room's width a picture shown larger is.
pub const FACE: &str = "view.face";
pub const THUMB: &str = "view.thumb";
pub const CONTROL: &str = "view.control";
pub const TOOL: &str = "view.tool";
pub const MARK: &str = "view.mark";
pub const LARGE: &str = "view.large";
/// The token under which the room's people stand beside its stream:
/// narrower than that, who and when stand over the strip.
pub const SPLIT: &str = "room.split";
/// The rows of `data/tool.tsv`, by what each does.
pub const ZOOM: &str = "zoom";
pub const SHARE: &str = "share";
pub const SAVE: &str = "save";
pub const OPEN: &str = "open";
pub const CLOSE: &str = "close";
pub const BEFORE: &str = "before";
pub const AFTER: &str = "after";
pub const UNDO: &str = "undo";
pub const REDO: &str = "redo";
pub const KEEP: &str = "keep";
pub const SKETCH: &str = "sketch";
pub const TEXT: &str = "text";
/// The theme's tokens: how thick a sketched line is, how much of the
/// window's ground lies over the picture's haze, and an ink's dot.
pub const PEN: &str = "view.pen";
pub const VEIL: &str = "view.veil";
pub const DOT: &str = "view.dot";


/// What a press on a control does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Act {
    /// Show the picture of that number.
    Show(usize),
    /// Show the one before, and the one after.
    Before,
    After,
    Zoom,
    Share,
    Save,
    Open,
    Close,
    /// Take the last mark back, put it back, keep the marks.
    Undo,
    Redo,
    Keep,
    /// Mark with that pen, or press the picture as before again.
    Pen(Pen),
    /// Mark in the ink of that number.
    Ink(usize),
}

impl Act {
    /// The tool a row of the table names.
    pub fn named(name: &str) -> Option<Act> {
        match name {
            ZOOM => Some(Act::Zoom),
            SHARE => Some(Act::Share),
            SAVE => Some(Act::Save),
            OPEN => Some(Act::Open),
            CLOSE => Some(Act::Close),
            UNDO => Some(Act::Undo),
            REDO => Some(Act::Redo),
            KEEP => Some(Act::Keep),
            SKETCH => Some(Act::Pen(Pen::Sketch)),
            TEXT => Some(Act::Pen(Pen::Text)),
            _ => None,
        }
    }
}

/// A control as it was drawn last: where, what it does, what it is
/// called.
#[derive(Debug, Clone, PartialEq)]
pub struct Spot {
    pub rect: PRect,
    pub act: Act,
    pub word: String,
}

/// A picture of the room, looked at alone.
pub struct Viewer {
    frame: PRect,
    desk: Desk,
    /// Its controls: the rows of `data/tool.tsv`.
    tool: Tool,
    /// Where it was drawn last, and its controls.
    origin: Cell<PPoint>,
    spot: RefCell<Vec<Spot>>,
    /// Where the picture itself lies: a press there is not beside it;
    /// and where the whole of it is, larger than its room when it is.
    shown: Cell<PRect>,
    whole: Cell<PRect>,
    /// The picture the rest is true of: another is seen whole first.
    seen: Cell<Option<usize>>,
    large: Cell<bool>,
    /// Whether where its file is was copied.
    told: Cell<bool>,
    /// How far the larger picture is moved, and how far it can be.
    shift: Cell<PPoint>,
    reach: Cell<PPoint>,
    /// The control the pointer is over.
    over: Cell<Option<Act>>,
    /// Whether it was drawn since the last frame began: put away, it
    /// is not, and the next time it is opened it begins afresh.
    live: Cell<bool>,
    /// The pen, the ink, the marks of the picture shown and its key, and
    /// whether a line is being drawn or words written.
    pen: Cell<Option<Pen>>,
    ink: Cell<usize>,
    sheet: RefCell<(String, Sheet)>,
    drawing: Cell<bool>,
    writing: Cell<bool>,
    /// The pictures whose haze was asked for.
    hazed: RefCell<HashSet<String>>,
}

impl Viewer {
    pub fn new(desk: Desk) -> Viewer {
        let (pen, ink, sheet, drawing, writing, hazed) = (Cell::default(), Cell::default(), RefCell::default(), Cell::default(), Cell::default(), RefCell::default());
        Viewer { frame: PRect::default(), desk, tool: Tool::read(data::TOOL), origin: Cell::default(), spot: RefCell::default(), shown: Cell::default(), whole: Cell::default(), seen: Cell::default(), large: Cell::default(), told: Cell::default(), shift: Cell::default(), reach: Cell::default(), over: Cell::default(), live: Cell::default(), pen, ink, sheet, drawing, writing, hazed }
    }

    /// Show the picture so many after the one shown, if there is one.
    fn step(&self, by: isize) -> bool {
        let count = self.desk.now.album.borrow().len() as isize;
        let next = self.desk.view.get().map(|now| now as isize + by).filter(|next| (0..count).contains(next));
        next.into_iter().for_each(|next| self.desk.view.set(Some(next as usize)));
        next.is_some()
    }

    /// Where the file of the picture shown is, as a person reads it:
    /// one of the person's own where it lies, a posted one where the web
    /// keeps it.
    fn file(&self) -> Option<String> {
        self.path().map(|path| path.to_string_lossy().to_string())
    }

    fn path(&self) -> Option<PathBuf> {
        let list = self.desk.now.album.borrow();
        let shot = list.get(self.desk.view.get()?)?;
        shot.file.clone().or_else(|| Hosted::own().place(&shot.key).map(PathBuf::from))
    }

    /// Larger, or whole again: from its top left either way. It is
    /// true of the picture shown now, and of a viewer that is open.
    fn zoom(&self) {
        self.seen.set(self.desk.view.get());
        self.live.set(true);
        self.large.set(!self.large.get());
        self.shift.set(PPoint::default());
    }

    /// Do what a control does. What only the system can do is wished
    /// for: appkit's screen does it.
    fn act(&self, act: Act) {
        let wish = |deed: Option<PDeed>| self.desk.wish.borrow_mut().extend(deed);
        match act {
            Act::Show(index) => self.desk.view.set(Some(index)),
            Act::Before => _ = self.step(-1),
            Act::After => _ = self.step(1),
            Act::Zoom => self.zoom(),
            Act::Share => {
                let file = self.file();
                self.told.set(file.is_some());
                wish(file.map(PDeed::Copy));
            }
            Act::Save => wish(self.file().map(PDeed::Save)),
            Act::Open => wish(self.file().map(PDeed::Launch)),
            Act::Close => wish(Some(PDeed::Close)),
            Act::Undo => _ = self.marks().undo(),
            Act::Redo => _ = self.marks().redo(),
            Act::Keep => {
                let value = self.marks().value();
                self.desk.choice.put(&format!("mark/{}", self.key().unwrap_or_default()), &value);
                self.marks().kept = true;
            }
            Act::Pen(pen) => {
                let next = Some(pen).filter(|pen| self.pen.get() != Some(*pen));
                self.pen.set(next);
                self.large.set(false);
            }
            Act::Ink(ink) => self.ink.set(ink),
        }
    }
}
