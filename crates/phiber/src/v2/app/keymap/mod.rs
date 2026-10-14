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
    ///
    /// The shape is `WM_keymap_item_raw_to_string`'s
    /// (`wm_keymap.cc:1222`): the held modifiers, in the order **Shift,
    /// Ctrl, Alt, OS**, then **one** key name. So the modifiers are a
    /// prefix and everything after them is the key — which matters because
    /// Blender names some keys in two words. Reading word by word turned
    /// "Page Down" into the Down arrow.
    pub fn parse(label: &str) -> Option<Self> {
        let mut stroke = Self { control: false, alt: false, shift: false, key: String::new() };
        let mut rest = label.trim();
        loop {
            let (word, tail) = rest.split_once(' ').unwrap_or((rest, ""));
            match word {
                "Ctrl" => stroke.control = true,
                "Alt" => stroke.alt = true,
                "Shift" => stroke.shift = true,
                _ => break,
            }
            rest = tail.trim_start();
        }
        stroke.key = Self::key_of(rest)?;
        Some(stroke)
    }

    /// The toolkit's name for the key a label spells.
    ///
    /// Blender's own spelling is what a label should carry, and that is the
    /// event enum's `name` under `compact = false` (`interface.cc:1678`):
    /// `,` and `Page Down`, not `Comma` and `PageDown`. Both are accepted
    /// here, because a parser has no reason to be strict about an input it
    /// can understand — only the labels are held to Blender's spelling
    /// (`tests/menu_truth.rs`).
    fn key_of(name: &str) -> Option<String> {
        // `Numpad 7`, `Numpad .`, `Numpad +`: the key after it, since the
        // toolkit does not tell the numpad from the number row — Blender's
        // own *Emulate Numpad*.
        if let Some(key) = name.strip_prefix("Numpad ") {
            return (!key.is_empty()).then(|| key.to_lowercase());
        }
        Some(match name {
            "" => return None,
            "Page Down" | "PageDown" => "pagedown".into(),
            "Page Up" | "PageUp" => "pageup".into(),
            "," | "Comma" => ",".into(),
            "." | "Period" => ".".into(),
            "Del" | "Delete" => "delete".into(),
            "Esc" | "Escape" => "escape".into(),
            "Bksp" | "Backspace" => "backspace".into(),
            "Enter" | "Return" => "enter".into(),
            key => key.to_lowercase(),
        })
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
        // Blender's own spelling of these two, and the compact spelling
        // the labels used to carry — both have to land on the same key.
        assert_eq!(BKeystroke::parse("Ctrl ,"), Some(stroke(true, false, false, ",")));
        assert_eq!(BKeystroke::parse("Ctrl Comma"), Some(stroke(true, false, false, ",")));
        assert_eq!(BKeystroke::parse("Ctrl Page Down"), Some(stroke(true, false, false, "pagedown")));
        assert_eq!(BKeystroke::parse("Ctrl PageDown"), Some(stroke(true, false, false, "pagedown")));
        assert_eq!(BKeystroke::parse("Shift Page Up"), Some(stroke(false, false, true, "pageup")));
        // A two word key name is one key, not a word and an arrow: reading
        // this label word by word used to bind it to Down.
        assert_ne!(BKeystroke::parse("Ctrl Page Down"), BKeystroke::parse("Ctrl Down"));
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
