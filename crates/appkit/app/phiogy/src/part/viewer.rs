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

mod lay;

use std::cell::{Cell, RefCell};

use appkit::pui::core::{PEvent, PPoint, PRect, PView};
use appkit::pui::window::PDeed;

use super::{Desk, Tool};
use crate::data;
use crate::hosted::Hosted;

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
    /// Where the picture itself lies: a press there is not beside it.
    shown: Cell<PRect>,
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
}

impl Viewer {
    pub fn new(desk: Desk) -> Viewer {
        Viewer { frame: PRect::default(), desk, tool: Tool::read(data::TOOL), origin: Cell::default(), spot: RefCell::default(), shown: Cell::default(), seen: Cell::default(), large: Cell::default(), told: Cell::default(), shift: Cell::default(), reach: Cell::default(), over: Cell::default(), live: Cell::default() }
    }

    /// Show the picture so many after the one shown, if there is one.
    fn step(&self, by: isize) -> bool {
        let count = self.desk.now.album.borrow().len() as isize;
        let next = self.desk.view.get().map(|now| now as isize + by).filter(|next| (0..count).contains(next));
        next.into_iter().for_each(|next| self.desk.view.set(Some(next as usize)));
        next.is_some()
    }

    /// Where the file of the picture shown is, as a person reads it.
    fn file(&self) -> Option<String> {
        let list = self.desk.now.album.borrow();
        Hosted::own().place(&list.get(self.desk.view.get()?)?.key)
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
        }
    }
}

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
            PEvent::KeyDown(PEvent::LEFT) => self.step(-1),
            PEvent::KeyDown(PEvent::RIGHT) => self.step(1),
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
                match (under(at), self.shown.get().contains(at.plus(self.origin.get()))) {
                    (Some(act), _) => self.act(act),
                    (None, true) => self.zoom(),
                    // A press beside the picture puts the viewer away.
                    (None, false) => self.act(Act::Close),
                }
                true
            }
            _ => false,
        }
    }
}
