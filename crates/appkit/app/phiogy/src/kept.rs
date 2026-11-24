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
use std::sync::mpsc::{Sender, channel};
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
}

impl Kept {
    /// What Phiora answers at a path, where it gave the site; nothing
    /// where it did not (the files answer then).
    pub fn read(&self, path: &str) -> Option<String> {
        self.phiora.borrow().as_ref()?.get(path)
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
        let kept = Kept { doc: RefCell::new(std::mem::take(&mut doc)), word, store, post: RefCell::default(), at, phiora: RefCell::new(known), me: RefCell::default() };
        kept.fetch(false);
        kept
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
}

/// A thread that keeps on Phiora what it is sent, in order; nothing
/// where there are no threads.
fn carry(phiora: Phiora) -> Option<Carry> {
    let (post, sent) = channel::<Sent>();
    let kept = std::thread::Builder::new().spawn(move || {
        sent.iter().for_each(|sent| match sent {
            Sent::Keep(name, document) => _ = phiora.keep(&name, &document),
            Sent::Ask(path, body) => _ = phiora.post(&path, &body),
        })
    });
    kept.ok().map(|kept| (post, kept))
}
