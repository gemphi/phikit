//! Phiegg's collections and clubs, as the data step brought them into
//! the web Phiogy's files (`shelf.json`, written by puikit
//! app/phiogy/library.py): a collection is a family of clubs, a club a
//! set of items a creator made (Bold Pups' 10,000 pups the largest),
//! each item its portrait and the traits it was made of.

use std::path::Path;

use serde_json::Value;

use super::Item;

/// A club: its id, name, tagline, collection, world's picture, how many
/// items it has in all, what it is (in Phiegg's words), and its items.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Club {
    pub id: String,
    pub name: String,
    pub tagline: String,
    pub collection: String,
    pub world: String,
    pub supply: u64,
    pub attribute: Vec<(String, String)>,
    pub item: Vec<Item>,
}

/// A collection: its id, name, what it is about, its world's picture,
/// and its clubs by id.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub topic: String,
    pub world: String,
    pub club: Vec<String>,
}

/// Every collection and club.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Shelf {
    pub collection: Vec<Collection>,
    pub club: Vec<Club>,
}

/// Words of a field.
fn text(value: &Value, name: &str) -> String {
    value[name].as_str().unwrap_or_default().to_string()
}

/// Pairs of words: `[[name, value], ...]`.
fn pairs(value: &Value) -> Vec<(String, String)> {
    let pair = |row: &Value| Some((row.get(0)?.as_str()?.to_string(), row.get(1)?.as_str()?.to_string()));
    value.as_array().into_iter().flatten().filter_map(pair).collect()
}

impl Shelf {
    /// The shelf of that file; an empty one where it cannot be read. A
    /// club's items have that icon where they have no face.
    pub fn read(path: &Path, icon: &str) -> Shelf {
        let doc: Value = std::fs::read_to_string(path).ok().and_then(|text| serde_json::from_str(&text).ok()).unwrap_or_default();
        let item = |club: &str, one: &Value| {
            let face = one["cell"].as_array().and_then(|cell| Some((cell.first()?.as_str()?.to_string(), cell.get(1)?.as_u64()? as u32, cell.get(2)?.as_u64()? as u32)));
            Item { face, attribute: pairs(&one["attribute"]), ..Item::named(format!("club:{club}/{}", text(one, "id")), text(one, "name"), "profile", icon) }
        };
        let club = doc["club"].as_array().into_iter().flatten().map(|one| {
            let id = text(one, "id");
            let list = one["item"].as_array().into_iter().flatten().map(|it| item(&id, it)).collect();
            Club { name: text(one, "name"), tagline: text(one, "tagline"), collection: text(one, "collection"), world: text(one, "world"), supply: one["supply"].as_u64().unwrap_or_default(), attribute: pairs(&one["attribute"]), item: list, id }
        });
        let collection = doc["collection"].as_array().into_iter().flatten().map(|one| {
            let club = one["club"].as_array().into_iter().flatten().filter_map(Value::as_str).map(str::to_string).collect();
            Collection { id: text(one, "id"), name: text(one, "name"), topic: text(one, "topic"), world: text(one, "world"), club }
        });
        Shelf { club: club.collect(), collection: collection.collect() }
    }

    pub fn club(&self, id: &str) -> Option<&Club> {
        self.club.iter().find(|club| club.id == id)
    }

    pub fn collection(&self, id: &str) -> Option<&Collection> {
        self.collection.iter().find(|collection| collection.id == id)
    }
}
