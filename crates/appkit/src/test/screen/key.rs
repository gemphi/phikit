use crate::pui::core::PEvent;
use crate::screen::key::typed;

fn none() -> Option<String> {
    None
}

#[test]
fn a_key_types_the_text_it_writes_and_the_keys_that_write_nothing_have_their_own_characters() {
    assert_eq!(typed("a", Some("a"), false, none), ['a']);
    assert_eq!(typed("space", Some(" "), false, none), [' ']);
    let silent = [("backspace", PEvent::BACKSPACE), ("enter", PEvent::ENTER), ("escape", PEvent::ESCAPE), ("left", PEvent::LEFT), ("right", PEvent::RIGHT), ("up", PEvent::UP), ("down", PEvent::DOWN), ("delete", PEvent::DELETE), ("home", PEvent::HOME), ("end", PEvent::END)];
    for (name, key) in silent {
        assert_eq!(typed(name, None, false, none), [key], "{name}");
    }
    // Enter is Enter whatever text the system says it writes.
    assert_eq!(typed("enter", Some("\n"), false, none), [PEvent::ENTER]);
    // The arrows and their like are of the range kept for such keys.
    assert!(PEvent::silent(PEvent::LEFT) && PEvent::silent(PEvent::END) && !PEvent::silent('a') && !PEvent::silent(PEvent::ENTER));
    // A key that writes nothing types nothing, and nor does a shortcut.
    assert!(typed("f5", None, false, none).is_empty() && typed("a", Some("a"), true, none).is_empty());
}

#[test]
fn the_paste_chord_types_the_clipboard_s_text_on_one_line() {
    let clip = || Some("two\nlines\there".to_string());
    assert_eq!(typed("v", None, true, clip).into_iter().collect::<String>(), "two lines here");
    // An empty clipboard pastes nothing; without the chord `v` is a letter.
    assert!(typed("v", None, true, none).is_empty());
    assert_eq!(typed("v", Some("v"), false, clip), ['v']);
}
