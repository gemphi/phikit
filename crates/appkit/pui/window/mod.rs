//! How a window is built. appkit creates the window and every level of it;
//! an app supplies its entries, as data, and the content of its parts.
//! `PWindow`, and `PLevel`, the trait the levels below share: a set
//! of entries, each with a stable name, a label and an icon.
//!
//! The levels and their meanings are Phiber's (`crates/phiber/src/space`
//! to `part`, and `level.rs`): a window shows a space; a space has zones;
//! a zone has areas; an area holds a host; a host has regions; a region
//! has panes; a pane holds parts. A modal lies over the space.

pub mod area;
mod deed;
mod frame;
pub mod host;
pub mod modal;
pub mod pane;
mod panel;
pub mod part;
mod place;
pub mod region;
mod set;
pub mod space;
mod stage;
pub mod zone;

pub use area::PArea;
pub use deed::{PDeed, PWish};
pub use frame::PFrame;
pub use host::{PHost, PSlot};
pub use modal::{PModal, PShape};
pub use pane::{PPane, PSide};
pub use panel::PPanel;
pub use part::{PActivity, PBar, PCaption, PFact, PGrip, PGroup, PItem, PPart, PProfile, PRail, PSeat, PTab};
pub use place::PPlace;
pub use region::PRegion;
pub use set::{PEntry, PSet};
pub use space::PSpace;
pub use stage::PStage;
pub use zone::PZone;

/// A level an app supplies entries of: a space, a zone, a host, a part, a
/// modal. Phiber's `BLevelType`, with the entry as data.
pub trait PLevel {
    /// The level's word: `space`, `host`.
    const WORD: &'static str;

    /// An entry of this level with nothing in it yet.
    fn of(entry: PEntry) -> Self;

    fn entry(&self) -> &PEntry;

    /// Take what a table says of an entry after its icon. Nothing, for a
    /// level that has no more to say.
    fn more(&mut self, _cell: &[&str]) {}

    /// The stable name: what a saved layout keeps.
    fn slug(&self) -> &str {
        &self.entry().slug
    }

    fn label(&self) -> &str {
        &self.entry().label
    }

    fn icon(&self) -> &str {
        &self.entry().icon
    }
}

/// A top-level surface. It shows one space at a time, and a modal over it
/// when one is open.
#[derive(Debug, Clone, PartialEq)]
pub struct PWindow {
    pub title: String,
    /// The spaces it can show, as its tabs, in order.
    pub space: Vec<PSpace>,
    active: usize,
    /// On a mobile window, the area in front (`place.rs`).
    front: usize,
    modal: Option<String>,
    /// Whether the open modal is as large as the window (`panel.rs`).
    wide: bool,
    /// Whether the bar's menu is drawn out over the space, in a hand.
    drawer: bool,
    /// Whether the bar's menu is put away, at a desk.
    fold: bool,
    /// The row of the bar whose menu is the one to show.
    pick: String,
}

impl PWindow {
    /// A window on these spaces, showing the first.
    pub fn new(title: &str, space: Vec<PSpace>) -> PWindow {
        PWindow { title: title.to_string(), space, active: 0, front: 0, modal: None, wide: false, drawer: false, fold: false, pick: String::new() }
    }

    /// The space it shows.
    pub fn current(&self) -> Option<&PSpace> {
        self.space.get(self.active)
    }

    /// Show the space of that name. `false`, and nothing changes, when the
    /// window has none.
    pub fn show(&mut self, slug: &str) -> bool {
        let found = self.space.iter().position(|space| space.slug() == slug);
        found.inspect(|index| (self.active, self.front) = (*index, 0)).is_some()
    }

    /// Lay a modal over the space.
    pub fn open(&mut self, modal: &str) {
        self.modal = Some(modal.to_string());
    }

    /// Take the open modal away. The next one opens as a square again.
    pub fn close(&mut self) {
        (self.modal, self.wide) = (None, false);
    }

    /// Whether the open modal is as large as the window.
    pub fn wide(&self) -> bool {
        self.wide
    }

    /// Whether the bar's menu is drawn out over the space, in a hand.
    pub fn drawer(&self) -> bool {
        self.drawer
    }

    /// Whether the bar's menu is put away, at a desk.
    pub fn fold(&self) -> bool {
        self.fold
    }

    /// The row of the bar whose menu is the one to show.
    pub fn pick(&self) -> &str {
        &self.pick
    }

    /// Have that row's menu be the one to show, without drawing it out.
    pub fn choose(&mut self, row: &str) {
        self.pick = row.to_string();
    }

    /// The modal that is open, if one is.
    pub fn modal(&self) -> Option<&str> {
        self.modal.as_deref()
    }

    /// Every name the window uses that the app's sets do not have: a
    /// host an area holds, a part a host's pane holds, the modal that is
    /// open. Empty when the window can be built whole.
    pub fn missing(&self, host: &PSet<PHost>, part: &PSet<PPart>, modal: &PSet<PModal>) -> Vec<String> {
        let held = self.space.iter().flat_map(|space| &space.zone).flat_map(|zone| &zone.area).map(|area| area.host.as_str());
        let mut out: Vec<String> = held.filter(|slug| host.find(slug).is_none()).map(|slug| format!("{} {slug}", PHost::WORD)).collect();
        let shown = host.list().iter().flat_map(|host| &host.region).flat_map(|region| &region.pane).flat_map(|pane| &pane.part);
        out.extend(shown.filter(|slug| part.find(slug).is_none()).map(|slug| format!("{} {slug}", PPart::WORD)));
        out.extend(self.modal.iter().filter(|slug| modal.find(slug).is_none()).map(|slug| format!("{} {slug}", PModal::WORD)));
        out.sort();
        out.dedup();
        out
    }
}
