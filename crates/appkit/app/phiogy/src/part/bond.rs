//! Friends, and the conversations a person began, as the web keeps
//! them (`puikit/app/phi-kit/src/friends.tsx`, `direct.tsx`). How the
//! person and somebody stand is one of four: friends, the person
//! asked, they asked, or nothing. The rooms' file says how it starts;
//! what the person does since is their own document and wins. Nobody
//! is on the other side yet, so a request that was sent waits.
//!
//! Where both are people of Phiora (a signed-in person, and a member
//! that is one of Phiora's persons), a request is a record on Phiora
//! that the other can answer, and how they stand is what those records
//! say; with anybody else it stays the person's own document.

use serde_json::{Value, json};

use super::Desk;
use crate::kept::BONDS;
use crate::site::Member;

/// The person's own documents these are kept in.
pub const FRIEND: &str = "friend";
pub const BEGUN: &str = "direct";
/// The four ways two people stand, in the web's own words.
pub const WITH: &str = "with";
pub const SENT: &str = "sent";
pub const ASKED: &str = "asked";
pub const NONE: &str = "none";

/// What one press does from a bond: what it leads to, and the row of
/// the words that names it.
pub fn next(bond: &str) -> (&'static str, &'static str) {
    match bond {
        SENT => (NONE, "friend.cancel"),
        ASKED => (WITH, "friend.accept"),
        WITH => (NONE, "friend.remove"),
        _ => (SENT, "friend.ask"),
    }
}

impl Desk {
    /// Phiora's key of a member who is one of its persons: a seeded
    /// person's id has an underscore (`u_ines`); the md5 of `kind:id`.
    pub(crate) fn person_key(id: &str) -> Option<String> {
        use md5::{Digest, Md5};
        id.contains('_').then(|| format!("{:x}", Md5::digest(format!("person:{id}").as_bytes())))
    }

    /// The person's own key on Phiora; nothing for a guest or where it
    /// does not answer.
    pub(crate) fn my_key(&self) -> Option<String> {
        let me = self.kept.me();
        Some(me[self.kept.word("me.id")].as_str()?.to_string()).filter(|id| !id.is_empty() && !self.kept.guest())
    }

    /// How they stand by the requests on Phiora, where there are some
    /// between the two; nothing where there are none.
    fn heard_stand(&self, rows: &[Value], mine: &str, id: &str) -> Option<&'static str> {
        let other = Self::person_key(id)?;
        let state = |from: &str, to: &str| rows.iter().find(|row| row["from"] == from && row["to"] == to).and_then(|row| row["state"].as_str().map(str::to_string));
        match (state(mine, &other).as_deref(), state(&other, mine).as_deref()) {
            (Some("with"), _) | (_, Some("with")) => Some(WITH),
            (_, Some("sent")) => Some(ASKED),
            (Some("sent"), _) => Some(SENT),
            (None, None) => None,
            _ => Some(NONE),
        }
    }

    /// How the person and somebody stand, given what the person said.
    fn stand(&self, said: &Value, id: &str) -> String {
        let rows = self.kept.heard(BONDS);
        match (self.my_key(), rows.is_empty()) {
            (Some(mine), false) => self.stand_by(said, id, &rows, Some(&mine)),
            _ => self.stand_by(said, id, &rows, None),
        }
    }

    fn stand_by(&self, said: &Value, id: &str, rows: &[Value], mine: Option<&str>) -> String {
        if let Some(stood) = mine.and_then(|mine| self.heard_stand(rows, mine, id)) {
            return stood.to_string();
        }
        let extra = &self.site.extra;
        let from = match (extra.friend.iter().any(|known| known == id), extra.asked.iter().any(|known| known == id)) {
            (true, _) => WITH,
            (false, true) => ASKED,
            (false, false) => NONE,
        };
        said.get(id).and_then(Value::as_str).unwrap_or(from).to_string()
    }

    /// Tell Phiora how the person stands with somebody now, where both are
    /// its people: a request is a record the other can answer (their id is
    /// its `userId`), and the answer changes it.
    fn share_bond(&self, id: &str, bond: &str) {
        let (Some(mine), Some(other)) = (self.my_key(), Self::person_key(id)) else { return };
        let rows = self.kept.heard(BONDS);
        let has = |from: &str, to: &str| rows.iter().any(|row| row["from"] == from && row["to"] == to);
        let mut set: Vec<(String, String, &str)> = Vec::new();
        match bond {
            SENT => set.push((mine, other, "sent")),
            WITH if has(&other, &mine) => set.push((other, mine, "with")),
            WITH => set.push((mine, other, "with")),
            _ => {
                if has(&mine, &other) {
                    set.push((mine.clone(), other.clone(), "none"));
                }
                if has(&other, &mine) {
                    set.push((other, mine, "none"));
                }
            }
        }
        // Each is a record of its own, named by who asked whom; the one asked may answer it (`userId`).
        if let Ok(records) = self.kept.records() {
            for (from, to, state) in set {
                _ = records.set("bond", &format!("{from}>{to}"), json!({ "from": from, "to": to, "userId": to, "state": state }));
            }
        }
        self.kept.refresh();
    }

    /// How the person and somebody stand.
    pub fn bond(&self, id: &str) -> String {
        self.stand(&self.kept.get(FRIEND)[&self.site.app], id)
    }

    /// Say how the person and somebody stand from now on.
    pub fn bind(&self, id: &str, bond: &str) {
        let mut own = self.kept.get(FRIEND)[&self.site.app].as_object().cloned().unwrap_or_default();
        own.insert(id.to_string(), json!(bond));
        self.kept.put(FRIEND, &self.site.app, Value::Object(own));
        self.share_bond(id, bond);
    }

    /// Everyone the person stands so with, never themself.
    pub fn bonded(&self, bond: &str) -> Vec<Member> {
        let said = self.kept.get(FRIEND)[&self.site.app].clone();
        // What Phiora says of the requests is read once for everybody, not once for each.
        let rows = self.kept.heard(BONDS);
        let mine = self.my_key().filter(|_| !rows.is_empty());
        self.site.member.iter().filter(|member| member.id != self.site.me && self.stand_by(&said, &member.id, &rows, mine.as_deref()) == bond).cloned().collect()
    }

    /// The people the person began a conversation with, newest first.
    pub fn begun(&self) -> Vec<String> {
        let all = self.kept.get(BEGUN)[&self.site.app].as_array().cloned().unwrap_or_default();
        all.iter().filter_map(Value::as_str).map(str::to_string).collect()
    }

    /// Begin a conversation with somebody, once.
    pub fn begin(&self, person: &str) {
        let mut all = self.begun();
        if !all.iter().any(|known| known == person) {
            all.insert(0, person.to_string());
            self.kept.put(BEGUN, &self.site.app, json!(all));
        }
    }

    /// The people whose name or handle holds what was typed, never the
    /// person; the first of everyone when nothing was.
    pub fn seek(&self, find: &str, most: usize) -> Vec<Member> {
        // As search ranks (search/rank.rs): every word sought begins a word
        // of their name or handle, the best first.
        let others = self.site.member.iter().filter(|member| member.id != self.site.me);
        super::search::rank::sift(others, find, most, |member| (member.name.clone(), member.handle.clone())).into_iter().cloned().collect()
    }
}
