//! Typing into a field: which field has the caret, and what each key does.
//!
//! Blender ref: `ui_textedit_*` in `interface_handlers.cc` — one text button
//! is active at a time, printable keys insert, Backspace deletes, Return
//! confirms and Escape cancels back to the value the field started with.

use crate::v2::state::{BKey as _, AppKey};
use crate::v2::state::FileKey;
use super::path::BPath;
use super::BState;

/// What a key press did to the edited field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BEdit {
    /// The field changed; re-render.
    Typed,
    /// Return: the field keeps its value and the caret leaves.
    Confirmed,
    /// Escape: the value before editing is restored.
    Cancelled,
    /// Nothing here handles this key.
    Ignored,
}

/// The encoded key of the field being typed into, if any.
/// Taking the caret: remember what the field held so Escape can put it
/// back. Runs before a `Set` of `edit` is stored.
pub fn before_set(s: &mut BState, path: &BPath, value: &str) {
    if *path == AppKey::Edit.path() && !value.is_empty() {
        let before = s.get_encoded(value).unwrap_or("").to_string();
        s.set(AppKey::EditUndo, before);
    }
}

pub fn editing(s: &BState) -> Option<String> {
    s.get(AppKey::Edit).filter(|e| !e.is_empty()).map(str::to_string)
}

/// Is `path` the field with the caret?
pub fn is_editing(s: &BState, path: impl crate::v2::state::BKey) -> bool {
    let path = path.path();
    let path = &path;
    s.get(AppKey::Edit) == Some(path.encode().as_str())
}

/// Put the caret in `path`, remembering its value so Escape can restore it.
pub fn start(s: &mut BState, path: impl crate::v2::state::BKey) {
    let path = path.path();
    let path = &path;
    let before = s.get(path).unwrap_or("").to_string();
    s.set(AppKey::Edit, path.encode());
    s.set(AppKey::EditUndo, before);
}

/// Take the caret out of whatever field has it.
pub fn stop(s: &mut BState) {
    s.set(AppKey::Edit, "");
    s.set(AppKey::EditUndo, "");
}

/// The path behind an edited field's encoded key: Return on a field
/// re-applies its value as a normal `Set` so everything watching the key
/// still fires.
///
/// The named four are matched first so they keep their exact `'static`
/// segments; anything else — a property row, a number button — is rebuilt
/// from the encoded string. It used to be the four and nothing else, which
/// meant Return on a property field confirmed the caret away without ever
/// dispatching the write, so typing a value into Max Samples could not take
/// effect even once the keys reached it.
pub fn field_path(encoded: &str) -> Option<BPath> {
    let known = [
        FileKey::Path.path(),
        FileKey::Search.path(),
        crate::v2::state::AppKey::CommandQuery.path(),
        crate::v2::state::AppKey::RenameText.path(),
    ]
    .into_iter()
    .find(|p| p.encode() == encoded);
    known.or_else(|| rebuilt(encoded))
}

/// `ctx.a.b.c` back into a `BPath`. Segments are interned (`path::intern`)
/// because a `BPath` holds `&'static str` and these names are only known at
/// runtime; the pool leaks each distinct name at most once.
fn rebuilt(encoded: &str) -> Option<BPath> {
    let rest = encoded.strip_prefix("ctx.")?;
    if rest.is_empty() {
        return None;
    }
    Some(BPath::owned(rest.split('.').map(super::path::intern).collect()))
}

/// Apply one key to the edited field. `key` is GPUI's key name ("a", "space",
/// "backspace", "enter", "escape"); `text` is what it would type, when
/// printable.
pub fn key(s: &mut BState, key: &str, text: Option<&str>) -> BEdit {
    let Some(path) = editing(s) else {
        return BEdit::Ignored;
    };
    let current = s.get_encoded(&path).unwrap_or("").to_string();
    match key {
        "escape" => {
            let before = s.get(AppKey::EditUndo).unwrap_or("").to_string();
            s.set_encoded(&path, before);
            stop(s);
            BEdit::Cancelled
        }
        "enter" | "return" => {
            stop(s);
            BEdit::Confirmed
        }
        "backspace" => {
            let mut next = current;
            next.pop();
            s.set_encoded(&path, next);
            BEdit::Typed
        }
        "delete" => {
            s.set_encoded(&path, String::new());
            BEdit::Typed
        }
        _ => match text.filter(|t| !t.is_empty() && !t.chars().any(char::is_control)) {
            Some(t) => {
                s.set_encoded(&path, format!("{current}{t}"));
                BEdit::Typed
            }
            None => BEdit::Ignored,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
        use crate::v2::state::BState;

    fn typing(s: &mut BState, text: &str) {
        for ch in text.chars() {
            assert_eq!(key(s, &ch.to_string(), Some(&ch.to_string())), BEdit::Typed);
        }
    }

    #[test]
    fn a_property_row_is_a_field_too() {
        // The four named fields keep resolving.
        let search = FileKey::Search.path();
        assert_eq!(field_path(&search.encode()), Some(search));
        // And so does a key nothing named ahead of time: a property row's
        // own path, which is what Max Samples is.
        let samples = BPath::of(&["buts", "cycles", "samples"]);
        assert_eq!(field_path(&samples.encode()), Some(samples));
        // Not a field key at all.
        assert_eq!(field_path("ctx"), None);
        assert_eq!(field_path(""), None);
    }

    #[test]
    fn types_backspaces_confirms_and_cancels() {
        let mut s = BState::default();
        let search = FileKey::Search.path();
        // Nothing is being edited: keys are not ours.
        assert_eq!(key(&mut s, "a", Some("a")), BEdit::Ignored);
        assert!(editing(&s).is_none());

        start(&mut s, &search);
        assert!(is_editing(&s, &search));
        typing(&mut s, "cube");
        assert_eq!(s.get(&search), Some("cube"));
        assert_eq!(key(&mut s, "backspace", None), BEdit::Typed);
        assert_eq!(s.get(&search), Some("cub"));

        // Return keeps the value and drops the caret.
        assert_eq!(key(&mut s, "enter", None), BEdit::Confirmed);
        assert_eq!(s.get(&search), Some("cub"));
        assert!(editing(&s).is_none());

        // Escape restores what the field held when typing started.
        start(&mut s, &search);
        typing(&mut s, "xyz");
        assert_eq!(s.get(&search), Some("cubxyz"));
        assert_eq!(key(&mut s, "escape", None), BEdit::Cancelled);
        assert_eq!(s.get(&search), Some("cub"));
        assert!(editing(&s).is_none());
    }

    #[test]
    fn control_keys_never_reach_the_field() {
        let mut s = BState::default();
        let search = FileKey::Search.path();
        start(&mut s, &search);
        for (k, text) in [("f5", None), ("shift", None), ("tab", Some("\t"))] {
            assert_eq!(key(&mut s, k, text), BEdit::Ignored, "{k}");
        }
        assert!(s.get(&search).unwrap_or("").is_empty(), "nothing was typed");
    }

    #[test]
    fn typed_fields_are_reachable_by_their_key() {
        for p in [FileKey::Search.path(), FileKey::Path.path(), crate::v2::state::AppKey::CommandQuery.path()] {
            assert_eq!(field_path(&p.encode()), Some(p));
        }
        // Any other `ctx.` key is rebuilt, so Return on a property field
        // writes; what is not a key at all is still nothing.
        assert_eq!(field_path("ctx.not.a.field").map(|p| p.encode()), Some("ctx.not.a.field".to_string()));
        assert_eq!(field_path("not a key"), None);
    }
}
