//! Pages and the rail. What the window is opened on is a page, as on
//! the web: a room of the site, a room of a host the person made, a
//! direct conversation with anybody (`dm-` and their id, which no file
//! holds), or one of the pages that are no room: friends, finding
//! people, the hosts, the shops, the bag. `data/page.tsv` says which
//! space each of those is shown in.
//!
//! The rail is what the person can go to: the places of the app's own
//! menu, the hosts they are in, the ones they made, and the shops they
//! have open, as the web's shell builds its menu
//! (`puikit/app/phi-kit/src/PhiShell.tsx` with `hosts.tsx`,
//! `direct.tsx`, `friends.tsx`, `shops.tsx`).

use appkit::pui::window::PDeed;
use appkit::service::PTable;
use serde_json::json;

use super::{Desk, ROOM, talk};
use crate::data;
use crate::site::Plate;

/// The rows of `data/page.tsv` the code asks for by name.
pub const FRIEND: &str = "friend";
pub const PEOPLE: &str = "people";
pub const HOSTS: &str = "hosts";
pub const SHOPS: &str = "shops";
pub const BAG: &str = "bag";
pub const DIRECT: &str = "direct";
pub const HOME: &str = "home";
pub const ACCOUNT: &str = "account";

/// A row of the rail: a page by its id, what it is called, its icon,
/// the person it is a conversation with, the heading it stands under,
/// and how many things wait there.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Rung {
    pub id: String,
    pub label: String,
    pub icon: String,
    pub with: String,
    pub head: String,
    pub count: u64,
}

impl Rung {
    pub fn of(plate: &Plate) -> Rung {
        Rung { id: plate.id.clone(), label: plate.label.clone(), icon: plate.icon.clone(), with: plate.with.clone(), head: plate.head.clone(), count: 0 }
    }
}

/// The pages that are no room, as `data/page.tsv` has them.
#[derive(Debug, Clone, Default)]
pub struct Page {
    row: Vec<Vec<String>>,
}

impl Page {
    pub fn own() -> Page {
        Page { row: data::row(data::PAGE).into_iter().map(|cell| cell.into_iter().map(str::to_string).collect()).collect() }
    }

    fn cell(&self, name: &str, at: usize) -> &str {
        self.row.iter().find(|cell| cell.first().is_some_and(|known| known == name)).and_then(|cell| cell.get(at)).map_or("", String::as_str)
    }

    /// The id of a page, as the web has it.
    pub fn id(&self, name: &str) -> &str {
        self.cell(name, 1)
    }

    /// The space a page of that id is shown in, where it is no room.
    pub fn space(&self, id: &str) -> Option<&str> {
        self.row.iter().find(|cell| cell.get(1).is_some_and(|known| known == id) && cell.len() > 2).map(|cell| cell[2].as_str())
    }

    /// The web's icon of a page.
    pub fn icon(&self, name: &str) -> &str {
        self.cell(name, 4)
    }

    /// The web's icon of somebody with no face.
    pub fn face(&self) -> &str {
        self.cell(PEOPLE, 5)
    }

    /// A row of the rail for a page that is no room.
    pub fn rung(&self, name: &str, text: &PTable, count: u64) -> Rung {
        Rung { id: self.id(name).to_string(), label: text.get(self.cell(name, 3)).to_string(), icon: self.cell(name, 4).to_string(), count, ..Rung::default() }
    }
}

impl Desk {
    /// The person a page is a direct conversation with, when its id
    /// says it is one that no file holds.
    pub fn direct_with(&self, id: &str) -> Option<String> {
        id.strip_prefix(self.page.id(DIRECT)).filter(|person| self.site.member(person).is_some()).map(str::to_string)
    }

    /// The room a person is written to in: the one the site has with
    /// them, else the one named after them.
    pub fn direct_room(&self, person: &str) -> String {
        let written = self.site.venue.iter().flat_map(|venue| &venue.plate).find(|plate| plate.with == person);
        written.map_or_else(|| format!("{}{person}", self.page.id(DIRECT)), |plate| plate.id.clone())
    }

    /// A room by its id: one of the site, one of a host the person
    /// made, or a direct conversation, named after who it is with.
    pub fn plate(&self, id: &str) -> Option<Plate> {
        let made = || self.made().into_iter().flat_map(|venue| venue.plate).find(|plate| plate.id == id);
        let direct = || {
            let person = self.direct_with(id)?;
            let name = self.site.member(&person).map(|member| member.name.clone())?;
            Some(Plate { id: id.to_string(), label: name, icon: self.page.face().to_string(), with: person, ..Plate::default() })
        };
        self.site.plate(id).map(|(_, plate)| plate.clone()).or_else(made).or_else(direct)
    }

    /// The space a page is shown in: a room's, or the page's own.
    pub fn space(&self, id: &str) -> Option<String> {
        let own = self.page.space(id).map(str::to_string);
        own.or_else(|| self.shop_of(id).map(|_| self.page.cell(SHOPS, 2).to_string())).or_else(|| self.plate(id).map(|_| self.home.clone()))
    }

    /// Open a page and ask the window for its space. `false` when
    /// there is no such page.
    pub fn show(&self, id: &str) -> bool {
        let Some(space) = self.space(id) else { return false };
        // Another page begins with nothing answered, sought or shown
        // beside it; and what waited in a room waits no more.
        self.now.answer.borrow_mut().clear();
        self.now.finding.set(false);
        self.now.pins.set(false);
        *self.now.who.borrow_mut() = None;
        *self.now.product.borrow_mut() = None;
        if self.waiting(id) > 0 {
            self.kept.put(talk::READ, &self.key(id), json!(true));
        }
        // A conversation that is opened has begun: it stays in the rail.
        self.direct_with(id).into_iter().for_each(|person| self.begin(&person));
        *self.open.borrow_mut() = id.to_string();
        if space == self.home {
            self.choice.put(ROOM, id);
            self.saw(id);
        }
        self.wish.borrow_mut().push(PDeed::Show(space));
        true
    }

    /// The rail: each place the person can go to, with its rows.
    pub fn rail(&self) -> Vec<(String, Vec<Rung>)> {
        let site = &self.site;
        let mut rail: Vec<(String, Vec<Rung>)> = Vec::new();
        for (index, venue) in site.venue.iter().enumerate() {
            let own = index < site.extra.nav;
            if !(own || self.joined(venue)) {
                continue;
            }
            let mut rung: Vec<Rung> = venue.plate.iter().map(Rung::of).collect();
            // The direct list is led by friends and the way to find
            // anyone, and ends with everyone a conversation began with.
            if own && venue.id == site.extra.direct && !venue.id.is_empty() {
                let lead = [self.page.rung(FRIEND, &self.text, self.bonded(super::bond::ASKED).len() as u64), self.page.rung(PEOPLE, &self.text, 0)];
                let begun = self.begun().into_iter().filter(|person| !venue.plate.iter().any(|plate| plate.with == *person));
                let after = begun.filter_map(|person| self.plate(&format!("{}{person}", self.page.id(DIRECT)))).map(|plate| Rung::of(&plate));
                rung = lead.into_iter().chain(rung).chain(after).collect();
            }
            rail.push((venue.label.clone(), rung));
        }
        rail.extend(self.made().iter().map(|venue| (venue.label.clone(), venue.plate.iter().map(Rung::of).collect())));
        rail.extend(self.shop_rail());
        rail
    }
}
