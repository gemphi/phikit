//! The keymap — Blender's `wm.keyconfigs`, read off the menus.
//!
//! Blender shows a menu item's shortcut beside it because the shortcut *is*
//! the item's keymap entry; here the same is true the other way round: every
//! menu item that carries a shortcut label is a binding (`set.rs::bindings`),
//! so the menus and the keys can never say different things. The keys
//! Blender binds outside its menus — Space to play, the arrows to step,
//! Escape to stop — are the extras in `set.rs`.
//!
//! A key reaches the keymap only when no field holds the caret
//! (`handle_key`), as Blender's text buttons swallow typing. The numpad is
//! not told apart from the number row by the toolkit, so `Numpad 7` is `7`
//! — Blender's own *Emulate Numpad*.

pub mod set;

use crate::v2::state::BState;
use crate::v2::view::prim::BAction;

/// A key with its modifiers, as a menu label spells it and as a keystroke
/// arrives: `Shift Ctrl S` ↔ shift + control + `s`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BKeystroke {
    pub control: bool,
    pub alt: bool,
    pub shift: bool,
    /// The toolkit's key name, lower-case: `s`, `f12`, `pagedown`, `space`, `.`.
    pub key: String,
}

impl BKeystroke {
    /// A menu's shortcut label, or `None` when the label is not one.
    pub fn parse(label: &str) -> Option<Self> {
        let mut stroke = Self { control: false, alt: false, shift: false, key: String::new() };
        let mut words = label.split_whitespace().peekable();
        while let Some(word) = words.next() {
            match word {
                "Ctrl" => stroke.control = true,
                "Alt" => stroke.alt = true,
                "Shift" => stroke.shift = true,
                // `Numpad 7`, `Numpad .`, `Numpad +`: the key after it.
                "Numpad" => stroke.key = words.next()?.to_lowercase(),
                "Comma" => stroke.key = ",".into(),
                "Period" => stroke.key = ".".into(),
                "Space" => stroke.key = "space".into(),
                "Del" | "Delete" => stroke.key = "delete".into(),
                "PageDown" => stroke.key = "pagedown".into(),
                "PageUp" => stroke.key = "pageup".into(),
                "Left" | "Right" | "Up" | "Down" | "Home" | "End" | "Escape" => stroke.key = word.to_lowercase(),
                key => stroke.key = key.to_lowercase(),
            }
        }
        (!stroke.key.is_empty()).then_some(stroke)
    }

    /// The keystroke the toolkit delivered. A symbol typed with Shift (`+`
    /// over `=`) is the symbol, without the Shift.
    pub fn of(key: &str, key_char: Option<&str>, control: bool, alt: bool, shift: bool) -> Self {
        let symbol = key_char.filter(|c| c.chars().count() == 1 && !c.chars().next().is_some_and(char::is_alphanumeric));
        match symbol {
            Some(symbol) if symbol.trim() != "" => Self { control, alt, shift: false, key: symbol.to_string() },
            _ => Self { control, alt, shift, key: key.to_lowercase() },
        }
    }
}

/// The action a keystroke runs, and the name it runs under (the menu path
/// or the operator), read with the state in hand — Space plays or pauses.
pub fn action_for(stroke: &BKeystroke, state: &BState) -> Option<(BAction, String)> {
    // A transform under way takes its own keys first (`transform_modal_keymap`).
    if !stroke.control && !stroke.alt {
        if let Some(hit) = crate::v2::scene::xform::BTransformModal::key(&stroke.key, stroke.shift, state) {
            return Some(hit);
        }
    }
    let over = crate::v2::view::draw::host::BPointer::host();
    set::bindings(over).into_iter().find(|binding| binding.stroke == *stroke).map(|binding| ((binding.action)(state), binding.name))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stroke(control: bool, alt: bool, shift: bool, key: &str) -> BKeystroke {
        BKeystroke { control, alt, shift, key: key.into() }
    }

    #[test]
    fn menu_labels_parse_the_way_blender_writes_them() {
        assert_eq!(BKeystroke::parse("Shift Ctrl S"), Some(stroke(true, false, true, "s")));
        assert_eq!(BKeystroke::parse("Numpad 7"), Some(stroke(false, false, false, "7")));
        assert_eq!(BKeystroke::parse("Ctrl Numpad 1"), Some(stroke(true, false, false, "1")));
        assert_eq!(BKeystroke::parse("Numpad ."), Some(stroke(false, false, false, ".")));
        assert_eq!(BKeystroke::parse("Ctrl Comma"), Some(stroke(true, false, false, ",")));
        assert_eq!(BKeystroke::parse("Ctrl PageDown"), Some(stroke(true, false, false, "pagedown")));
        assert_eq!(BKeystroke::parse("F12"), Some(stroke(false, false, false, "f12")));
        assert_eq!(BKeystroke::parse("Alt A"), Some(stroke(false, true, false, "a")));
        assert_eq!(BKeystroke::parse("Ctrl Space"), Some(stroke(true, false, false, "space")));
        assert_eq!(BKeystroke::parse(""), None);
    }

    #[test]
    fn a_shifted_symbol_arrives_as_the_symbol() {
        assert_eq!(BKeystroke::of("=", Some("+"), false, false, true), stroke(false, false, false, "+"));
        assert_eq!(BKeystroke::of("s", Some("S"), true, false, true), stroke(true, false, true, "s"));
        assert_eq!(BKeystroke::of("F12", None, false, false, false), stroke(false, false, false, "f12"));
    }
}
