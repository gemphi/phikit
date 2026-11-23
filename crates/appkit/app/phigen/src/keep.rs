//! Keeping the chats: the history as a table, a turn a line, so that it
//! can be written to a file and read again the next time Phigen opens.
//!
//! A line is the chat's number, who spoke and what they said, between
//! tabs; a line that says `when` gives the moment the chat began. A line
//! break or a tab in what was said is written `\n` or `\t`, and a
//! backslash `\\`. A chat nothing was said in is not kept.

use crate::chat::{Chat, Turn, Who};
use crate::history::History;
use crate::reply::Reply;

/// How each speaker is written.
const PERSON: &str = "person";
const PHIGEN: &str = "phigen";
/// How the moment a chat began is written.
const WHEN: &str = "when";

/// What was said, on one line.
fn flat(text: &str) -> String {
    text.replace('\\', "\\\\").replace('\n', "\\n").replace('\t', "\\t")
}

/// One line, as it was said.
fn full(text: &str) -> String {
    let mut out = String::new();
    let mut letter = text.chars();
    while let Some(now) = letter.next() {
        let next = match now {
            '\\' => match letter.next() {
                Some('n') => '\n',
                Some('t') => '\t',
                Some(other) => other,
                None => '\\',
            },
            other => other,
        };
        out.push(next);
    }
    out
}

impl History {
    /// Every chat something was said in, a turn a line, the newest chat
    /// first.
    pub fn text(&self) -> String {
        let used = self.list().iter().filter(|chat| chat.title().is_some());
        let line = |(number, chat): (usize, &Chat)| {
            let turn = |turn: &Turn| {
                let who = match turn.who {
                    Who::Person => PERSON,
                    Who::Phigen => PHIGEN,
                };
                format!("{number}\t{who}\t{}\n", flat(&turn.text))
            };
            format!("{number}\t{WHEN}\t{}\n", chat.when) + &chat.turn.iter().map(turn).collect::<String>()
        };
        used.enumerate().map(line).collect()
    }

    /// The history a table holds, with a new chat begun over the kept
    /// ones. A line that cannot be read is skipped.
    pub fn read(text: &str, reply: Box<dyn Reply>, greeting: &str) -> History {
        let mut history = History::new(reply, greeting);
        let mut kept: Vec<Chat> = Vec::new();
        let mut last = None;
        for line in text.lines() {
            let mut cell = line.splitn(3, '\t');
            let (number, who, said) = (cell.next().unwrap_or_default(), cell.next().unwrap_or_default(), cell.next().unwrap_or_default());
            let who = match who {
                PERSON => Some(Who::Person),
                PHIGEN => Some(Who::Phigen),
                WHEN => None,
                _ => continue,
            };
            if last != Some(number) {
                kept.push(Chat::default());
                last = Some(number);
            }
            let Some(chat) = kept.iter_mut().last() else { continue };
            match who {
                Some(who) => chat.turn.push(Turn::new(who, &full(said))),
                None => chat.when = said.trim().parse().unwrap_or(0),
            }
        }
        history.take(kept);
        history
    }
}
