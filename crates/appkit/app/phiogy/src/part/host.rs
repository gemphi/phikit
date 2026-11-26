//! Hosts a person is in, and hosts they make, as the web keeps them
//! (`useHosts` in `puikit/app/phi-kit/src/hosts.tsx`). A host of the
//! catalogue is joined from the start when the app's document says so,
//! until the person says otherwise; what they say is their own
//! document. A host they make starts from one of the catalogue's
//! templates: its rooms under their headings, kept whole in another
//! document of theirs, spelled as the web spells it, so the web's rail
//! shows it too.

use serde_json::{Map, Value, json};

use appkit::pui::state::PState;
use appkit::pui::window::{PDeed, PEntry, PItem};
use appkit::service::PTable;

use super::Desk;
use crate::data;
use crate::hour;
use crate::site::{Plate, Template, Venue};

/// The person's own documents hosts are kept in.
pub const IN: &str = "host";
pub const MADE: &str = "made";
/// Where the hosts others published are kept for the page to list, and what a published host's id begins with.
pub const PUBLISHED: &str = "hosts/published";
pub const PUB: &str = "pub:";
/// The word a row of the bar tells to go to a host: its id follows.
pub const GO: &str = "host ";
/// The row of `data/site.tsv` that lists the web's icons for a kind of
/// host, which are not a host's own mark.
pub const KIND: &str = "host.kind";

/// A number written with the ten digits and the twenty-six letters,
/// as the web writes the moment a host was made.
fn short(mut number: u128) -> String {
    let digit = |value: u128| char::from_digit(value as u32, 36).unwrap_or('0');
    let mut out = vec![digit(number % 36)];
    while number >= 36 {
        number /= 36;
        out.push(digit(number % 36));
    }
    out.into_iter().rev().collect()
}

impl Desk {
    /// Whether the person is in a host: as they last said, or as the
    /// app's document says.
    pub fn joined(&self, venue: &Venue) -> bool {
        self.kept.flag(IN, &self.key(&venue.id)).unwrap_or(venue.joined)
    }

    /// The hosts the person is in, as rows of the bar: each with its
    /// mark, the one whose room is open marked, and a press going to
    /// its first room. They follow the bar's own rows, as the web's
    /// rail has them: the site's own places, then the hosts joined,
    /// then the ones the person made. The direct list is the bar's own
    /// first row, and has none here.
    pub fn place(&self) -> Vec<PItem> {
        let (site, open) = (&self.site, self.room());
        let stands = |(index, venue): &(usize, &Venue)| (*index < site.extra.nav || self.joined(venue)) && venue.id != site.extra.direct;
        let own = site.venue.iter().enumerate().filter(stands).map(|(_, venue)| venue.clone());
        // A host the person made has no mark of its own: its row
        // shows the letter of its name, drawn.
        let made = self.made().into_iter().map(|venue| Venue { icon: String::new(), ..venue });
        // An icon that says only what kind a host is, is not its own:
        // such a host shows the letter of its name, as one made does.
        let kind: Vec<String> = PTable::read(data::SITE).get(KIND).split_whitespace().map(str::to_string).collect();
        own.chain(made).filter(|venue| !venue.plate.is_empty()).map(|venue| {
            let icon = Some(venue.icon.as_str()).filter(|icon| !icon.is_empty() && !kind.iter().any(|kind| kind == icon)).map(|icon| self.icon(icon)).unwrap_or_default();
            let entry = PEntry::new(&format!("{GO}{}", venue.id), &venue.label, icon);
            let face = venue.mark.as_ref().map(|face| (face.sheet.clone(), face.column, face.index));
            let on = venue.plate.iter().any(|plate| plate.id == open);
            PItem { entry, deed: Some(PDeed::Tell(format!("{GO}{}", venue.id))), face, on, ..PItem::default() }
        }).collect()
    }

    /// The host a room is a room of: its id and what it is called.
    pub fn host_of(&self, room: &str) -> Option<(String, String)> {
        let made = self.made();
        let host = self.site.venue.iter().chain(&made).find(|venue| venue.plate.iter().any(|plate| plate.id == room));
        host.map(|venue| (venue.id.clone(), venue.label.clone()))
    }

    /// Go to a host: its first room. `false` when there is no such host.
    pub fn visit(&self, id: &str) -> bool {
        let made = self.made();
        let venue = self.site.venue.iter().chain(&made).find(|venue| venue.id == id);
        // A host opened shows every one of its channels: a tab chosen
        // before (Favorites) would leave it empty.
        let tab = self.tabs().1;
        (!tab.is_empty()).then(|| self.sift(&tab));
        // Who was pressed in another host is not this one's.
        *self.now.who.borrow_mut() = None;
        venue.and_then(|venue| venue.plate.first()).is_some_and(|plate| self.show(&plate.id))
    }

    /// Join a host, or leave it.
    pub fn enter(&self, id: &str, on: bool) {
        self.kept.put(IN, &self.key(id), json!(on));
    }

    /// The hosts the person made, each with its rooms under their
    /// headings.
    pub fn made(&self) -> Vec<Venue> {
        let key = PTable::read(data::SITE);
        let text = |value: &Value, name: &str| value.get(key.get(name)).and_then(Value::as_str).unwrap_or_default().to_string();
        let list = |value: &Value, name: &str| value.get(key.get(name)).and_then(Value::as_array).cloned().unwrap_or_default();
        let all = self.kept.get(MADE).get(&self.site.app).and_then(Value::as_array).cloned().unwrap_or_default();
        let one = |one: &Value| {
            let (group, room) = (one.get(key.get("group")).cloned().unwrap_or_default(), one.get(key.get("room")).cloned().unwrap_or_default());
            let plate = |head: &Value, item: &Value| {
                let id = text(item, "id");
                let voice = text(&room[&id], "kind") == key.get("voice");
                Plate { label: text(item, "label"), icon: text(item, "icon"), head: text(head, "label"), voice, id, ..Plate::default() }
            };
            let plate = list(&group, "item").iter().flat_map(|head| list(head, "item").iter().map(|item| plate(head, item)).collect::<Vec<_>>()).collect();
            Venue { id: text(&group, "id"), label: text(&group, "label"), icon: text(&group, "icon"), joined: true, plate, ..Venue::default() }
        };
        all.iter().map(one).collect()
    }

    /// Make a host of that name from a template. The id of its first
    /// room, to go to; nothing for a name of nothing but spaces.
    pub fn create(&self, name: &str, template: &Template) -> Option<String> {
        self.create_with(name, template).map(|(_, room)| room)
    }

    /// Make a host as `create` does, and say its id as well as its first
    /// room: the id is what it is published under.
    pub fn create_with(&self, name: &str, template: &Template) -> Option<(String, String)> {
        let (key, name) = (PTable::read(data::SITE), name.trim());
        if name.is_empty() {
            return None;
        }
        let id = format!("{}{}", key.get("sent.id"), short(hour::moment()));
        self.build(&id, name, &template.icon, &template.room).map(|room| (id, room))
    }

    /// A host of that id, name and icon with those rooms (each its label,
    /// its heading and whether it is a voice room), kept as the person's
    /// own. Nothing where they already have it. The id of its first room.
    pub fn build(&self, id: &str, name: &str, mark: &str, rooms: &[(String, String, bool)]) -> Option<String> {
        let key = PTable::read(data::SITE);
        if self.made().iter().any(|venue| venue.id == id) {
            return None;
        }
        let word = |name: &str| key.get(name).to_string();
        let icon = |voice: bool| match voice {
            true => word("made.voice"),
            false => word("made.text"),
        };
        let room: Vec<(String, &(String, String, bool))> = rooms.iter().enumerate().map(|(index, room)| (format!("{id}-{index}"), room)).collect();
        let mut head: Vec<&str> = Vec::new();
        room.iter().for_each(|(_, (_, under, _))| if !head.contains(&under.as_str()) { head.push(under) });
        let item = |under: &str| {
            let own = room.iter().filter(|(_, (_, head, _))| head == under);
            own.map(|(id, (label, _, voice))| json!({ word("id"): id, word("label"): label, word("icon"): icon(*voice) })).collect::<Vec<_>>()
        };
        let heads: Vec<Value> = head.iter().enumerate().map(|(index, under)| json!({ word("id"): format!("{id}-h{index}"), word("label"): under, word("icon"): word("made.text"), word("head"): true, word("item"): item(under) })).collect();
        let group = json!({ word("id"): id, word("label"): name, word("icon"): mark, word("item"): heads });
        let kind = |voice: bool| Some(word("voice")).filter(|_| voice);
        let rooms: Map<String, Value> = room.iter().map(|(id, (label, _, voice))| {
            let mut one = json!({ word("label"): label, word("space"): name, word("member"): [], word("message"): [] });
            kind(*voice).into_iter().for_each(|kind| one[word("kind")] = json!(kind));
            (id.clone(), one)
        }).collect();
        let mut all = self.kept.get(MADE).get(&self.site.app).and_then(Value::as_array).cloned().unwrap_or_default();
        all.push(json!({ word("group"): group, word("room"): rooms }));
        self.kept.put(MADE, &self.site.app, Value::Array(all));
        room.first().map(|(id, _)| id.clone())
    }

    /// Remove a host the person made.
    pub fn unmake(&self, id: &str) {
        let key = PTable::read(data::SITE);
        let all = self.kept.get(MADE).get(&self.site.app).and_then(Value::as_array).cloned().unwrap_or_default();
        let kept: Vec<Value> = all.into_iter().filter(|one| one[key.get("group")][key.get("id")] != json!(id)).collect();
        self.kept.put(MADE, &self.site.app, Value::Array(kept));
    }

    /// Publish a host just made on Phiora, as Phiegg's own hosts are, so
    /// others can see it and join it. What Phiora said, or why it could
    /// not (a guest cannot publish).
    pub fn publish_host(&self, id: &str, name: &str, template: &Template) -> String {
        let rooms: Vec<Value> = template.room.iter().map(|(label, under, voice)| json!({ "label": label, "under": under, "voice": voice })).collect();
        let payload = json!({ "name": name.trim(), "genre": template.label, "icon": template.icon, "members": 1, "online": 1, "rooms": rooms });
        match self.kept.records().and_then(|records| records.set("host", id, payload)) {
            Ok(set) => set.said("host", id),
            Err(why) => why,
        }
    }

    /// Ask Phiora for the hosts other people published, and keep them
    /// for the hosts' page to list.
    pub fn refresh_published(&self) {
        let word = PTable::read(data::MAKE);
        let me = self.kept.me();
        let mine = me[self.kept.word("me.id")].as_str().unwrap_or_default().to_string();
        let tree = self.kept.read(&word.fill("list", "kind", "host")).and_then(|text| serde_json::from_str::<Value>(&text).ok());
        let rows = tree.iter().filter_map(|tree| tree[word.get("answer.sources")].as_array()).flatten().filter(|source| source[word.get("answer.kind")] == "host").filter_map(|source| source[word.get("answer.rows")].as_array()).flatten();
        let list: Vec<Value> = rows.filter(|row| row["ref"].is_string() && row["rooms"].is_array() && row["ownerId"].as_str().is_some_and(|owner| owner != mine)).map(|row| json!({ "ref": row["ref"], "name": row["name"], "icon": row["icon"], "members": row["members"], "online": row["online"], "rooms": row["rooms"] })).collect();
        self.shelf.set(PUBLISHED, &Value::Array(list).to_string());
    }

    /// The hosts others published that the person is not yet in, as
    /// hosts they could join: their rooms are the ones it was made with.
    pub fn published(&self) -> Vec<Venue> {
        let key = PTable::read(data::SITE);
        let have: Vec<String> = self.made().into_iter().map(|venue| venue.id).collect();
        let list: Vec<Value> = serde_json::from_str(&self.shelf.get(PUBLISHED)).unwrap_or_default();
        let text = |value: &Value| value.as_str().unwrap_or_default().to_string();
        list.iter().filter_map(|one| {
            let id = one["ref"].as_str().filter(|id| !have.iter().any(|have| have == id))?;
            let plate = one["rooms"].as_array()?.iter().enumerate().map(|(index, room)| {
                let voice = room["voice"].as_bool().unwrap_or(false);
                Plate { id: format!("{id}-{index}"), label: text(&room["label"]), icon: key.get(if voice { "made.voice" } else { "made.text" }).to_string(), head: text(&room["under"]), voice, ..Plate::default() }
            }).collect();
            let icon = Some(text(&one["icon"])).filter(|icon| !icon.is_empty()).unwrap_or_else(|| key.get("made.text").to_string());
            Some(Venue { id: format!("{PUB}{id}"), label: text(&one["name"]), icon, count: one["members"].as_u64().unwrap_or(1), online: one["online"].as_u64().unwrap_or(1), joined: false, mark: None, plate })
        }).collect()
    }

    /// Join a host another published: it becomes one of the person's own,
    /// with the rooms it was made with. The id of its first room.
    pub fn adopt(&self, place: &Venue) -> Option<String> {
        let id = place.id.strip_prefix(PUB)?;
        let rooms: Vec<(String, String, bool)> = place.plate.iter().map(|plate| (plate.label.clone(), plate.head.clone(), plate.voice)).collect();
        self.build(id, &place.label, &place.icon, &rooms)
    }
}
