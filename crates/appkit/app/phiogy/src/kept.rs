//! What a person keeps: their own documents, as the web keeps them
//! (`persisted` in `puikit/app/phi-kit/src/api.ts`). What they wrote in
//! each room, the marks they gave, what they pinned: each is a
//! document with a name, read when Phiogy opens and written each time
//! it changes.
//!
//! A document is kept in the person's own folder for Phiogy and, with
//! Phiora answering, on Phiora under their session, by the names the
//! web's pages use. Phiora's copy is the one believed: what it has
//! replaces what the folder has when Phiogy opens. A change goes to
//! Phiora on a thread of its own, in order, so the window never waits.
//! Where there is no folder and no Phiora, a document lasts as long as
//! the window. The names are `data/keep.tsv`.

mod who;

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::thread::JoinHandle;

use appkit::service::{PStore, PTable};
use serde_json::{Map, Value};

use crate::data;
use crate::phiora::Phiora;

/// How the row of a document's name begins in `data/keep.tsv`.
const DOC: &str = "doc.";

/// A person's own documents.
#[derive(Debug, Default)]
pub struct Kept {
    /// Each document, by the name Phiora keeps it under.
    doc: RefCell<Map<String, Value>>,
    word: PTable,
    store: Option<PStore>,
    /// What carries a change to Phiora: a name, and the document; and
    /// the thread that takes them there.
    post: RefCell<Option<Carry>>,
    /// Where Phiora keeps them, when it does.
    pub at: Option<String>,
    /// Phiora as the person's session asks it, and who it says they
    /// are.
    phiora: RefCell<Option<Phiora>>,
    me: RefCell<Value>,
    /// What everybody said, by room, as Phiora has it; kept up to date by
    /// a thread of its own while Phiora answers.
    said: Arc<Mutex<HashMap<String, Vec<Value>>>>,
    /// The room the person is in, and who they are to Phiora (its key for
    /// them): what the thread asks for.
    want: Arc<Mutex<String>>,
    mine: Arc<Mutex<String>>,
}

impl Kept {
    /// What Phiora answers at a path, where it gave the site; nothing
    /// where it did not (the files answer then).
    pub fn read(&self, path: &str) -> Option<String> {
        self.phiora.borrow().as_ref()?.get(path)
    }

    /// Make, change or take away a record on Phiora (post, patch,
    /// delete), at once, and hear whether it did and what it said;
    /// nothing where Phiora does not answer.
    pub fn record(&self, verb: crate::phiora::Verb, path: &str, body: &str) -> Option<(bool, String)> {
        self.phiora.borrow().as_ref()?.record(verb, path, body)
    }

    /// Phiora's records, by kind and name key (`get`, `has`, `set`): for the
    /// person who is signed in. `Err` is why not: nobody signed in (a guest
    /// is refused there anyway), or Phiora not answering.
    pub fn records(&self) -> Result<crate::records::Records, String> {
        let word = PTable::read(data::MAKE);
        let phiora = self.phiora.borrow().clone().ok_or_else(|| word.get("say.nowhere").to_string())?;
        match self.me().is_null() || self.guest() {
            true => Err(word.get("say.nobody").to_string()),
            false => Ok(crate::records::Records::new(phiora)),
        }
    }

    /// Set a record in its own time, on the thread that keeps the person's
    /// changes in order, so the window does not wait (what is said, a
    /// like). Where nobody is signed in, nothing is sent.
    pub fn send(&self, kind: &str, key: &str, payload: Value) {
        if self.guest() {
            return;
        }
        self.post.borrow().iter().for_each(|(post, _)| _ = post.send(Sent::Record(kind.to_string(), key.to_string(), payload.clone())));
    }

    /// Documents kept nowhere: they last as long as the window.
    pub fn none() -> Kept {
        Kept { word: PTable::read(data::KEEP), ..Kept::default() }
    }

    /// The documents of that folder and, when Phiora is given and
    /// answers, of the person's session there, which is the one kept in
    /// the folder or a new one, kept from now on.
    pub fn open(store: Option<PStore>, phiora: Option<Phiora>) -> Kept {
        let word = PTable::read(data::KEEP);
        let read = store.as_ref().and_then(|store| store.read(word.get("file")));
        let mut doc = read.and_then(|text| serde_json::from_str::<Map<String, Value>>(&text).ok()).unwrap_or_default();
        let known = phiora.and_then(|mut phiora| {
            phiora.cookie = store.as_ref().and_then(|store| store.read(word.get("session"))).unwrap_or_default().trim().to_string();
            phiora.enter().then_some(phiora)
        });
        let at = known.as_ref().map(|phiora| phiora.at.clone());
        let kept = Kept { doc: RefCell::new(std::mem::take(&mut doc)), word, store, post: RefCell::default(), at, phiora: RefCell::new(known), me: RefCell::default(), said: Arc::default(), want: Arc::default(), mine: Arc::default() };
        kept.fetch(false);
        kept.watch();
        kept
    }

    /// What was said in a room by everybody who said it on Phiora, the
    /// records as it keeps them (the latest of them); nothing where Phiora
    /// does not answer. Asking says which room the person is in, so the
    /// thread asks for that one only.
    pub fn heard(&self, room: &str) -> Vec<Value> {
        if room != BONDS && room != MARKS {
            let new = self.want.lock().map(|mut want| (*want != room).then(|| *want = room.to_string()).is_some()).unwrap_or(false);
            // A room the person has just gone to is asked for at once, so it opens with what was said in it.
            if new && self.phiora.borrow().is_some() {
                self.refresh();
            }
        }
        self.said.lock().map(|said| said.get(room).cloned().unwrap_or_default()).unwrap_or_default()
    }

    /// Who else gave a message which marks, by Phiora's records of the room
    /// the person is in: each icon with how many people (not the person
    /// themself, whose own is theirs) gave it.
    pub fn marked(&self, message: &str, me: &str) -> Vec<(String, u64)> {
        let Ok(said) = self.said.lock() else { return Vec::new() };
        let mut out: Vec<(String, u64)> = Vec::new();
        for row in said.get(MARKS).into_iter().flatten().filter(|row| row["messageId"] == message && row["state"] == "on" && row["ownerId"].as_str() != Some(me)) {
            let icon = row["icon"].as_str().unwrap_or_default().to_string();
            match out.iter_mut().find(|(known, _)| *known == icon) {
                Some((_, count)) => *count += 1,
                None => out.push((icon, 1)),
            }
        }
        out
    }
    /// Ask Phiora what has been said in the room the person is in and how
    /// they stand with people, once now (so a room opens with it) and then
    /// when the room changes and every few seconds, on a thread of its
    /// own. The thread ends with the window.
    fn watch(&self) {
        let every = self.word.get("watch").parse::<u64>().unwrap_or_default();
        let (Some(phiora), true) = (self.phiora.borrow().clone(), every > 0) else { return };
        self.refresh();
        let (said, want, mine, word, make) = (Arc::clone(&self.said), Arc::clone(&self.want), Arc::clone(&self.mine), self.word.clone(), PTable::read(data::MAKE));
        _ = std::thread::Builder::new().spawn(move || {
            let (mut last, mut asked, began) = (String::new(), 0u64, std::time::Instant::now());
            while Arc::strong_count(&said) > 1 {
                std::thread::sleep(Duration::from_millis(200));
                let (room, me) = (want.lock().map(|room| room.clone()).unwrap_or_default(), mine.lock().map(|me| me.clone()).unwrap_or_default());
                let now = began.elapsed().as_millis() as u64;
                // At once when the person went to another room, else every few seconds.
                if room != last || now.saturating_sub(asked) >= every {
                    hear(&phiora, &word, &make, &room, &me, &said);
                    (last, asked) = (room, now);
                }
            }
        });
    }

    /// Ask Phiora now what has been said in the room the person is in and
    /// how they stand, as the thread does: after the person changed
    /// something.
    pub fn refresh(&self) {
        let Some(phiora) = self.phiora.borrow().clone() else { return };
        let (room, me) = (self.want.lock().map(|room| room.clone()).unwrap_or_default(), self.mine.lock().map(|me| me.clone()).unwrap_or_default());
        hear(&phiora, &self.word, &PTable::read(data::MAKE), &room, &me, &self.said);
    }
    /// Read what Phiora has for the session: who the person is, and
    /// each of their documents. What Phiora has is what is true; what
    /// it never had stays, unless the person is another one now
    /// (`fresh`), who begins with what Phiora has alone. Changes go
    /// there from now on under that session, which the folder keeps.
    fn fetch(&self, fresh: bool) {
        let Some(phiora) = self.phiora.borrow().clone() else { return };
        self.store.iter().for_each(|store| _ = store.write(self.word.get("session"), &phiora.cookie));
        *self.me.borrow_mut() = phiora.me().and_then(|text| serde_json::from_str(&text).ok()).unwrap_or_default();
        // Who they are to Phiora, unless a guest (a guest asks nobody), for the thread that asks it for what is theirs.
        let mine = self.me.borrow()[self.word.get("me.id")].as_str().filter(|_| self.me.borrow()[self.word.get("me.guest")] != true).unwrap_or_default().to_string();
        if let Ok(mut key) = self.mine.lock() {
            *key = mine;
        }
        let mut doc = match fresh {
            true => Map::new(),
            false => self.doc.borrow().clone(),
        };
        let name = self.word.list().iter().filter(|(name, _)| name.starts_with(DOC)).map(|(_, name)| name.clone());
        for name in name {
            let had = phiora.state(&name).and_then(|text| serde_json::from_str::<Value>(&text).ok());
            had.into_iter().for_each(|value| _ = doc.insert(name.clone(), value));
        }
        let all = Value::Object(doc.clone()).to_string();
        self.store.iter().for_each(|store| _ = store.write(self.word.get("file"), &all));
        *self.doc.borrow_mut() = doc;
        *self.post.borrow_mut() = carry(phiora);
    }

    /// The name Phiora keeps a document under.
    fn name(&self, doc: &str) -> String {
        self.word.get(&format!("{DOC}{doc}")).to_string()
    }

    /// A document; nothing where none was kept.
    pub fn get(&self, doc: &str) -> Value {
        self.doc.borrow().get(&self.name(doc)).cloned().unwrap_or_default()
    }

    /// Keep a document: here, in the folder, and on Phiora.
    pub fn set(&self, doc: &str, value: Value) {
        let name = self.name(doc);
        // What is Phiora's to write is never sent to it as a document.
        let server = self.word.get("server").split(' ').any(|known| known == name);
        self.post.borrow().iter().filter(|_| !server).for_each(|(post, _)| _ = post.send(Sent::Keep(name.clone(), value.to_string())));
        self.doc.borrow_mut().insert(name, value);
        let all = Value::Object(self.doc.borrow().clone()).to_string();
        self.store.iter().for_each(|store| _ = store.write(self.word.get("file"), &all));
    }

    /// Ask Phiora for something to be done, by what `data/keep.tsv`
    /// calls it: with Phiora answering it goes there in order with
    /// the rest; without, nothing is asked.
    pub fn ask(&self, name: &str, body: Value) {
        let path = self.word.get(&format!("ask.{name}")).to_string();
        self.post.borrow().iter().for_each(|(post, _)| _ = post.send(Sent::Ask(path.clone(), body.to_string())));
    }

    /// Wait until Phiora has all that changed. After it nothing more
    /// goes there: it is for a window that is closing.
    pub fn settle(&self) {
        if let Some((post, kept)) = self.post.borrow_mut().take() {
            drop(post);
            _ = kept.join();
        }
    }

    /// What a document that is a list of things by name says of one:
    /// nothing when it was never said.
    pub fn flag(&self, doc: &str, key: &str) -> Option<bool> {
        self.get(doc).get(key).and_then(Value::as_bool)
    }

    /// Say something of one thing in such a document, and keep it.
    pub fn put(&self, doc: &str, key: &str, value: Value) {
        let mut all = self.get(doc).as_object().cloned().unwrap_or_default();
        all.insert(key.to_string(), value);
        self.set(doc, Value::Object(all));
    }
}

/// What carries changes to Phiora.
type Carry = (Sender<Sent>, JoinHandle<()>);

/// What goes to Phiora: a document to keep under a name, or a body
/// sent to a path, asking for something to be done.
enum Sent {
    Keep(String, String),
    Ask(String, String),
    Record(String, String, Value),
}

/// A thread that keeps on Phiora what it is sent, in order; nothing
/// where there are no threads.
fn carry(phiora: Phiora) -> Option<Carry> {
    let (post, sent) = channel::<Sent>();
    let kept = std::thread::Builder::new().spawn(move || {
        sent.iter().for_each(|sent| match sent {
            Sent::Keep(name, document) => _ = phiora.keep(&name, &document),
            Sent::Ask(path, body) => _ = phiora.post(&path, &body),
            Sent::Record(kind, key, payload) => _ = crate::records::Records::new(phiora.clone()).set(&kind, &key, payload),
        })
    });
    kept.ok().map(|kept| (post, kept))
}

/// What was said in a room (the latest messages of it) and, under `BONDS`,
/// the requests the person made and was made: as Phiora has it now, put in
/// what was heard. Phiora is asked for the one room and the one person, not
/// for everything there is.
fn hear(phiora: &Phiora, word: &PTable, make: &PTable, room: &str, me: &str, said: &Mutex<HashMap<String, Vec<Value>>>) {
    let enc = crate::make::enc;
    let rows = |path: String| {
        let tree = phiora.get(&path).and_then(|text| serde_json::from_str::<Value>(&text).ok())?;
        Some(tree[make.get("answer.sources")].as_array()?.iter().filter_map(|source| source[make.get("answer.rows")].as_array()).flatten().cloned().collect::<Vec<Value>>())
    };
    let most = word.get("most");
    // A Phiora that does not narrow what it is asked (an older one) sends everything: it is narrowed here too.
    let said_in = (!room.is_empty()).then(|| rows(format!("{}&where=room:{}&limit={most}", word.get("list.message"), enc(room)))).flatten().map(|rows| rows.into_iter().filter(|row| row["room"] == room).collect::<Vec<_>>());
    // The marks given in that room (a like on a message), for every message there is at once.
    let marks = (!room.is_empty()).then(|| rows(format!("{}&where=room:{}&limit=2000", word.get("list.mark"), enc(room)))).flatten().map(|rows| rows.into_iter().filter(|row| row["room"] == room).collect::<Vec<_>>());
    let bonds = (!me.is_empty()).then(|| {
        let mut all = rows(format!("{}&where=from:{}", word.get("list.bond"), enc(me)))?;
        all.extend(rows(format!("{}&where=to:{}", word.get("list.bond"), enc(me)))?);
        all.retain(|row| row["from"] == me || row["to"] == me);
        Some(all)
    }).flatten();
    if let Ok(mut said) = said.lock() {
        said_in.into_iter().for_each(|rows| _ = said.insert(room.to_string(), rows));
        marks.into_iter().for_each(|rows| _ = said.insert(MARKS.to_string(), rows));
        bonds.into_iter().for_each(|rows| _ = said.insert(BONDS.to_string(), rows));
    }
}
/// Where the marks given in the room the person is in are kept among what was heard.
pub const MARKS: &str = "~mark";

/// Where how people stand is kept among what was heard.
pub const BONDS: &str = "~bond";