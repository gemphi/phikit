//! Every call this window was in, kept (the user, 25 November 2026: "each
//! call is recorded"): the room, what it is called and the host it is of,
//! when it began, whether a camera was on, who was in it and how long it
//! lasted, the newest first, in the person's own documents as their
//! friends and hosts are. What is kept is that the call was, never what
//! was said or seen in it: nothing of its sound or picture is recorded.

use appkit::service::PTable;
use serde_json::{Value, json};

use super::Desk;
use crate::hour;

/// The person's own document the calls are kept in, and how many it keeps.
pub const CALL: &str = "call";
const MOST: usize = 64;

/// One call, as kept: a past one, or the one going on.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Past {
    pub room: String,
    pub label: String,
    pub host: String,
    /// When it began, as the person reads it.
    pub when: String,
    pub video: bool,
    pub people: Vec<String>,
    pub seconds: u64,
    /// Whether it is still going on (this window is in it).
    pub open: bool,
}

impl Past {
    fn read(value: &Value) -> Past {
        let word = |name: &str| value[name].as_str().unwrap_or_default().to_string();
        Past {
            room: word("room"),
            label: word("label"),
            host: word("host"),
            when: word("when"),
            video: value["video"].as_bool().unwrap_or_default(),
            people: value["people"].as_array().into_iter().flatten().filter_map(Value::as_str).map(str::to_string).collect(),
            seconds: value["seconds"].as_u64().unwrap_or_default(),
            open: value["open"].as_bool().unwrap_or_default(),
        }
    }

    fn value(&self) -> Value {
        json!({"room": self.room, "label": self.label, "host": self.host, "when": self.when, "video": self.video, "people": self.people, "seconds": self.seconds, "open": self.open})
    }

    /// The call as a player card's row (data/listing.tsv `aside.calls`):
    /// its room, when it was, an icon where there is no picture or face,
    /// the picture, who was in it, how long it lasted or that it is on
    /// (`live`, this window is in it now), the face, its level, and Join
    /// to call again, or Leave while it is on.
    pub fn card(&self, text: &PTable, picture: String, face: String, live: bool) -> Vec<String> {
        let icon = match (picture.is_empty() && face.is_empty(), self.video) {
            (false, _) => "",
            (true, true) => "video",
            (true, false) => "phone",
        };
        let line = match self.people.is_empty() {
            true => text.get("call.alone").to_string(),
            false => text.get("call.with").replace("{who}", &self.people.join(", ")),
        };
        let (end, deed) = match self.open && live {
            true => (text.get("call.now").to_string(), "leave:phone-off:1"),
            false => (format!("{}:{:02}", self.seconds / 60, self.seconds % 60), "join:phone"),
        };
        vec![self.room.clone(), self.when.clone(), icon.to_string(), picture, line, end, face, "1".to_string(), deed.to_string()]
    }
}

impl Desk {
    /// Every call kept, the newest first.
    pub fn history(&self) -> Vec<Past> {
        self.kept.get(CALL)[&self.site.app].as_array().into_iter().flatten().map(Past::read).collect()
    }

    fn keep_history(&self, all: &[Past]) {
        let list: Vec<Value> = all.iter().take(MOST).map(Past::value).collect();
        self.kept.put(CALL, &self.site.app, Value::Array(list));
    }

    /// A call this window began or joined, in a room, now.
    pub fn began(&self, room: &str) {
        let label = self.plate(room).map(|plate| plate.label).unwrap_or_default();
        let host = self.host_of(room).map(|(_, label)| label).unwrap_or_default();
        let one = Past { room: room.to_string(), label, host, when: hour::now(self.text.get("call.when")), open: true, ..Past::default() };
        let mut all = self.history();
        all.insert(0, one);
        self.keep_history(&all);
    }

    /// The call this window is in, as it goes on: who is in it (by the
    /// names they call themselves), whether a camera was on, and for how
    /// long; and whether it is over.
    pub fn lasted(&self, people: &[String], video: bool, seconds: u64, over: bool) {
        let mut all = self.history();
        let Some(one) = all.iter_mut().find(|one| one.open) else { return };
        let new: Vec<String> = people.iter().filter(|name| !one.people.contains(name)).cloned().collect();
        one.people.extend(new);
        one.video |= video;
        one.seconds = one.seconds.max(seconds);
        one.open = !over;
        self.keep_history(&all);
    }
}
