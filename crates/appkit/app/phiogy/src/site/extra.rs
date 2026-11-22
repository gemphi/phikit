//! What the site has around its rooms: what a host a person makes can
//! start from, how the person's friends start, and the shops there
//! are. Each is read from the file the app's document names for it, by
//! the names of `data/site.tsv`.

use appkit::service::PTable;
use serde_json::Value;

/// What a new host starts from: its rooms, each with what it is
/// called, the heading it stands under, and whether it is a voice room.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Template {
    pub id: String,
    pub label: String,
    pub icon: String,
    pub room: Vec<(String, String, bool)>,
}

/// A shop of the index: its id, what it is called, a line about it,
/// its icon, and how many things it has.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Store {
    pub id: String,
    pub label: String,
    pub sub: String,
    pub icon: String,
    pub count: u64,
}

/// What the site has around its rooms.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Extra {
    /// How many of the site's venues are the rail's own; the rest are
    /// the hosts of the catalogue.
    pub nav: usize,
    /// The group of the rail that lists direct conversations, by id.
    pub direct: String,
    pub template: Vec<Template>,
    /// Who the person is friends with from the start, and who has
    /// asked to be.
    pub friend: Vec<String>,
    pub asked: Vec<String>,
    /// The shops: the folder their own files are in, the ones open
    /// from the start, and every one of the index.
    pub stock: String,
    pub stocked: Vec<String>,
    pub store: Vec<Store>,
}

/// Read what is around the rooms out of the app's document, the rooms'
/// file, the hosts' file and the shops' index.
pub fn read(doc: &Value, people: &Value, host: &Value, shop: &Value, key: &PTable) -> Extra {
    let text = |value: &Value, name: &str| value.get(key.get(name)).and_then(Value::as_str).unwrap_or_default().to_string();
    let list = |value: &Value, name: &str| value.get(key.get(name)).and_then(Value::as_array).cloned().unwrap_or_default();
    let word = |value: &Value, name: &str| list(value, name).iter().filter_map(Value::as_str).map(str::to_string).collect::<Vec<_>>();
    let room = |one: &Value| (text(one, "label"), text(one, "template.head"), text(one, "kind") == key.get("voice"));
    let template = |one: &Value| Template { id: text(one, "id"), label: text(one, "label"), icon: text(one, "icon"), room: list(one, "room").iter().map(room).collect() };
    let store = |one: &Value| Store { id: text(one, "id"), label: text(one, "label"), sub: text(one, "sub"), icon: text(one, "icon"), count: one.get(key.get("shop.count")).and_then(Value::as_u64).unwrap_or_default() };
    let friend = people.get(key.get("friend")).cloned().unwrap_or_default();
    Extra {
        nav: list(doc, "nav").len(),
        direct: text(doc, "direct"),
        template: list(host, "template").iter().map(template).collect(),
        friend: word(&friend, "friend.with"),
        asked: word(&friend, "friend.asked"),
        stock: text(shop, "shop.folder"),
        stocked: word(shop, "shop.open"),
        store: list(shop, "shop.list").iter().map(store).collect(),
    }
}
