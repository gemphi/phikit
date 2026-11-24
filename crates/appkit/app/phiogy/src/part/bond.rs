//! Friends, and the conversations a person began, as the web keeps
//! them (`puikit/app/phi-kit/src/friends.tsx`, `direct.tsx`). How the
//! person and somebody stand is one of four: friends, the person
//! asked, they asked, or nothing. The rooms' file says how it starts;
//! what the person does since is their own document and wins. Nobody
//! is on the other side yet, so a request that was sent waits.

use serde_json::{Value, json};

use super::Desk;
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
    /// How the person and somebody stand, given what the person said.
    fn stand(&self, said: &Value, id: &str) -> String {
        let extra = &self.site.extra;
        let from = match (extra.friend.iter().any(|known| known == id), extra.asked.iter().any(|known| known == id)) {
            (true, _) => WITH,
            (false, true) => ASKED,
            (false, false) => NONE,
        };
        said.get(id).and_then(Value::as_str).unwrap_or(from).to_string()
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
    }

    /// Everyone the person stands so with, never themself.
    pub fn bonded(&self, bond: &str) -> Vec<Member> {
        let said = self.kept.get(FRIEND)[&self.site.app].clone();
        self.site.member.iter().filter(|member| member.id != self.site.me && self.stand(&said, &member.id) == bond).cloned().collect()
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
