//! The tabs over the guide: Recent, Favorites, Older. With none chosen
//! the guide lists every place and room; a tab keeps the rooms it is
//! for, and a press on the chosen one lets them all back. Which tabs
//! there are is `data/app.tsv`'s (`guide`), and what a person chose is
//! kept with their other choices.
//!
//! * Recent: a room the person opened not long ago (`recent`, in hours,
//!   of `data/app.tsv`), or one where something waits for them.
//! * Favorites: a room they starred from its title, and the room of a
//!   person they follow.
//! * Older: every other room than the recent ones.
//!
//! When a room was opened, and which are starred, are documents of the
//! person's own (`data/keep.tsv`).

use appkit::pui::block::PBand;
use appkit::pui::core::{PPaint, PPoint, PRect};
use appkit::service::PTable;
use appkit::pui::state::{PState, flag};

use super::super::state::CHOICE;
use super::super::{Desk, GAP, LINE, Rung};
use crate::{data, hour};

/// What the chosen tab is kept under, and the tabs by their names.
pub const TAB: &str = "guide";
pub const RECENT: &str = "recent";
pub const FAVORITE: &str = "favorite";
/// The theme's token for how many hours ago a room was opened at most
/// for it to be a recent one.
pub const SPAN: &str = "guide.recent";
/// The person's own documents: when each room was opened last, and
/// the rooms they starred.
pub const SEEN: &str = "seen";
pub const STAR: &str = "star";
/// How many thousandths of a second an hour is.
const HOUR: f64 = 3_600_000.0;

impl Desk {
    /// The tabs there are, in order, and the one chosen: nothing when
    /// none is.
    pub fn tabs(&self) -> (Vec<String>, String) {
        let all: Vec<String> = PTable::read(data::APP).get(TAB).split_whitespace().map(str::to_string).collect();
        let chosen = Some(self.get(&format!("{CHOICE}/{TAB}"))).filter(|name| all.contains(name)).unwrap_or_default();
        (all, chosen)
    }

    /// Choose a tab; the chosen one again lets every room back.
    pub fn sift(&self, name: &str) {
        match self.tabs().1 == name {
            true => self.set(&format!("{CHOICE}/{TAB}"), ""),
            false => self.set(&format!("{CHOICE}/{TAB}"), name),
        }
    }

    /// Keep that a room was opened now.
    pub fn saw(&self, room: &str) {
        self.set(&format!("{SEEN}/{}", self.key(room)), &(hour::moment() as f64).to_string());
    }

    /// Whether a room is a recent one: opened within the hours the
    /// app's data gives, or with something waiting in it.
    pub fn lately(&self, rung: &Rung) -> bool {
        let span = f64::from(self.size(SPAN)) * HOUR;
        let opened = self.get(&format!("{SEEN}/{}", self.key(&rung.id))).parse::<f64>().ok();
        opened.is_some_and(|at| hour::moment() as f64 - at <= span) || rung.count.max(self.waiting(&rung.id)) > 0
    }

    /// Whether the person starred a room; and star it, or no more.
    pub fn starred(&self, room: &str) -> bool {
        self.has(&format!("{STAR}/{}", self.key(room)))
    }

    pub fn star(&self, room: &str, on: bool) {
        self.set(&format!("{STAR}/{}", self.key(room)), flag(on));
    }

    /// The rail as the chosen tab keeps it: the place open with the
    /// rooms the tab is for. A tab that keeps none of them leaves them
    /// all: a host opened is never an empty list (`unsifted` says so).
    pub fn sifted(&self) -> Vec<(String, Vec<Rung>)> {
        self.opened().into_iter().map(|(label, all, kept)| match kept.is_empty() {
            true => (label, all),
            false => (label, kept),
        }).filter(|(_, rung)| !rung.is_empty()).collect()
    }

    /// Whether the chosen tab keeps none of the open place's rooms, which
    /// stand all the same.
    pub fn unsifted(&self) -> bool {
        !self.tabs().1.is_empty() && self.opened().is_some_and(|(_, all, kept)| kept.is_empty() && !all.is_empty())
    }

    /// The place open: its name, every room of it, and those the chosen
    /// tab keeps.
    fn opened(&self) -> Option<(String, Vec<Rung>, Vec<Rung>)> {
        let chosen = self.tabs().1;
        let keeps = |rung: &Rung| match chosen.as_str() {
            "" => true,
            RECENT => self.lately(rung),
            FAVORITE => self.starred(&rung.id) || (!rung.with.is_empty() && self.follows(&rung.with)),
            _ => !self.lately(rung),
        };
        // A sub-channel that is kept keeps the channels it stands under,
        // so it is never shown without them.
        let kept = |rung: Vec<Rung>| {
            let mut out: Vec<Rung> = Vec::new();
            let mut above: Vec<Rung> = Vec::new();
            for one in rung {
                above.truncate(one.depth as usize);
                if keeps(&one) {
                    let missing: Vec<Rung> = above.iter().filter(|over| !out.iter().any(|known| known.id == over.id)).cloned().collect();
                    out.extend(missing);
                    out.push(one.clone());
                }
                above.push(one);
            }
            out
        };
        // One place at a time: the place whose room is open, which the
        // rail's row chose; a host not in the rail (opened from the hosts
        // or a search) is shown all the same; where the open room is in no
        // place, the first (the direct list). A tab sifts that place's
        // rooms.
        let all = self.rail();
        let open = self.open.borrow().clone();
        let host = || self.host_of(&open).and_then(|(id, label)| self.site.venue.iter().find(|venue| venue.id == id).map(|venue| (label, venue.plate.iter().map(Rung::of).collect::<Vec<_>>())));
        let place = match all.iter().position(|(_, rung)| rung.iter().any(|rung| rung.id == open)) {
            Some(at) => all.into_iter().nth(at),
            None => host().or_else(|| all.into_iter().next()),
        };
        place.map(|(label, rung)| (label, rung.clone(), kept(rung)))
    }
}

impl super::Guide {
    /// How high the tabs over the rows are: a row and a little under it.
    pub(super) fn head(&self) -> f32 {
        self.desk.size(LINE) + self.desk.size(GAP) * 1.25
    }

    /// Draw the tabs, the kit's group of pills, from a place: where
    /// each stands, with its name.
    pub(super) fn tabs(&self, paint: &mut dyn PPaint, origin: PPoint) -> Vec<(PRect, String)> {
        let (all, chosen) = self.desk.tabs();
        let said: Vec<&str> = all.iter().map(|name| self.desk.text.get(&format!("guide.tab.{name}"))).collect();
        let pill: Vec<(&str, &str, bool)> = said.iter().zip(&all).map(|(said, name)| ("", *said, *name == chosen)).collect();
        // In the list's own width, a quarter of a gap in from each side.
        let apart = paint.size(GAP) / 4.0;
        pill.draw_in(paint, PPoint::new(origin.x + apart, origin.y), self.frame.size.width - apart * 2.0).into_iter().zip(all).collect()
    }
}
