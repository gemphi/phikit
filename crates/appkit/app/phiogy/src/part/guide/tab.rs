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

use appkit::pui::core::{PPaint, PPoint, PRect};
use appkit::service::PTable;
use serde_json::{Value, json};

use super::super::{Desk, GAP, ROW, Rung};
use crate::{data, hour};

/// What the chosen tab is kept under, and the tabs by their names.
pub const TAB: &str = "guide";
pub const RECENT: &str = "recent";
pub const FAVORITE: &str = "favorite";
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
        let chosen = Some(self.choice.get(TAB)).filter(|name| all.contains(name)).unwrap_or_default();
        (all, chosen)
    }

    /// Choose a tab; the chosen one again lets every room back.
    pub fn sift(&self, name: &str) {
        match self.tabs().1 == name {
            true => self.choice.put(TAB, ""),
            false => self.choice.put(TAB, name),
        }
    }

    /// Keep that a room was opened now.
    pub fn saw(&self, room: &str) {
        self.kept.put(SEEN, &self.key(room), json!(hour::moment() as f64));
    }

    /// Whether a room is a recent one: opened within the hours the
    /// app's data gives, or with something waiting in it.
    pub fn lately(&self, rung: &Rung) -> bool {
        let span = PTable::read(data::APP).get(RECENT).parse::<f64>().unwrap_or_default() * HOUR;
        let opened = self.kept.get(SEEN).get(self.key(&rung.id)).and_then(Value::as_f64);
        opened.is_some_and(|at| hour::moment() as f64 - at <= span) || rung.count.max(self.waiting(&rung.id)) > 0
    }

    /// Whether the person starred a room; and star it, or no more.
    pub fn starred(&self, room: &str) -> bool {
        self.kept.flag(STAR, &self.key(room)).unwrap_or(false)
    }

    pub fn star(&self, room: &str, on: bool) {
        self.kept.put(STAR, &self.key(room), json!(on));
    }

    /// The rail as the chosen tab keeps it: every place with the rooms
    /// the tab is for, and no place it leaves nothing of.
    pub fn sifted(&self) -> Vec<(String, Vec<Rung>)> {
        let chosen = self.tabs().1;
        let keeps = |rung: &Rung| match chosen.as_str() {
            "" => true,
            RECENT => self.lately(rung),
            FAVORITE => self.starred(&rung.id) || (!rung.with.is_empty() && self.follows(&rung.with)),
            _ => !self.lately(rung),
        };
        let place = self.rail().into_iter().map(|(label, rung)| (label, rung.into_iter().filter(&keeps).collect::<Vec<_>>()));
        place.filter(|(_, rung)| !rung.is_empty()).collect()
    }
}

impl super::Guide {
    /// How high the tabs over the rows are: a row and a little under it.
    pub(super) fn head(&self) -> f32 {
        self.desk.size(ROW) + self.desk.size(GAP) / 4.0
    }

    /// Where each tab is, in a guide at that place: side by side over
    /// its whole width, each as wide as its own word and its share of
    /// the room that is left. Each with its name and what it reads.
    pub(super) fn tabs(&self, paint: &dyn PPaint, origin: PPoint) -> Vec<(PRect, String, String)> {
        let (gap, row) = (paint.size(GAP), paint.size(ROW));
        let all: Vec<(String, String)> = self.desk.tabs().0.into_iter().map(|name| (self.desk.text.get(&format!("guide.tab.{name}")).to_string(), name)).collect();
        let (count, apart) = (all.len().max(1) as f32, gap / 4.0);
        let words: f32 = all.iter().map(|(said, _)| paint.measure(said)).sum();
        let pad = ((self.frame.size.width - apart * (count + 1.0) - words) / count).max(0.0);
        let mut left = origin.x + apart;
        let place = |(said, name): (String, String)| {
            let rect = PRect::new(left, origin.y, paint.measure(&said) + pad, row - gap / 4.0);
            left += rect.size.width + apart;
            (rect, name, said)
        };
        all.into_iter().map(place).collect()
    }
}
