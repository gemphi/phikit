//! The history: every chat, the newest first, and the one that is open.

use appkit::service::PStore;

use crate::chat::{Chat, Turn, Who};
use crate::reply::{Reply, Source};

/// Every chat, the newest first, and the one that is open.
pub struct History {
    chat: Vec<Chat>,
    open: usize,
    reply: Box<dyn Reply>,
    greeting: String,
    /// Where the chats are kept, and in which file, if they are.
    keep: Option<(PStore, String)>,
    /// The answer that is still arriving, and the number of its chat.
    pending: Option<(Box<dyn Source>, usize)>,
    /// What time it is, in seconds since 1970.
    clock: fn() -> u64,
}

/// The system's time. A browser has no clock that can be read this way,
/// so there a chat has no day.
fn system() -> u64 {
    match cfg!(target_family = "wasm") {
        true => 0,
        false => std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |since| since.as_secs()),
    }
}

impl History {
    /// A history with one chat, greeted and open.
    pub fn new(reply: Box<dyn Reply>, greeting: &str) -> History {
        History { chat: vec![Chat::greeted(greeting)], open: 0, reply, greeting: greeting.to_string(), keep: None, pending: None, clock: system }
    }

    /// The same history, kept in a file of that store from now on: every
    /// time something is said the file is written again (`keep.rs`).
    pub fn kept(self, store: PStore, name: &str) -> History {
        History { keep: Some((store, name.to_string())), ..self }
    }

    /// The same history, told the time by another clock.
    pub fn timed(self, clock: fn() -> u64) -> History {
        History { clock, ..self }
    }

    /// What time it is, in seconds since 1970.
    pub fn now(&self) -> u64 {
        (self.clock)()
    }

    /// Put chats under the ones it has: the ones kept from before.
    pub fn take(&mut self, kept: Vec<Chat>) {
        self.chat.extend(kept);
    }

    /// Begin a chat and open it. When the open chat has had nothing said
    /// in it yet, it is the new one, and no other is made.
    pub fn begin(&mut self) {
        let unused = self.current().title().is_none();
        match unused {
            true => {}
            false => {
                self.chat.insert(0, Chat::greeted(&self.greeting));
                self.open = 0;
                // The chat an answer is arriving in is one further down now.
                self.pending.iter_mut().for_each(|(_, chat)| *chat += 1);
            }
        }
    }

    /// Open the chat of that number. `false`, and nothing changes, when
    /// there is none.
    pub fn open(&mut self, index: usize) -> bool {
        let known = index < self.chat.len();
        if known {
            self.open = index;
        }
        known
    }

    /// The open chat.
    pub fn current(&self) -> &Chat {
        &self.chat[self.open]
    }

    /// The number of the open chat.
    pub fn index(&self) -> usize {
        self.open
    }

    /// Every chat, the newest first.
    pub fn list(&self) -> &[Chat] {
        &self.chat
    }

    /// The person says something in the open chat, and the answer begins.
    /// `false`, and nothing is said, when the text is blank or an answer
    /// is still arriving.
    pub fn say(&mut self, text: &str) -> bool {
        let said = text.trim();
        if said.is_empty() || self.pending.is_some() {
            return false;
        }
        let now = self.now();
        let chat = &mut self.chat[self.open];
        // A chat begins when the person first says something in it.
        if chat.when == 0 {
            chat.when = now;
        }
        let answer = self.reply.reply(said, &chat.turn);
        chat.turn.push(Turn::new(Who::Person, said));
        chat.turn.push(Turn::new(Who::Phigen, ""));
        self.pending = Some((answer, self.open));
        // An answer that is there at once is taken at once.
        self.poll();
        true
    }

    /// Take in what has arrived of the answer. `true` while more is to
    /// come. When the last of it has arrived the chats are kept.
    pub fn poll(&mut self) -> bool {
        let Some((source, chat)) = self.pending.as_mut() else { return false };
        let (more, done) = source.more();
        let chat = *chat;
        self.chat[chat].turn.last_mut().into_iter().for_each(|turn| turn.text.push_str(&more));
        if done {
            self.pending = None;
            if let Some((store, name)) = &self.keep {
                store.write(name, &self.text());
            }
        }
        !done
    }

    /// Stop the answer that is arriving: what has arrived of it stays,
    /// and the chats are kept. `false` when none was arriving.
    pub fn stop(&mut self) -> bool {
        let Some((mut source, _)) = self.pending.take() else { return false };
        source.stop();
        if let Some((store, name)) = &self.keep {
            store.write(name, &self.text());
        }
        true
    }

    /// Whether an answer is still arriving.
    pub fn waiting(&self) -> bool {
        self.pending.is_some()
    }

    /// What whoever answers is called; nothing for the stand-in.
    pub fn voice(&self) -> &str {
        self.reply.name()
    }

    /// The names of those that could answer.
    pub fn roster(&mut self) -> Vec<String> {
        self.reply.list()
    }

    /// Whether who could answer is still being found out.
    pub fn asking(&self) -> bool {
        self.reply.asking()
    }

    /// Have the one of that name answer from now on. An answer that is
    /// arriving is finished by the one that began it.
    pub fn choose(&mut self, name: &str) -> bool {
        self.reply.choose(name)
    }
}
