//! Phiogy's parts: what each slot of its window shows. `title`,
//! `present` and `status` are the room host's (`data/region.tsv`):
//! which room it is, who is on its line, and where the line is. `venue`
//! is the hosts space's: every place with rooms. `guide` is the guide
//! host's, in the bar's menu: the places a person joined and their
//! rooms. Each is a view; appkit gives it its rectangle and sends it
//! what a hand does.

mod account;
#[cfg(not(target_family = "wasm"))]
pub mod figure;
#[cfg(not(target_family = "wasm"))]
pub mod library;
pub mod bond;
pub mod call;
pub mod crowd;
mod friend;
mod guide;
mod host;
mod join;
pub mod list;
mod now;
pub mod page;
mod person;
mod present;
pub mod record;
mod room;
mod search;
pub mod state;
pub mod shop;
mod status;
mod store;
mod stream;
mod talk;
mod title;
pub mod tool;
mod venue;
mod viewer;
pub mod voice;

pub use account::{Account, who};
#[cfg(not(target_family = "wasm"))]
pub use figure::Figure;
#[cfg(not(target_family = "wasm"))]
pub use library::Browse;
pub use call::Calling;
pub use friend::Friend;
pub use guide::Guide;
pub use search::{SEARCH, Search};
pub use join::Join;
pub use now::{Now, Shot};
pub use page::{Page, Rung};
pub use person::Person;
pub use present::Present;
pub use room::Room;
pub use shop::Shopping;
pub use status::Status;
pub use stream::{Stream, VIEW};
pub use title::Title;
pub use tool::Tool;
pub use venue::Venue;
pub use viewer::Viewer;
pub use voice::Voice;

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use appkit::pui::core::PPaint;
use appkit::pui::window::{PDeed, PWish};
use appkit::service::PTable;
use appkit::theme::{PLook, PShade, PTheme};

use crate::board::{Board, lock};
use crate::choice::Choice;
use crate::comp::Roster;
use crate::kept::Kept;
use crate::peer::{Call, Maker, Ring};
use crate::site::{Message, Site};

/// The theme's tokens more than one part is drawn with.
pub const GAP: &str = "gap";
pub const INK: &str = "text.ink";
pub const DIM: &str = "text.dim";
pub const LINE: &str = "text.line";
pub const ICON: &str = "icon.size";
pub const ROW: &str = "row.height";
pub const RADIUS: &str = "row.radius";
/// How wide the column down the middle of the body is, which every part
/// there keeps to.
pub const COLUMN: &str = "column.width";
/// The word that ends a text cut short.
pub const CUT: &str = "text.cut";
/// The row of `data/icon.tsv` for a name with no row of its own, and
/// what a person's choice of room is kept under.
pub const ANY: &str = "any";
pub const ROOM: &str = "room";

/// What every part works on: the site, the board, the words a person
/// reads, and the look, which a part asks for its measures.
#[derive(Clone)]
pub struct Desk {
    pub site: Rc<Site>,
    /// Phiegg's records search finds and pages show, read once when first
    /// asked for.
    pub book: Rc<std::cell::OnceCell<search::Book>>,
    /// Who is on every room's line. The line, where there is one, runs
    /// on other threads and changes it.
    pub board: Arc<Mutex<Board>>,
    pub text: Rc<PTable>,
    /// The web's icons, and the shared icon each is drawn with.
    pub icon: Rc<PTable>,
    pub look: Rc<PLook>,
    /// The looks Phiogy can be shown in, and the one it is shown in.
    pub shade: Rc<PShade>,
    /// What the person chose, which is kept.
    pub choice: Rc<Choice>,
    /// Where a part asks the window for a deed.
    pub wish: PWish,
    /// The room that is open, by its id.
    pub open: Rc<RefCell<String>>,
    /// Where the line listens; or why there is none.
    pub line: Rc<Result<String, String>>,
    /// The space a room is shown in.
    pub home: String,
    /// The picture the viewer shows: its number among the pictures it
    /// was opened on (`now.album`).
    pub view: Rc<Cell<Option<usize>>>,
    /// The person's own documents: what they wrote, marked and pinned.
    pub kept: Rc<Kept>,
    /// What is so of the open room just now, which nothing keeps.
    pub now: Rc<Now>,
    /// The pages that are no room.
    pub page: Rc<Page>,
    /// The shops that were opened, each read once from its own file.
    pub stock: Rc<RefCell<std::collections::HashMap<String, Option<Rc<crate::site::Shop>>>>>,
    /// The call this window is in, if it is in one.
    pub call: Rc<RefCell<Option<Call>>>,
    /// What makes the engine that carries a call this window joins.
    pub engine: Rc<dyn Fn() -> Maker>,
    /// Why the last call could not be joined, when it could not.
    pub refused: Rc<RefCell<String>>,
    /// Whether the line is somebody else's, not one this window serves;
    /// and, then, the open room's line listened to (`Desk::listen`).
    pub away: bool,
    pub ring: Rc<RefCell<Option<Ring>>>,
    /// The asset library, and what only the window keeps of it: the
    /// place, kind and order shown, the file opened.
    #[cfg(not(target_family = "wasm"))]
    pub library: Rc<crate::library::Library>,
    pub shelf: Rc<appkit::pui::state::PMemory>,
}

/// Wear a look as it is kept: a look's name, or Auto with the look for
/// the system's light and the one for its dark (the user, 25 November
/// 2026). `false` when there is no such look.
pub fn dressed(shade: &PShade, kept: &str) -> bool {
    match kept.split_whitespace().collect::<Vec<_>>().as_slice() {
        ["auto", light, dark] if shade.list().contains(light) && shade.list().contains(dark) => {
            shade.auto(Some((light.to_string(), dark.to_string())));
            true
        }
        _ => {
            let turned = shade.turn(kept);
            if turned {
                shade.auto(None);
            }
            turned
        }
    }
}

impl Desk {
    /// Phiegg's records (search/book.rs), read the first time they are
    /// asked for and kept: search finds in them, a record's page shows one.
    pub fn book(&self) -> &search::Book {
        self.book.get_or_init(|| {
            let kinds = search::Book::kinds();
            search::Book::read(&self.kept, &kinds.iter().map(String::as_str).collect::<Vec<_>>())
        })
    }

    /// The window's film: the pictures that move, a call's and the
    /// library's 3D ones.
    pub fn film(&self) -> appkit::pui::core::PFilm {
        #[cfg(not(target_family = "wasm"))]
        return self.library.film.clone();
        #[cfg(target_family = "wasm")]
        return appkit::pui::core::PFilm::default();
    }

    /// A size of the look; nothing for a token it does not have.
    pub fn size(&self, token: &str) -> f32 {
        self.look.size(token).unwrap_or_default()
    }

    /// The column down the middle of a part that wide: its left edge and
    /// its width, in the part's own coordinates. It is as wide as the
    /// theme says, and never nearer the part's sides than a gap.
    pub fn column(&self, width: f32) -> (f32, f32) {
        let wide = self.look.size(COLUMN).unwrap_or(f32::MAX).min(width - self.size(GAP) * 2.0).max(0.0);
        ((width - wide) / 2.0, wide)
    }

    /// The shared icon one of the web's is drawn with.
    pub fn icon(&self, name: &str) -> &str {
        match self.icon.get(name) {
            "" => self.icon.get(ANY),
            found => found,
        }
    }

    /// The room the open page shows: the one its block of a room's
    /// kind names; the page's own id where it has none.
    pub fn room(&self) -> String {
        let open = self.open.borrow();
        let block = self.site.block(&open);
        let shown = block.iter().find(|block| (&block.kind, &block.variant) == (&self.site.own.0, &self.site.own.1) && !block.room.is_empty());
        shown.map_or_else(|| open.clone(), |block| block.room.clone())
    }

    /// The kinds of the open page's blocks this window does not draw.
    pub fn undrawn(&self) -> Vec<String> {
        let block = self.site.block(&self.open.borrow());
        block.into_iter().filter(|block| (&block.kind, &block.variant) != (&self.site.own.0, &self.site.own.1)).map(|block| block.kind).collect()
    }

    /// The pictures posted with one message.
    fn posted(&self, one: &Message) -> Vec<Shot> {
        let when = format!("{} {} {}", one.day, self.text.get("between"), one.at);
        one.image.iter().map(|key| Shot { key: key.clone(), who: one.who.clone(), by: one.by.clone(), when: when.clone(), file: None }).collect()
    }

    /// The pictures posted in the open room, in order.
    pub fn shot(&self) -> Vec<Shot> {
        self.said().iter().flat_map(|one| self.posted(one)).collect()
    }

    /// The pictures somebody posted: in every room the site has, then
    /// those Phiegg keeps of them (their shots), newest first.
    pub fn shot_of(&self, id: &str) -> Vec<Shot> {
        let said = self.site.venue.iter().flat_map(|venue| &venue.plate).flat_map(|plate| &plate.message).filter(|one| one.who == id);
        let own = self.site.member(id).map(|one| one.picture.clone()).unwrap_or_default();
        let kept = own.into_iter().map(|key| Shot { key, who: id.to_string(), by: self.site.member(id).map(|one| one.name.clone()).unwrap_or_default(), when: String::new(), file: None });
        said.flat_map(|one| self.posted(one)).chain(kept).collect()
    }

    /// Open the viewer on one of a list of pictures.
    pub fn look(&self, album: Vec<Shot>, number: usize) {
        *self.now.album.borrow_mut() = album;
        self.view.set(Some(number));
        self.wish.borrow_mut().push(PDeed::Open(VIEW.to_string()));
    }

    /// A room's name on the line.
    pub fn key(&self, room: &str) -> String {
        format!("{}/{room}", self.site.app)
    }

    /// Who is on a room's line now: in its call, and with it open.
    pub fn on(&self, room: &str) -> (Roster, Roster) {
        let board = lock(&self.board);
        board.room(&self.key(room)).map(|word| (board.roster(word), board.watch(word))).unwrap_or_default()
    }

    /// Show Phiogy as the person prefers from now on (`kind token value`
    /// rows apart by `;`, as the profile tells it), and keep that.
    pub fn prefer(&self, kept: &str) -> bool {
        self.shade.prefer(kept);
        self.choice.put("prefer", kept);
        true
    }

    /// Show Phiogy in the look of that name from now on, or on Auto
    /// (`auto <light look> <dark look>`, following the system), and keep
    /// that. `false` when there is no such look.
    pub fn dress(&self, name: &str) -> bool {
        let turned = dressed(&self.shade, name);
        if turned {
            self.choice.put("look", name);
        }
        turned
    }
}

/// `text` when it is no wider than `width`; else as much of its start
/// as fits with the theme's mark for a text cut short after it.
pub fn fit(text: &str, width: f32, paint: &dyn PPaint) -> String {
    appkit::pui::block::PText::fit(text, width, paint)
}
