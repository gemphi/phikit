//! A room's talk, as the web's room has it
//! (`puikit/app/phi-kit/src/blocks/RoomStream.tsx`): what the site says
//! was said in it, and after that what the person wrote; which
//! messages are pinned, which marks the person gave, and which
//! messages hold what is searched for. What the person adds is their
//! own documents (`kept.rs`), under the room's name on the line
//! (`app/room`), spelled as the web spells them.

use serde_json::{Value, json};

use appkit::service::PTable;

use super::Desk;
use crate::data;
use crate::hour;
use crate::site::{Message, spoken};

/// The person's own documents a room's talk is made of.
pub const SENT: &str = "sent";
pub const MARK: &str = "mark";
pub const PIN: &str = "pin";
pub const READ: &str = "read";
pub const FOLLOW: &str = "follow";
pub const QUIET: &str = "quiet";

/// A mark on a message as it reads now: its icon, how many gave it,
/// the person among them, and whether the person did.
pub type Given = (String, u64, bool);

impl Desk {
    /// Where the open room's own writing, marks and pins are kept.
    pub fn store(&self) -> String {
        self.key(&self.room())
    }

    /// Everything said in the open room: what the site has, then what
    /// the person wrote, each by the name of who said it.
    pub fn said(&self) -> Vec<Message> {
        let key = PTable::read(data::SITE);
        let room = self.room();
        let site = self.plate(&room).map(|plate| plate.message).unwrap_or_default();
        let own = self.kept.get(SENT).get(self.store()).and_then(Value::as_array).cloned().unwrap_or_default();
        let name = |id: &str| self.site.member(id).map_or_else(|| id.to_string(), |member| member.name.clone());
        let own = own.iter().map(|one| spoken(one, &key)).map(|one| Message { by: name(&one.who), ..one }).collect::<Vec<_>>();
        // What other people wrote in it, as Phiora has it (not what this person wrote: that is theirs, above).
        let me = self.kept.me();
        let mine = me[self.kept.word("me.id")].as_str().unwrap_or_default().to_string();
        let text = |row: &Value, name: &str| row[name].as_str().unwrap_or_default().to_string();
        let today = hour::now("%Y-%m-%d");
        let others = self.kept.heard(&self.store()).into_iter().filter(|row| text(row, "ownerId") != mine && !own.iter().any(|one| one.id == text(row, "ref"))).map(|row| {
            let at = moment_of(&text(&row, "ref"));
            let day = Some(hour::of(at, "%Y-%m-%d")).map(|day| if day == today { self.text.get("stream.today").to_string() } else { day }).unwrap_or_default();
            let who = text(&row, "ownerId");
            let by = Some(text(&row, "authorName")).filter(|by| !by.is_empty()).unwrap_or_else(|| name(&who));
            Message { id: text(&row, "ref"), by, who, day, at: hour::of(at, self.text.get("stream.hour")), text: text(&row, "text"), reply: text(&row, "reply"), ..Message::default() }
        }).collect::<Vec<_>>();
        let mut said: Vec<Message> = own.into_iter().chain(others).collect();
        said.sort_by_key(|one| moment_of(&one.id));
        site.into_iter().chain(said).collect()
    }

    /// Write something in the open room, answering the message the
    /// person chose to answer, if they chose one. `false` for nothing
    /// but spaces.
    pub fn write(&self, text: &str) -> bool {
        !text.trim().is_empty() && self.say(text.trim(), None)
    }

    /// Post a picture in the open room: a file of this disk, named by
    /// where it is. `false` for a file that is not there, or is no
    /// picture by its ending.
    pub fn post(&self, path: &str) -> bool {
        let key = PTable::read(data::SITE);
        let file = std::path::Path::new(path.trim());
        let ending = file.extension().map(|ending| ending.to_string_lossy().to_lowercase()).unwrap_or_default();
        let known = file.is_file() && key.get("posted.kind").split(' ').any(|kind| kind == ending);
        known && self.say("", Some(format!("{}{}", key.get("posted"), path.trim().replace('\\', "/").trim_start_matches('/'))))
    }

    /// Say something in the open room: words, a picture, or both.
    fn say(&self, words: &str, picture: Option<String>) -> bool {
        let key = PTable::read(data::SITE);
        let answer = std::mem::take(&mut *self.now.answer.borrow_mut());
        let mut one = json!({
            key.get("id"): format!("{}{}", key.get("sent.id"), hour::moment()),
            key.get("by"): self.site.me,
            key.get("day"): self.text.get("stream.today"),
            key.get("at"): hour::now(self.text.get("stream.hour")),
            key.get("text"): words,
        });
        if !answer.is_empty() {
            one[key.get("reply")] = json!(answer);
        }
        picture.into_iter().for_each(|picture| one[key.get("image")] = json!([picture]));
        // What was said in words is also made a record on Phiora, so the others in the room read it; a guest's is
        // refused there and stays with them.
        if !words.is_empty() {
            let me = self.kept.me();
            let by = me[self.kept.word("me.name")].as_str().unwrap_or_default().to_string();
            let id = one[key.get("id")].as_str().unwrap_or_default().to_string();
            self.kept.send("message", &id, json!({ "room": self.store(), "text": words, "reply": one[key.get("reply")], "authorName": by }));
        }
        let mut all = self.kept.get(SENT).get(self.store()).and_then(Value::as_array).cloned().unwrap_or_default();
        all.push(one);
        self.kept.put(SENT, &self.store(), Value::Array(all));
        true
    }

    /// Whether a message is among the room's pinned: as the person
    /// last said, or as the site has it.
    pub fn pinned(&self, one: &Message) -> bool {
        self.kept.flag(PIN, &format!("{}/{}", self.store(), one.id)).unwrap_or(one.pin)
    }

    /// Pin a message, or unpin it.
    pub fn pin(&self, one: &Message) {
        self.kept.put(PIN, &format!("{}/{}", self.store(), one.id), json!(!self.pinned(one)));
    }

    /// Whether the person gave a message that mark.
    pub fn gave(&self, id: &str, icon: &str) -> bool {
        self.kept.flag(MARK, &format!("{}/{id}/{icon}", self.store())).unwrap_or(false)
    }

    /// Give a message a mark, or take it back.
    pub fn give(&self, id: &str, icon: &str) {
        let on = !self.gave(id, icon);
        self.kept.put(MARK, &format!("{}/{id}/{icon}", self.store()), json!(on));
        // And on Phiora, so the others in the room count it.
        if let Some(me) = self.my_key() {
            let room = self.store();
            self.kept.send("mark", &format!("mk:{me}:{room}:{id}:{icon}"), json!({ "room": room, "messageId": id, "icon": icon, "state": if on { "on" } else { "off" } }));
        }
    }

    /// The mark a like is.
    pub fn like(&self) -> String {
        PTable::read(data::SITE).get("like").to_string()
    }

    /// The marks on a message as they read now: the site's, each
    /// counting the person when they gave it, and a like of theirs
    /// where the site has none.
    pub fn mark(&self, one: &Message) -> Vec<Given> {
        let like = self.like();
        let mut list: Vec<Given> = one.mark.iter().map(|(icon, count)| (icon.clone(), *count, self.gave(&one.id, icon))).collect();
        if !list.iter().any(|(icon, ..)| *icon == like) && self.gave(&one.id, &like) {
            list.push((like, 0, true));
        }
        // What others gave it on Phiora counts with the rest.
        let others = self.kept.marked(&one.id, &self.my_key().unwrap_or_default());
        for (icon, n) in others {
            match list.iter_mut().find(|(known, ..)| *known == icon) {
                Some((_, count, _)) => *count += n,
                None => {
                    let on = self.gave(&one.id, &icon);
                    list.push((icon, n, on));
                }
            }
        }
        list.into_iter().map(|(icon, count, on)| (icon, count + on as u64, on)).collect()
    }

    /// The messages that hold what is searched for, in their words or
    /// in the name of who said them; all of them when nothing is.
    pub fn sought(&self, find: &str) -> Vec<Message> {
        let wanted = find.trim().to_lowercase();
        let holds = |one: &Message| wanted.is_empty() || one.text.to_lowercase().contains(&wanted) || one.by.to_lowercase().contains(&wanted);
        self.said().into_iter().filter(holds).collect()
    }

    /// How many messages wait in a room: what the site says, until the
    /// person has opened it.
    pub fn waiting(&self, room: &str) -> u64 {
        let unread = self.plate(room).map_or(0, |plate| plate.unread);
        match self.kept.flag(READ, &self.key(room)) {
            Some(true) => 0,
            _ => unread,
        }
    }

    /// Whether the person muted somebody: what they said is not shown
    /// until they are unmuted. And mute them, or unmute.
    pub fn muted(&self, id: &str) -> bool {
        self.kept.flag(QUIET, id).unwrap_or(false)
    }

    pub fn mute(&self, id: &str) {
        self.kept.put(QUIET, id, json!(!self.muted(id)));
    }

    /// Whether the person follows somebody; and follow them, or stop.
    pub fn follows(&self, id: &str) -> bool {
        self.kept.flag(FOLLOW, id).unwrap_or(false)
    }

    pub fn follow(&self, id: &str) {
        let on = !self.follows(id);
        self.kept.put(FOLLOW, id, json!(on));
        // Told to Phiora too (the person followed, by their key there if they are one of its persons).
        if let Some(me) = self.my_key() {
            let them = Self::person_key(id).unwrap_or_else(|| id.to_string());
            self.kept.send("follow", &format!("fl:{me}>{them}"), json!({ "from": me, "to": them, "state": if on { "on" } else { "off" } }));
        }
    }
}

/// When a message of this window was said: the moment its id ends with.
fn moment_of(id: &str) -> u128 {
    id.trim_start_matches(|one: char| !one.is_ascii_digit()).parse().unwrap_or_default()
}
