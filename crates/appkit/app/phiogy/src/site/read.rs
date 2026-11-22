//! Reading the site out of the web Phiogy's files: the app's own file,
//! the file of its rooms and people, the file of every host. What each
//! name in them is called is `data/site.tsv`; nothing is spelled here.

use std::collections::HashMap;

use appkit::service::PTable;
use serde_json::Value;

use super::{Block, Face, Message, Plate, Site, Venue, extra, member};

/// The files, and how to read a room out of them.
struct Read<'a> {
    key: &'a PTable,
    room: Value,
    name: HashMap<String, String>,
    /// The sheet a face is on when its person names none, and how many
    /// faces every sheet has across.
    sheet: Face,
}

impl Read<'_> {
    fn text(&self, value: &Value, name: &str) -> String {
        value.get(self.key.get(name)).and_then(Value::as_str).unwrap_or_default().to_string()
    }

    fn list(&self, value: &Value, name: &str) -> Vec<Value> {
        value.get(self.key.get(name)).and_then(Value::as_array).cloned().unwrap_or_default()
    }

    fn count(&self, value: &Value, name: &str) -> u64 {
        value.get(self.key.get(name)).and_then(Value::as_u64).unwrap_or_default()
    }

    /// Who someone is called; their id where nobody named them.
    fn named(&self, id: &str) -> String {
        self.name.get(id).cloned().unwrap_or_else(|| id.to_string())
    }

    /// A person's face: a cell of their sheet, or of the one all share.
    fn face(&self, one: &Value) -> Face {
        let own = Some(self.text(one, "sheet")).filter(|sheet| !sheet.is_empty());
        Face { sheet: own.unwrap_or_else(|| self.sheet.sheet.clone()), column: self.sheet.column, index: self.count(one, "face") as u32 }
    }

    /// A place's mark, where it has one: a cell of a sheet it names.
    fn mark(&self, item: &Value) -> Option<Face> {
        let mark = item.get(self.key.get("mark")).filter(|mark| mark.is_object())?;
        Some(Face { sheet: self.text(mark, "sheet"), column: self.count(mark, "sheet.column") as u32, index: self.count(mark, "mark.index") as u32 }).filter(|face| !face.sheet.is_empty())
    }

    /// Something said, as the rooms' file writes it.
    fn message(&self, one: &Value) -> Message {
        let who = self.text(one, "by");
        Message { by: self.named(&who), ..spoken(one, self.key) }
    }

    /// A room: what the rail says of it, the heading it stands under,
    /// and what the rooms' file adds.
    fn plate(&self, item: &Value, head: &str) -> Plate {
        let id = self.text(item, "id");
        let more = self.room.get(&id).cloned().unwrap_or_default();
        let member = self.list(&more, "member").iter().filter_map(Value::as_str).map(|id| self.named(id)).collect();
        let message = self.list(&more, "message").iter().map(|one| self.message(one)).collect();
        Plate { label: self.text(item, "label"), icon: self.text(item, "icon"), voice: self.text(&more, "kind") == self.key.get("voice"), topic: self.text(&more, "topic"), member, head: head.to_string(), unread: self.count(&more, "unread"), with: self.text(&more, "with"), message, id }
    }

    /// A page's blocks, as the app's file writes them.
    fn block(&self, page: &Value) -> Vec<Block> {
        self.list(page, "block").iter().map(|one| Block { kind: self.text(one, "kind"), variant: self.text(one, "variant"), room: self.text(one, "room") }).collect()
    }

    /// A venue: its rooms are what it holds, and what its headings hold.
    fn venue(&self, item: &Value, joined: bool) -> Venue {
        let head = |item: &Value| item.get(self.key.get("head")).and_then(Value::as_bool).unwrap_or(false);
        // What a heading holds stands under it; the rest under none.
        let under = |item: &Value| match head(item) {
            true => self.list(item, "item").into_iter().map(|room| (self.text(item, "label"), room)).collect(),
            false => vec![(String::new(), item.clone())],
        };
        let plate = self.list(item, "item").iter().flat_map(under).map(|(head, item)| self.plate(&item, &head)).collect();
        Venue { id: self.text(item, "id"), label: self.text(item, "label"), icon: self.text(item, "icon"), count: self.count(item, "count"), online: self.count(item, "online"), joined, mark: self.mark(item), plate }
    }
}

/// Something said, as the rooms' file and a person's own document of
/// what they wrote both write it. Who said it is their id, until
/// whoever reads it knows what they are called.
pub fn spoken(one: &Value, key: &PTable) -> Message {
    let text = |name: &str| one.get(key.get(name)).and_then(Value::as_str).unwrap_or_default().to_string();
    let list = |name: &str| one.get(key.get(name)).and_then(Value::as_array).cloned().unwrap_or_default();
    let image = list("image").iter().filter_map(Value::as_str).map(str::to_string).collect();
    let count = |mark: &Value| mark.get(key.get("mark.count")).and_then(Value::as_u64).unwrap_or_default();
    let icon = |mark: &Value| mark.get(key.get("icon")).and_then(Value::as_str).unwrap_or_default().to_string();
    let mark = list("mark").iter().map(|mark| (icon(mark), count(mark))).collect();
    let pin = one.get(key.get("pin")).and_then(Value::as_bool).unwrap_or(false);
    Message { id: text("id"), by: text("by"), who: text("by"), day: text("day"), at: text("at"), text: text("text"), image, mark, reply: text("reply"), pin }
}

impl Site {
    /// Whether a text is an app's document: it has the list of the rail.
    pub fn whole(doc: &str, key: &PTable) -> bool {
        serde_json::from_str::<Value>(doc).is_ok_and(|doc| doc.get(key.get("nav")).is_some_and(Value::is_array))
    }

    /// The site of the app's own file. `load` gives the text of a file
    /// that one names; `key` what each name in the files is called.
    pub fn read(doc: &str, load: &dyn Fn(&str) -> Option<String>, key: &PTable) -> Site {
        let parsed = |text: Option<String>| text.and_then(|text| serde_json::from_str::<Value>(&text).ok()).unwrap_or_default();
        let doc = parsed(Some(doc.to_string()));
        let file = |name: &str| parsed(doc.get(key.get(name)).and_then(Value::as_str).and_then(load));
        let (people, host) = (file("people"), file("host"));
        let mut read = Read { key, room: Value::Null, name: HashMap::new(), sheet: Face::default() };
        let shared = people.get(key.get("sheet")).cloned().unwrap_or_default();
        read.sheet = Face { sheet: read.text(&shared, "sheet.src"), column: read.count(&shared, "sheet.column") as u32, index: 0 };
        read.name = read.list(&people, "member").iter().map(|one| (read.text(one, "id"), read.text(one, "name"))).collect();
        read.room = people.get(key.get("room")).cloned().unwrap_or_default();
        // A face is drawn only where a sheet is named for it.
        let face = read.list(&people, "member").iter().map(|one| (read.text(one, "id"), read.face(one))).filter(|(_, face)| !face.sheet.is_empty()).collect();
        let joined: Vec<String> = read.list(&doc, "joined").iter().filter_map(Value::as_str).map(str::to_string).collect();
        let rail = read.list(&doc, "nav").into_iter().map(|item| read.venue(&item, true));
        let other = read.list(&host, "host").into_iter().map(|item| read.venue(&item, joined.contains(&read.text(&item, "id"))));
        let page = doc.get(key.get("view")).and_then(Value::as_object).cloned().unwrap_or_default();
        let view = page.iter().map(|(id, page)| (id.clone(), read.block(page))).collect();
        Site { app: read.text(&doc, "app"), start: read.text(&doc, "start"), venue: rail.chain(other).collect(), view, own: (key.get("block.kind").to_string(), key.get("block.variant").to_string()), face, extra: extra::read(&doc, &people, &host, &file("shop"), key), member: member::read(&people, key), me: read.text(&people, "me"), link: read.text(&people, "link"), from: String::new(), asked: None }
    }
}
