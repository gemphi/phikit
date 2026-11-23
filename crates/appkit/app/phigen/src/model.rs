//! The model that answers: a server on this computer that takes a chat
//! and writes its answer a piece at a time, as Ollama does. Which server,
//! which path and which model are `data/model.tsv`; nothing of a chat
//! leaves the computer.
//!
//! A line of the answer is a JSON object: `message.content` is the next
//! piece, `done` says it was the last, and `error` says why there is none.
//!
//! The server also says which models it has, at the path the table gives
//! as `list`: one object whose `models` each have a `name`. It is asked
//! the first time anyone wants to know, and any of them can be chosen.

use appkit::service::{PCall, PStream, PTable};
use serde_json::{Value, json};

use crate::chat::{Turn, Who};
use crate::reply::{Reply, Source};

/// The kind `data/model.tsv` names for a server that speaks as Ollama does.
pub const OLLAMA: &str = "ollama";
/// The tag such a server puts after a model's name when none was given:
/// the name without it is the same model.
pub const LATEST: &str = ":latest";

/// A model on a server.
pub struct Model {
    call: PCall,
    path: String,
    /// The model's name on the server, which is also what the profile shows.
    pub name: String,
    /// What it is told before the chat: who it is.
    system: String,
    /// What is said when it cannot answer, with `{why}`.
    fail: String,
    /// The models the server has, as far as is known: this one, and the
    /// others once the server has said.
    known: Vec<String>,
    /// Where the server says which models it has, until it is asked.
    tag: Option<String>,
    /// Its answer to that, while it is awaited.
    asked: Option<PStream>,
}

impl Model {
    /// The model `data/model.tsv` names, told what `data/text.tsv` says.
    /// `None` when the table names no server of a kind known here: then
    /// the stand-in answers.
    pub fn read(model: &PTable, text: &PTable) -> Option<Model> {
        let known = model.get("kind") == OLLAMA && !model.get("host").is_empty();
        let (name, tag) = (model.get("name").to_string(), Some(model.get("list").to_string()).filter(|path| !path.is_empty()));
        known.then(|| Model { call: PCall::to(model.get("host")), path: model.get("path").to_string(), known: vec![name.clone()], name, system: text.get("model.system").to_string(), fail: text.get("reply.fail").to_string(), tag, asked: None })
    }

    /// Take in the server's answer to which models it has, if it has
    /// come: each is known from then on.
    fn learn(&mut self) {
        let Some(end) = self.asked.as_ref().and_then(PStream::end) else { return };
        let line = self.asked.take().map(|stream| stream.take()).unwrap_or_default();
        let said = line.iter().filter_map(|line| serde_json::from_str::<Value>(line).ok()).filter(|_| end.is_ok());
        let named = said.flat_map(|value| value["models"].as_array().cloned().unwrap_or_default());
        for model in named {
            let name = model["name"].as_str().unwrap_or_default();
            let name = name.strip_suffix(LATEST).unwrap_or(name).to_string();
            if !name.is_empty() && !self.known.contains(&name) {
                self.known.push(name);
            }
        }
    }
}

impl Reply for Model {
    fn reply(&mut self, said: &str, past: &[Turn]) -> Box<dyn Source> {
        let role = |who: Who| match who {
            Who::Person => "user",
            Who::Phigen => "assistant",
        };
        // The greeting is Phigen's own, not something the model said.
        let earlier = past.iter().skip_while(|turn| turn.who == Who::Phigen).map(|turn| json!({ "role": role(turn.who), "content": turn.text }));
        let first = json!({ "role": "system", "content": self.system });
        let last = json!({ "role": "user", "content": said });
        let message: Vec<Value> = std::iter::once(first).chain(earlier).chain(std::iter::once(last)).collect();
        let body = json!({ "model": self.name, "stream": true, "messages": message }).to_string();
        Box::new(Heard { stream: self.call.post(&self.path, &body), fail: self.fail.clone() })
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn list(&mut self) -> Vec<String> {
        // The server is asked once, the first time anyone wants to know.
        if let Some(path) = self.tag.take() {
            self.asked = Some(self.call.get(&path));
        }
        self.learn();
        self.known.clone()
    }

    fn asking(&self) -> bool {
        self.asked.is_some()
    }

    /// Any name can be chosen: one the server does not have says so when
    /// it is asked to answer.
    fn choose(&mut self, name: &str) -> bool {
        let named = !name.is_empty();
        if named {
            self.name = name.to_string();
            if !self.known.iter().any(|known| known == name) {
                self.known.push(name.to_string());
            }
        }
        named
    }
}

/// An answer arriving from the server.
struct Heard {
    stream: PStream,
    fail: String,
}

impl Source for Heard {
    fn more(&mut self) -> (String, bool) {
        // Its end is asked for first: every line is on the stream before
        // its end is, so none is missed.
        let end = self.stream.end();
        let (mut text, mut done, mut wrong) = (String::new(), end.is_some(), None);
        for line in self.stream.take() {
            let Ok(value) = serde_json::from_str::<Value>(&line) else { continue };
            text.push_str(value["message"]["content"].as_str().unwrap_or_default());
            done |= value["done"].as_bool().unwrap_or(false);
            wrong = wrong.or(value["error"].as_str().map(str::to_string));
        }
        // What the server said was wrong, or why it could not be reached.
        let why = wrong.or(end.and_then(Result::err));
        match why {
            Some(why) => (text + &self.fail.replace("{why}", &why), true),
            None => (text, done),
        }
    }

    fn stop(&mut self) {
        self.stream.stop();
    }
}
