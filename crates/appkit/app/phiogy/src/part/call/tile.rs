//! The tiles of a call: who and what is shown, in the order the web's
//! call shows them. This window's own side first: its camera as a
//! mirror shows it, or its person's face, and the screen it shares.
//! Then each other caller: their camera while it arrives, or their
//! face, their name and whether they are muted, and beside it a screen
//! they share. Last, in a room with one person, that person while they
//! have not joined: ringing.
//!
//! A tile says what it shows by name: a moving picture's name in the
//! window's film (`peer::film`), or a person's face.

use appkit::pui::core::{PFit, PLight, PPaint, PPoint, PRect, light};
use appkit::service::PTable;

use crate::part::{Desk, INK, LINE};
use crate::peer::{self, Seen};

/// What a tile shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Shown {
    /// A moving picture, by its name in the film.
    Film(String),
    /// A person's face: their id among the app's people, nothing for
    /// one who is not of them, and their name.
    Face(String, String),
}

/// One tile of a call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tile {
    pub shown: Shown,
    /// Whether it is a screen, which is wider than a camera.
    pub scene: bool,
    /// Who it is, and how they are; either may be nothing.
    pub name: String,
    pub said: String,
    /// Whether they are heard now.
    pub loud: bool,
}

impl Tile {
    fn film(name: String, scene: bool) -> Tile {
        Tile { shown: Shown::Film(name), scene, name: String::new(), said: String::new(), loud: false }
    }
}

impl Desk {
    /// The tiles of the call this window is in: none when it is in none.
    /// `table` is `data/peer.tsv`, `speak` how loud counts as heard.
    pub fn tile(&self, table: &PTable, speak: f32) -> Vec<Tile> {
        let Some(seen) = self.calling() else { return Vec::new() };
        let text = |name: &str| self.text.get(name).to_string();
        let mut tile = self.own(&seen, table);
        let room = self.room();
        let (roster, _) = self.on(&room);
        let mut joined = Vec::new();
        for far in &seen.far {
            // What they call themself: the board says, where this window
            // serves the line; the line told the call, where it does not.
            let caller = roster.list.iter().find(|(peer, _)| *peer == far.peer).map(|(_, caller)| (caller.name.clone(), caller.handle.clone()));
            let caller = caller.or_else(|| seen.named.iter().find(|(peer, _)| *peer == far.peer).map(|(_, name)| (name.clone(), None)));
            let member = caller.as_ref().and_then(|(name, handle)| self.site.member.iter().find(|member| handle.as_deref().is_some_and(|handle| handle == member.handle) || member.name == *name));
            let name = member.map(|member| member.name.clone()).or_else(|| caller.as_ref().map(|(name, _)| name.clone())).unwrap_or_else(|| far.peer.clone());
            let id = member.map(|member| member.id.clone()).unwrap_or_default();
            joined.push(id.clone());
            let shown = match far.camera() {
                Some(picture) => Shown::Film(peer::film(table, &far.peer, &picture.stream)),
                None => Shown::Face(id, name.clone()),
            };
            let said = match far.quiet {
                true => text("call.tile.muted"),
                false => text("call.incall"),
            };
            tile.push(Tile { shown, scene: false, name, said, loud: far.level > speak && !far.quiet });
            tile.extend(far.scene().map(|picture| Tile::film(peer::film(table, &far.peer, &picture.stream), true)));
        }
        // The person of a direct room, while they have not joined.
        let with = self.plate(&room).map(|plate| plate.with).filter(|with| !with.is_empty() && !joined.contains(with));
        let rung = with.and_then(|with| self.site.member(&with).cloned());
        tile.extend(rung.map(|member| Tile { shown: Shown::Face(member.id, member.name.clone()), scene: false, name: member.name, said: text("call.ringing"), loud: false }));
        tile
    }

    /// This window's own tiles: its camera or its person's face, and
    /// the screen it shares.
    fn own(&self, seen: &Seen, table: &PTable) -> Vec<Tile> {
        let text = |name: &str| self.text.get(name).to_string();
        let name = |stream: &str| peer::film(table, table.get("film.own"), table.get(stream));
        let shown = match seen.own.camera {
            true => Shown::Film(name("stream")),
            false => Shown::Face(self.site.me.clone(), text("call.you")),
        };
        let said = match seen.quiet {
            true => text("call.tile.muted"),
            false => text("call.you"),
        };
        let mine = Tile { shown, scene: false, name: String::new(), said, loud: false };
        std::iter::once(mine).chain(seen.own.screen.then(|| Tile::film(name("scene"), true))).collect()
    }
}

/// A person's face, round: their picture where the app has one; else,
/// for one who is not of the app's people, the first letter of what
/// they call themself.
pub fn face(desk: &Desk, paint: &mut dyn PPaint, rect: PRect, id: &str, name: &str) {
    match desk.site.face(id) {
        Some(cell) => paint.picture(&cell.sheet, rect, rect.size.width / 2.0, PFit::Cell { column: cell.column, index: cell.index }),
        None => {
            light::label(paint, rect, rect.size.width / 2.0, PLight::Rest, false, "");
            let letter: String = name.chars().take(1).collect();
            let at = PPoint::new(rect.center().x - paint.measure(&letter) / 2.0, rect.center().y - paint.size(LINE) / 2.0);
            paint.text(at, &letter, paint.color(INK));
        }
    }
}
