//! A deed: what a press on the window's own furniture does. The same few
//! deeds serve every app; which item of a bar does which is the app's
//! data.

use std::cell::RefCell;
use std::rc::Rc;

use super::{PLevel, PSpace, PWindow};

/// Deeds a part of the app's asks the window for: a press on a chat's
/// title wishes its space shown. The app hands the stage the list, its
/// parts add to it, and the stage does them after each event.
pub type PWish = Rc<RefCell<Vec<PDeed>>>;

/// What a press on a bar, a tab or a modal's panel does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PDeed {
    /// Show the space of that name.
    Show(String),
    /// Bring the host of that name in front, in whichever space holds it.
    Front(String),
    /// Lay the modal of that name over the space.
    Open(String),
    /// Take the open modal away.
    Close,
    /// Make the open modal as large as the window, or a square again.
    Expand,
    /// In a hand, draw the bar's menu out over the space, or put it away
    /// again.
    Side,
    /// At a desk, put the bar's menu away, or bring it back.
    Fold,
    /// Show the menu of that row of the bar: at a desk beside the bar, in
    /// a hand drawn out over the space.
    Pick(String),
    /// The area of the space shown that holds the first host holds the
    /// second instead: an area holds one host at a time, and which one
    /// can change.
    Hold(String, String),
    /// Tell the app a word: something only it can do, such as `begin`.
    /// The first of its views that takes the word does it.
    Tell(String),
    /// Put that text where the system keeps what was copied. This and
    /// the two after it are errands: the window cannot do them, the
    /// stage keeps them, and the screen that shows it does them.
    Copy(String),
    /// Open the file at that path in the system's own program for it.
    Launch(String),
    /// Ask the person, in the system's own dialog, where a copy of the
    /// file at that path is to be kept, and keep one there.
    Save(String),
    /// Ask the person, in the system's own dialog, for a file. The app
    /// is told this word with a space and the file's path after it;
    /// nothing, when they choose none.
    Choose(String),
}

impl PDeed {
    /// The deed a table names by its kind and its target: `space chat`,
    /// `host history`, `modal profile`.
    pub fn named(kind: &str, target: &str) -> Option<PDeed> {
        match kind {
            "space" => Some(PDeed::Show(target.to_string())),
            "host" => Some(PDeed::Front(target.to_string())),
            "modal" => Some(PDeed::Open(target.to_string())),
            "side" => Some(PDeed::Side),
            "fold" => Some(PDeed::Fold),
            "tell" => Some(PDeed::Tell(target.to_string())),
            _ => None,
        }
    }
}

impl PWindow {
    /// Do a deed. `false`, and nothing changes, when the window has no
    /// space or host of that name, for a word, which is the app's to do,
    /// and for an errand, which is the system's. Going to a space or a host puts the menu away, in a hand.
    pub fn apply(&mut self, deed: &PDeed) -> bool {
        match deed {
            PDeed::Show(space) => {
                self.drawer = false;
                self.show(space)
            }
            PDeed::Front(host) => {
                self.drawer = false;
                self.reach(host)
            }
            PDeed::Side => {
                self.drawer = !self.drawer;
                true
            }
            PDeed::Fold => {
                self.fold = !self.fold;
                true
            }
            PDeed::Pick(row) => {
                (self.pick, self.fold, self.drawer) = (row.clone(), false, true);
                true
            }
            PDeed::Tell(_) | PDeed::Copy(_) | PDeed::Launch(_) | PDeed::Save(_) | PDeed::Choose(_) => false,
            PDeed::Hold(from, to) => {
                let area = self.space.get_mut(self.active).into_iter().flat_map(|space| &mut space.zone).flat_map(|zone| &mut zone.area);
                let mut held = 0;
                area.filter(|area| area.host == *from).for_each(|area| {
                    area.host = to.clone();
                    held += 1;
                });
                held > 0
            }
            PDeed::Open(modal) => {
                self.open(modal);
                true
            }
            PDeed::Close => {
                self.close();
                true
            }
            PDeed::Expand => {
                self.wide = !self.wide;
                true
            }
        }
    }

    /// Bring a host in front, showing the first space that holds it when
    /// the space shown does not.
    fn reach(&mut self, host: &str) -> bool {
        let holds = |space: &PSpace| space.zone.iter().flat_map(|zone| &zone.area).any(|area| area.host == host);
        let other = self.space.iter().find(|space| holds(space)).map(|space| space.slug().to_string());
        match (self.current().is_some_and(holds), other) {
            (true, _) => {}
            (false, Some(space)) => _ = self.show(&space),
            (false, None) => return false,
        }
        self.index(host).is_some_and(|index| self.bring(index))
    }

    /// Whether a deed is what the window shows now: its space is the one
    /// shown, its host the one in front, its modal the one open, its row
    /// the one picked. A bar lights the item whose deed it is.
    pub fn shows(&self, deed: &PDeed) -> bool {
        match deed {
            PDeed::Show(space) => self.current().is_some_and(|shown| shown.slug() == space),
            PDeed::Front(host) => self.index(host) == Some(self.front()),
            PDeed::Open(modal) => self.modal() == Some(modal.as_str()),
            PDeed::Side => self.drawer,
            PDeed::Pick(row) => self.pick == *row,
            PDeed::Close | PDeed::Expand | PDeed::Fold | PDeed::Tell(_) | PDeed::Hold(..) | PDeed::Copy(_) | PDeed::Launch(_) | PDeed::Save(_) | PDeed::Choose(_) => false,
        }
    }
}
