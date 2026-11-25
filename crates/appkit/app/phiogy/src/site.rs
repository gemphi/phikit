//! The site: the web Phiogy's own hosts and rooms, read when Phiogy
//! opens. The app's document names the rooms shown in its rail and the
//! two files that say more: one of every room and person, one of every
//! host. The document is Phiora's when it is asked and answers, and
//! the app's own file when not; the two files are the app's own, read
//! from beside that file either way, as a page gets them. Nothing of them is copied here, and what each name in
//! them is called is `data/site.tsv`.
//!
//! What the web calls a host, a place with rooms, is a `Venue` here:
//! appkit's own host is what an area of the window holds.

mod extra;
mod member;
mod read;
mod shop;

pub use extra::{Extra, Store, Template};
pub use member::Member;
pub use shop::{Product, Shelf, Shop};
pub use read::spoken;

use std::collections::HashMap;
use std::path::Path;

use appkit::service::PTable;
use serde_json::Value;

use crate::data;
use crate::phiora::Phiora;

/// A face, or a place's mark: one square cell of a sheet of pictures.
/// The sheet by its key, how many cells it has across, and the cell's
/// number, the first at its top left.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Face {
    pub sheet: String,
    pub column: u32,
    pub index: u32,
}

/// What is written on a room's door: its name on the line, what it is
/// called, its icon, whether it is a voice room, its topic, its people.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Plate {
    pub id: String,
    pub label: String,
    pub icon: String,
    pub voice: bool,
    pub topic: String,
    pub member: Vec<String>,
    /// The heading it stands under in its place (a category); or none.
    pub head: String,
    /// How deep it stands: 0 a channel, 1 a channel's sub-channel (a
    /// world's city), and so on, as Discord nests threads.
    pub depth: u32,
    /// How many of its messages the person has not read.
    pub unread: u64,
    /// The person a direct room is with, by id; nothing for any other.
    pub with: String,
    /// What was said in it, the oldest first.
    pub message: Vec<Message>,
}

/// Something said in a room.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Message {
    pub id: String,
    /// Who said it, by name, and by id.
    pub by: String,
    pub who: String,
    /// The day and the hour, as the site writes them.
    pub day: String,
    pub at: String,
    pub text: String,
    /// The pictures posted with it, by their keys.
    pub image: Vec<String>,
    /// What people marked it with: an icon, and how many did.
    pub mark: Vec<(String, u64)>,
    /// The message it answers, by id; nothing for one that answers none.
    pub reply: String,
    pub pin: bool,
}

/// A piece of a page: its kind and variant, and the room it shows.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Block {
    pub kind: String,
    pub variant: String,
    pub room: String,
}

/// A place with rooms.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Venue {
    pub id: String,
    pub label: String,
    pub icon: String,
    /// How many members it has, and how many of them are here.
    pub count: u64,
    pub online: u64,
    /// Whether the person joined it: it is in their rail.
    pub joined: bool,
    /// Its mark, where it has one.
    pub mark: Option<Face>,
    pub plate: Vec<Plate>,
}

/// Every venue, and where a person begins.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Site {
    /// The app's name on the line: a room is `app/room` there.
    pub app: String,
    /// The room a person begins in.
    pub start: String,
    pub venue: Vec<Venue>,
    /// The pages the app's file gives, each by its id with its blocks.
    pub view: Vec<(String, Vec<Block>)>,
    /// The kind and the variant of a block that shows a room.
    pub own: (String, String),
    /// Each person's face, by their id.
    pub face: HashMap<String, Face>,
    /// Everyone of the app, and who among them the person using it is.
    pub member: Vec<Member>,
    pub me: String,
    /// The file of who is linked to whom, as the rooms' file names it.
    pub link: String,
    /// What stands around the rooms: templates, friends, shops.
    pub extra: Extra,
    /// Where it was read from.
    pub from: String,
    /// Where Phiora was asked for the document, and whether it gave it;
    /// nothing when nobody was asked.
    pub asked: Option<(String, bool)>,
}

/// A file read whole: from this disk, or from the app's server where it
/// is written as an address there.
fn whole(from: &str) -> Option<String> {
    match crate::net::far(from) {
        true => crate::net::fetch(from).and_then(|bytes| String::from_utf8(bytes).ok()),
        false => std::fs::read_to_string(from).ok(),
    }
}

/// A file named from the folder over the one a file is in, wherever
/// that file is: `data/room.json` beside `data/phiogy.json`.
fn beside(from: &str, name: &str) -> Option<String> {
    match crate::net::far(from) {
        true => {
            let root = from.rsplitn(3, '/').nth(2)?;
            whole(&format!("{root}/{}", name.trim_start_matches('/')))
        }
        false => {
            let root = Path::new(from).parent().and_then(Path::parent)?;
            std::fs::read_to_string(root.join(name)).ok()
        }
    }
}

impl Site {
    /// The site of the app's own file at a path on disk, with the files
    /// it names beside it. One that cannot be read is a site with
    /// nothing in it, which says where it looked.
    pub fn open(path: &str) -> Site {
        Site::first(None, "", path)
    }

    /// The site as a page loads it: the app's document from Phiora,
    /// when it is asked and answers with one, and from the app's file
    /// at that path when not. The files the document names are read
    /// from beside that file either way.
    pub fn first(phiora: Option<&Phiora>, app: &str, path: &str) -> Site {
        let key = PTable::read(data::SITE);
        let given = phiora.and_then(|phiora| phiora.app(app)).filter(|doc| Site::whole(doc, &key));
        let asked = phiora.map(|phiora| (phiora.at.clone(), given.is_some()));
        // The files it names are given from the folder over its own.
        let load = |name: &str| beside(path, name);
        let doc = given.unwrap_or_else(|| whole(path).unwrap_or_default());
        Site { from: path.to_string(), asked, ..Site::read(&doc, &load, &key) }
    }

    /// A room by its id, with the venue it is in: the first that has it.
    pub fn plate(&self, id: &str) -> Option<(&Venue, &Plate)> {
        self.venue.iter().find_map(|venue| venue.plate.iter().find(|plate| plate.id == id).map(|plate| (venue, plate)))
    }

    /// The blocks of a page. A room the app's file gives no page is
    /// shown as its own: one block of the kind and variant the table
    /// says a room has.
    pub fn block(&self, page: &str) -> Vec<Block> {
        let own = || vec![Block { kind: self.own.0.clone(), variant: self.own.1.clone(), room: page.to_string() }];
        self.view.iter().find(|(id, _)| id == page).map_or_else(own, |(_, block)| block.clone())
    }

    /// Somebody of the app, by their id.
    pub fn member(&self, id: &str) -> Option<&Member> {
        self.member.iter().find(|member| member.id == id)
    }

    /// A file the app's document names, read from beside its own file.
    pub fn load(&self, name: &str) -> Option<String> {
        beside(&self.from, name)
    }

    /// Who somebody is linked to, by their ids: read from the file of
    /// links when it is asked, since that file is large.
    pub fn linked(&self, id: &str) -> Vec<String> {
        let key = PTable::read(data::SITE);
        let doc = self.load(&self.link).and_then(|text| serde_json::from_str::<Value>(&text).ok()).unwrap_or_default();
        let every: Vec<&str> = doc[key.get("link.id")].as_array().into_iter().flatten().filter_map(Value::as_str).collect();
        let own = every.iter().position(|known| *known == id).and_then(|at| doc[key.get("link.list")][at].as_array());
        own.into_iter().flatten().filter_map(Value::as_u64).filter_map(|at| every.get(at as usize)).map(|id| id.to_string()).collect()
    }

    /// The face of the person of that id, where the site has one.
    pub fn face(&self, id: &str) -> Option<&Face> {
        self.face.get(id)
    }

    /// The venues in the person's rail, in order.
    pub fn joined(&self) -> impl Iterator<Item = &Venue> {
        self.venue.iter().filter(|venue| venue.joined)
    }
}
