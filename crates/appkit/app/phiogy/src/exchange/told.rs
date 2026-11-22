//! Told: who a page is, said by a line in front of this one that knows.
//! Phiora keeps the accounts; when it passes a page's asks on to this
//! line it says in the head who the page belongs to, and carries a word
//! the two share. Whoever carries the word is believed; with no word
//! set nobody is, and every page is a guest by this line's own cookie.

use appkit::service::PTable;

use super::ask::Ask;
use crate::entity::Caller;

/// The heads a line in front says a page's owner in, and the word.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Told {
    key: String,
    user: String,
    name: String,
    handle: String,
    /// The word shared with whoever is believed.
    word: Option<String>,
}

impl Told {
    /// The heads `data/line.tsv` names, believing nobody yet.
    pub fn read(line: &PTable) -> Told {
        let head = |name: &str| line.get(name).to_ascii_lowercase();
        Told { key: head("key"), user: head("user"), name: head("name"), handle: head("handle"), word: None }
    }

    /// The same heads, believing whoever carries that word.
    pub fn believing(mut self, word: Option<&str>) -> Told {
        self.word = word.map(str::trim).filter(|word| !word.is_empty()).map(str::to_string);
        self
    }

    /// Who an ask says its page belongs to: nobody when it does not
    /// carry the word, or names no one.
    pub fn caller(&self, ask: &Ask) -> Option<Caller> {
        let word = self.word.as_deref().filter(|word| ask.head(&self.key) == *word)?;
        let said = |name: &str| plain(ask.head(name));
        let user = Some(said(&self.user)).filter(|user| !user.is_empty() && !word.is_empty())?;
        let handle = Some(said(&self.handle)).filter(|handle| !handle.is_empty());
        Some(Caller::new(&user, &said(&self.name), handle.as_deref()))
    }
}

/// A value as it was before its bytes were written as %XX.
fn plain(value: &str) -> String {
    let raw = value.as_bytes();
    let (mut out, mut at) = (Vec::with_capacity(raw.len()), 0);
    while at < raw.len() {
        let pair = raw.get(at + 1..at + 3).and_then(|pair| std::str::from_utf8(pair).ok()).and_then(|pair| u8::from_str_radix(pair, 16).ok());
        match (raw[at], pair) {
            (b'%', Some(byte)) => {
                out.push(byte);
                at += 3;
            }
            (byte, _) => {
                out.push(byte);
                at += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).to_string()
}
