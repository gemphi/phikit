//! The hosts, as the kit's forms (`data/form.tsv`): the page `hosts`, the
//! drawer `make` a host is made in, and the drawer `host` a host opens
//! in. The page: Create a host in its bar, the line that searches, then
//! the hosts that are the person's (made or joined) and every host to
//! join, each a grid that lays out as the person chooses. Making one: the
//! steps said in words, its name, how it starts (a template), Create.
//! A host: its mark, what it is, its rooms, and what can be done (open,
//! join or leave, remove one of one's own). This file only says what each
//! list holds and what a press does; how it all stands is the table's.

use appkit::pui::core::{PDraw, PEvent, PPaint, PPoint, PRect, PView};
use appkit::pui::form::{PForm, PListed};
use appkit::pui::state::PState;
use appkit::pui::window::PDeed;
use std::rc::Rc;

use super::Desk;
use super::list::grouped;
use crate::data;
use crate::site::Venue as Place;

/// How many hosts of the catalogue are listed at most, as on the web.
pub const MOST: &str = "host.most";
/// The drawers, as `data/modal.tsv` names them.
pub const MAKE: &str = "make";
pub const HOST: &str = "host";

/// The paths the forms keep and are told.
const FIND: &str = "hosts/find";
const SORT: &str = "hosts/sort";
const DO: &str = "hosts/do";
const OPEN: &str = "hosts/open";
const NAME: &str = "make/name";
const TEMPLATE: &str = "make/template";
const MADE: &str = "make/do";
const DEED: &str = "host/do";
const SAID: &str = "hosts/said";
const ROOM: &str = "host/room";

type Row = Vec<String>;

/// What the hosts' forms hold, and what a press on them does.
struct Hosts {
    desk: Desk,
}

impl Hosts {
    fn get(&self, path: &str) -> String {
        self.desk.shelf.get(path)
    }

    /// A card of a host: what a press opens, its name, its icon, its
    /// members and who is here, its mark.
    fn card(&self, place: &Place) -> Row {
        let text = &self.desk.text;
        let face = place.mark.as_ref().map(|cell| format!("{} {} {}", cell.sheet, cell.column, cell.index)).unwrap_or_default();
        let icon = match face.is_empty() {
            true => self.desk.icon(&place.icon).to_string(),
            false => String::new(),
        };
        vec![place.id.clone(), place.label.clone(), icon, String::new(), text.fill("host.members", "n", &grouped(place.count)), String::new(), face]
    }

    /// The host a drawer is about.
    fn open(&self) -> Option<Place> {
        let id = self.get(OPEN);
        self.desk.made().into_iter().chain(self.desk.site.venue.iter().cloned()).chain(self.desk.published()).find(|place| place.id == id)
    }

    /// Whether a host is one the search leaves.
    fn sought(&self, place: &Place) -> bool {
        let wanted = self.get(FIND).trim().to_lowercase();
        wanted.is_empty() || place.label.to_lowercase().contains(&wanted)
    }

    /// The catalogue's hosts: the site's, less the places of the rail's
    /// own (the direct rooms, the site's home).
    fn catalogue(&self) -> &[Place] {
        let site = &self.desk.site;
        &site.venue[site.extra.nav.min(site.venue.len())..]
    }

    /// The catalogue: every host to join, those the search leaves, in
    /// the order chosen, no more than the theme says.
    fn every(&self) -> Vec<Place> {
        let mut all: Vec<Place> = self.catalogue().iter().filter(|place| self.sought(place)).cloned().collect();
        match self.get(SORT).as_str() {
            "name" => all.sort_by_key(|place| place.label.to_lowercase()),
            _ => all.sort_by_key(|place| std::cmp::Reverse(place.count)),
        }
        all.truncate(self.desk.size(MOST) as usize);
        all
    }

    fn rows(&self, name: &str) -> Vec<Row> {
        let text = &self.desk.text;
        match name {
            // Theirs: the hosts they made, then those of the catalogue they
            // joined; the search narrows them too.
            "hosts.mine" => {
                let made = self.desk.made();
                let joined = self.catalogue().iter().filter(|place| self.desk.joined(place) && !made.iter().any(|one| one.id == place.id));
                made.iter().chain(joined).filter(|place| self.sought(place)).map(|place| self.card(place)).collect()
            }
            "hosts.every" => self.every().iter().map(|place| self.card(place)).collect(),
            // Hosts people published on Phiora, that they are not in yet.
            "hosts.made" => self.desk.published().iter().filter(|place| self.sought(place)).map(|place| self.card(place)).collect(),
            "hosts.say" => Some(self.get(SAID)).filter(|said| !said.is_empty()).map(|said| vec![vec!["say".to_string(), said]]).unwrap_or_default(),
            "make.template" => self.desk.site.extra.template.iter().enumerate().map(|(index, template)| {
                let room = template.room.iter().map(|(label, ..)| label.as_str()).collect::<Vec<_>>().join(&format!(" {} ", text.get("between")));
                vec![index.to_string(), template.label.clone(), self.desk.icon(&template.icon).to_string(), room]
            }).collect(),
            "host.about" => self.open().map(|place| {
                let rows = [("host.fact.name", place.label.clone()), ("host.fact.members", grouped(place.count)), ("host.fact.here", grouped(place.online)), ("host.fact.rooms", grouped(place.plate.len() as u64))];
                rows.into_iter().map(|(word, value)| vec![word.to_string(), text.get(word).to_string(), value]).collect()
            }).unwrap_or_default(),
            "host.room" => self.open().map(|place| place.plate.iter().map(|plate| vec![plate.id.clone(), plate.label.clone(), self.desk.icon(&plate.icon).to_string(), plate.head.clone()]).collect()).unwrap_or_default(),
            // What can be done with it: open it; join or leave one of the
            // catalogue; remove one of their own.
            "host.tool" => self.open().map(|place| {
                let own = self.desk.made().iter().any(|one| one.id == place.id);
                let deed = match (own, self.desk.joined(&place)) {
                    (true, _) => ("remove", "host.remove", "trash"),
                    (false, true) => ("leave", "host.leave", "log-out"),
                    (false, false) => ("join", "host.enter", "log-in"),
                };
                vec![vec!["open".to_string(), "host.open".to_string(), "external-link".to_string()], vec![deed.0.to_string(), deed.1.to_string(), deed.2.to_string()]]
            }).unwrap_or_default(),
            // Any other list is a group of `data/filter.tsv`.
            _ => data::row(data::FILTER).into_iter().filter(|row| row.first() == Some(&name)).map(|row| row[1..].iter().map(|cell| cell.to_string()).collect()).collect(),
        }
    }

    /// Make the host named from the template chosen, go to its first room
    /// and put the drawer away.
    fn make(&self) {
        let name = self.get(NAME);
        let template = self.get(TEMPLATE).parse::<usize>().ok().and_then(|index| self.desk.site.extra.template.get(index).cloned());
        let (Some(template), false) = (template, name.trim().is_empty()) else { return };
        if let Some((id, room)) = self.desk.create_with(name.trim(), &template) {
            // Published on Phiora, so others can see it and join it; if that cannot be, it is the person's own.
            let said = self.desk.publish_host(&id, name.trim(), &template);
            let text = &self.desk.text;
            let published = said.starts_with("made ") || said.starts_with("changed ") || said.starts_with("same ");
            self.desk.shelf.set(SAID, &if published { text.get("host.published").to_string() } else { format!("{}: {said}", text.get("host.local")) });
            self.desk.shelf.set(NAME, "");
            _ = self.desk.show(&room);
            self.desk.wish.borrow_mut().push(PDeed::Close);
        }
    }
}

impl PState for Hosts {
    fn get(&self, path: &str) -> String {
        self.desk.shelf.get(path)
    }

    fn set(&self, path: &str, value: &str) {
        let (desk, wish) = (&self.desk, |deed: PDeed| self.desk.wish.borrow_mut().push(deed));
        match (path, value) {
            (DO, MAKE) => wish(PDeed::Open(MAKE.to_string())),
            (DO, crate::part::publish::CLUB | crate::part::publish::PFT) => wish(PDeed::Open(value.to_string())),
            (OPEN, id) => {
                desk.shelf.set(OPEN, id);
                wish(PDeed::Open(HOST.to_string()));
            }
            (MADE, _) => self.make(),
            (ROOM, room) => {
                _ = desk.show(room);
                wish(PDeed::Close);
            }
            (DEED, deed) => {
                let Some(place) = self.open() else { return };
                match deed {
                    "open" => {
                        place.plate.first().into_iter().for_each(|plate| _ = desk.show(&plate.id));
                        wish(PDeed::Close);
                    }
                    "join" if place.id.starts_with(crate::part::host::PUB) => {
                        // A host another published: it becomes theirs, rooms and all, and they go to it.
                        if let Some(room) = desk.adopt(&place) {
                            _ = desk.show(&room);
                        }
                        wish(PDeed::Close);
                    }
                    "join" => desk.enter(&place.id, true),
                    "leave" => desk.enter(&place.id, false),
                    "remove" => {
                        desk.unmake(&place.id);
                        wish(PDeed::Close);
                    }
                    _ => {}
                }
            }
            _ => desk.shelf.set(path, value),
        }
    }
}

/// A form of the hosts: the page (`hosts`) or a drawer (`make`, `host`).
pub struct Venue {
    form: PForm,
}

impl Venue {
    pub fn new(name: &str, desk: Desk) -> Venue {
        // The hosts page asks Phiora for the hosts others published when it is made.
        if name == "hosts" {
            desk.refresh_published();
        }
        let hosts = Rc::new(Hosts { desk: desk.clone() });
        let rows = hosts.clone();
        let state: Vec<(String, Rc<dyn PState>)> = vec![("hosts".to_string(), hosts.clone()), ("make".to_string(), hosts.clone()), ("host".to_string(), hosts), ("ui".to_string(), desk.shelf.clone())];
        let source = Rc::new(PListed::new(data::LISTING, Rc::new(move |list: &str| rows.rows(list)), desk.text.clone(), state));
        Venue { form: PForm::new(name, data::FORM, source, desk.text.clone(), desk.shade.clone()) }
    }
}

impl PDraw for Venue {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        self.form.draw(paint, origin);
    }
}

impl PView for Venue {
    fn frame(&self) -> PRect {
        self.form.frame()
    }

    fn set_frame(&mut self, frame: PRect) {
        self.form.set_frame(frame);
    }

    fn respond(&mut self, event: &PEvent) -> bool {
        self.form.respond(event)
    }

    fn tick(&mut self) -> bool {
        self.form.tick()
    }

    fn hand(&self) -> Vec<PRect> {
        self.form.hand()
    }
}
