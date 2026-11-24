use std::cell::RefCell;
use std::rc::Rc;

use crate::pui::control::*;
use crate::pui::core::*;
use crate::test::core::Board;
use crate::theme::PLook;

fn field() -> PField {
    PField::new(PRect::new(0.0, 0.0, 120.0, 24.0))
}

/// Type these keys into a field that has been pressed.
fn typed(key: &str) -> PField {
    let mut field = field();
    assert!(send(&mut field, &PEvent::PointerDown(PPoint::new(5.0, 5.0))));
    key.chars().for_each(|key| {
        send(&mut field, &PEvent::KeyDown(key));
    });
    field
}

#[test]
fn a_press_gives_it_the_keys_and_each_key_adds_its_character() {
    let field = typed("Cube");
    assert!(field.focused);
    assert_eq!(field.value(), "Cube");
}

#[test]
fn a_field_that_was_not_pressed_takes_no_keys() {
    let mut field = field();
    assert!(!send(&mut field, &PEvent::KeyDown('a')));
    assert_eq!(field.value(), "");
}

#[test]
fn backspace_takes_the_last_character_away_and_other_control_keys_do_nothing() {
    let field = typed("Cube\u{8}\u{8}be\u{1b}\t");
    assert_eq!(field.value(), "Cube");
    // On an empty field there is nothing to take, and nothing is sent.
    let mut empty = typed("");
    assert!(!send(&mut empty, &PEvent::KeyDown('\u{8}')));
}

#[test]
fn every_change_is_sent_with_the_whole_text() {
    let sent = Rc::new(RefCell::new(Vec::new()));
    let told = Rc::clone(&sent);
    let mut field = field();
    field.set_action(Box::new(move |text: &String| told.borrow_mut().push(text.clone())));
    send(&mut field, &PEvent::PointerDown(PPoint::new(5.0, 5.0)));
    "ab\u{8}".chars().for_each(|key| {
        send(&mut field, &PEvent::KeyDown(key));
    });
    assert_eq!(*sent.borrow(), ["a", "ab", "a"]);
    // A text the app stores is shown and not sent.
    field.store("Lamp".to_string());
    assert_eq!((field.value().as_str(), sent.borrow().len()), ("Lamp", 3));
}

#[test]
fn a_disabled_field_is_not_given_the_keys() {
    let mut field = field();
    field.set_enabled(false);
    assert!(!send(&mut field, &PEvent::PointerDown(PPoint::new(5.0, 5.0))));
    assert!(!field.focused && !send(&mut field, &PEvent::KeyDown('a')));
}

#[test]
fn the_field_that_takes_the_keys_is_underlined() {
    let mut board = Board::default();
    show(&field(), &mut board, PPoint::default());
    assert_eq!(board.fill.len(), 1);
    let mut board = Board::default();
    show(&typed("Hi"), &mut board, PPoint::default());
    // The test theme's line is 2 high, at the bottom of the field.
    assert_eq!(board.fill[1], (PRect::new(0.0, 22.0, 120.0, 2.0), board.color(ACCENT)));
    assert_eq!(board.text, [(PPoint::new(4.0, 4.0), "Hi".to_string())]);
}

#[test]
fn the_field_that_takes_the_keys_has_a_caret_after_its_text() {
    // A letter is 8 wide and a line 20 high; the text starts 4 in.
    let look = PLook::read("size\tcontrol.inset\t4\nsize\tcontrol.line\t2\nsize\ttext.advance\t8\nsize\ttext.line\t20\n");
    let caret = |field: &PField| {
        let mut sheet = PSheet::new(&look);
        show(field, &mut sheet, PPoint::default());
        match sheet.mark.last() {
            Some(PMark::Fill { rect, .. }) => *rect,
            other => panic!("the caret is the last thing drawn, not {other:?}"),
        }
    };
    assert_eq!(caret(&typed("")), PRect::new(2.0, 2.0, 2.0, 20.0));
    assert_eq!(caret(&typed("Hi")), PRect::new(18.0, 2.0, 2.0, 20.0));
    // A field that does not take the keys has none: its fill and its text.
    let mut sheet = PSheet::new(&look);
    show(&field(), &mut sheet, PPoint::default());
    assert_eq!(sheet.mark.len(), 2);
}

#[test]
fn the_arrows_home_and_end_move_the_caret_and_a_key_writes_where_it_stands() {
    let mut field = typed("helo");
    assert_eq!(field.caret(), 4);
    assert!(send(&mut field, &PEvent::KeyDown(PEvent::LEFT)));
    assert!(send(&mut field, &PEvent::KeyDown('l')));
    assert_eq!((field.value().as_str(), field.caret()), ("hello", 4));
    assert!(send(&mut field, &PEvent::KeyDown(PEvent::HOME)) && field.caret() == 0);
    assert!(send(&mut field, &PEvent::KeyDown('>')));
    assert!(send(&mut field, &PEvent::KeyDown(PEvent::END)) && field.caret() == 6);
    assert_eq!(field.value(), ">hello");
    // At either end there is nowhere further to go.
    assert!(!send(&mut field, &PEvent::KeyDown(PEvent::RIGHT)));
    send(&mut field, &PEvent::KeyDown(PEvent::HOME));
    assert!(!send(&mut field, &PEvent::KeyDown(PEvent::LEFT)) && field.caret() == 0);
}

#[test]
fn backspace_takes_the_character_before_the_caret_and_delete_the_one_after() {
    let mut field = typed("ab\u{e9}cd");
    [PEvent::LEFT, PEvent::LEFT].into_iter().for_each(|key| _ = send(&mut field, &PEvent::KeyDown(key)));
    // The caret stands between the accented letter and the c.
    assert!(send(&mut field, &PEvent::KeyDown(PEvent::BACKSPACE)));
    assert_eq!((field.value().as_str(), field.caret()), ("abcd", 2));
    assert!(send(&mut field, &PEvent::KeyDown(PEvent::DELETE)));
    assert_eq!((field.value().as_str(), field.caret()), ("abd", 2));
    // Nothing is before the first character or after the last, and a
    // key that writes nothing writes nothing.
    send(&mut field, &PEvent::KeyDown(PEvent::END));
    assert!(!send(&mut field, &PEvent::KeyDown(PEvent::DELETE)));
    assert!(!send(&mut field, &PEvent::KeyDown(PEvent::UP)) && !send(&mut field, &PEvent::KeyDown('\u{F704}')));
    assert_eq!(field.value(), "abd");
}

#[test]
fn a_text_that_is_stored_has_the_caret_after_it() {
    let mut field = typed("ab");
    send(&mut field, &PEvent::KeyDown(PEvent::HOME));
    field.store("Lamp".to_string());
    assert_eq!(field.caret(), 4);
    field.store(String::new());
    assert_eq!(field.caret(), 0);
}

#[test]
fn in_a_theme_with_light_a_field_wears_the_surface_and_a_bare_one_draws_no_ground() {
    let look = PLook::read("color\tshade.light\t#ffffff\nsize\tshade.share\t0.5\nsize\tcontrol.radius\t6\n");
    let mut field = PField::new(PRect::new(0.0, 0.0, 100.0, 30.0));
    let ground = |field: &PField| {
        let mut sheet = PSheet::new(&look);
        show(field, &mut sheet, PPoint::default());
        sheet.mark.into_iter().filter(|mark| matches!(mark, PMark::Blend { .. } | PMark::Fill { .. })).collect::<Vec<_>>()
    };
    let lit = PColor::new(1.0, 1.0, 1.0, 0.5);
    assert_eq!(ground(&field), [PMark::Blend { rect: PRect::new(0.0, 0.0, 100.0, 30.0), radius: 6.0, from: lit, to: PColor { a: 0.0, ..lit }, turn: 0.0, stop: [0.0, 1.0] }]);
    // One that stands in a box something else drew adds no light to it.
    field.bare = true;
    assert!(ground(&field).is_empty());
}

#[test]
fn a_secret_field_draws_a_mark_for_each_character_and_keeps_what_was_typed() {
    let look = PLook::read("word\tcontrol.mask\t·\nsize\ttext.advance\t8\n");
    let mut field = PField::new(PRect::new(0.0, 0.0, 120.0, 24.0));
    (field.secret, field.focused) = (true, true);
    "open".chars().for_each(|key| _ = field.respond(&PEvent::KeyDown(key)));
    let mut sheet = PSheet::new(&look);
    field.draw(&mut sheet, PPoint::default());
    // What is typed is the field's value, and none of it is drawn.
    assert_eq!((field.value(), sheet.text_list()), ("open".to_string(), vec!["····"]));
    field.secret = false;
    let mut sheet = PSheet::new(&look);
    field.draw(&mut sheet, PPoint::default());
    assert_eq!(sheet.text_list(), ["open"]);
}
