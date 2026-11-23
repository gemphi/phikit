//! What a key press types. The screen is told a key's name, the text it
//! would write, and whether the paste chord was held; this says which
//! characters go to the stage. It names nothing of gpui's, so it is
//! tested like the rest of appkit.

use crate::pui::core::PEvent;

/// The keys that write nothing, by the names gpui gives them, and the
/// characters pui knows them by.
const NAMED: [(&str, char); 10] = [
    ("backspace", PEvent::BACKSPACE),
    ("enter", PEvent::ENTER),
    ("escape", PEvent::ESCAPE),
    ("up", PEvent::UP),
    ("down", PEvent::DOWN),
    ("left", PEvent::LEFT),
    ("right", PEvent::RIGHT),
    ("delete", PEvent::DELETE),
    ("home", PEvent::HOME),
    ("end", PEvent::END),
];

/// The characters one key press sends. `name` is the key's name (`a`,
/// `enter`), `text` what it would write, and `chord` whether the key that
/// makes a shortcut was held (Control, or Command on a Mac). With the
/// chord, `v` types what `clip` gives, the clipboard's text, on one line.
pub fn typed(name: &str, text: Option<&str>, chord: bool, clip: impl FnOnce() -> Option<String>) -> Vec<char> {
    let flat = |pasted: String| -> Vec<char> { pasted.chars().map(|letter| if letter.is_control() { ' ' } else { letter }).collect() };
    let named = NAMED.iter().find(|(known, _)| *known == name).map(|(_, key)| *key);
    match (named, name, chord, text) {
        (Some(key), _, _, _) => vec![key],
        (None, "v", true, _) => clip().map(flat).unwrap_or_default(),
        (None, _, false, Some(text)) => text.chars().collect(),
        _ => Vec::new(),
    }
}
