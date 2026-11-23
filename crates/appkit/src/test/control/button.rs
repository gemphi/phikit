use std::cell::RefCell;
use std::rc::Rc;

use crate::pui::control::*;
use crate::pui::core::*;
use crate::test::core::Board;
use crate::theme::{PLook, PTheme};

fn button() -> PButton {
    PButton::new("Render", PRect::new(10.0, 10.0, 80.0, 24.0))
}

#[test]
fn every_press_counts_and_sends_the_count() {
    let sent = Rc::new(RefCell::new(Vec::new()));
    let told = Rc::clone(&sent);
    let mut button = button();
    button.set_action(Box::new(move |count| told.borrow_mut().push(*count)));
    let press = PEvent::PointerDown(PPoint::new(20.0, 20.0));
    assert!(send(&mut button, &press) && send(&mut button, &press));
    assert_eq!((button.value(), sent.borrow().as_slice()), (2, [1, 2].as_slice()));
}

#[test]
fn a_press_beside_it_or_on_a_disabled_one_is_nothing() {
    let mut button = button();
    assert!(!send(&mut button, &PEvent::PointerDown(PPoint::new(200.0, 20.0))));
    button.set_enabled(false);
    assert!(!send(&mut button, &PEvent::PointerDown(PPoint::new(20.0, 20.0))));
    // Letting go and moving are not presses.
    button.set_enabled(true);
    assert!(!send(&mut button, &PEvent::PointerUp(PPoint::new(20.0, 20.0))));
    assert_eq!(button.value(), 0);
}

#[test]
fn it_is_drawn_in_the_theme_s_colours_and_a_label_nothing_measures_is_inset() {
    let mut board = Board::default();
    show(&button(), &mut board, PPoint::default());
    assert_eq!(board.fill, [(PRect::new(10.0, 10.0, 80.0, 24.0), board.color(FILL))]);
    assert_eq!(board.text, [(PPoint::new(14.0, 14.0), "Render".to_string())]);
    assert!(board.color(FILL).a > 0.0, "the test theme has the token");
}

#[test]
fn its_label_is_in_its_middle_across_and_down() {
    let look = PLook::read("size\tcontrol.inset\t4\nsize\ttext.line\t20\nsize\tcontrol.radius\t6\nsize\ttext.advance\t8\n");
    let mut sheet = PSheet::new(&look);
    show(&PButton::new("Render", PRect::new(10.0, 10.0, 80.0, 40.0)), &mut sheet, PPoint::default());
    // A line of 20 in a button of 40 starts 10 down, and six letters of 8
    // in a button of 80 start 16 in; the fill is rounded.
    let fill = PMark::Fill { rect: PRect::new(10.0, 10.0, 80.0, 40.0), radius: 6.0, color: PColor::default() };
    let label = PMark::Text { at: PPoint::new(26.0, 20.0), text: "Render".to_string(), color: PColor::default() };
    assert_eq!(sheet.mark, [fill, label]);
}

#[test]
fn in_a_theme_with_light_a_button_wears_the_label_at_its_strongest_and_the_theme_s_ink() {
    let look = PLook::read("color\tshade.light\t#ffffff\nsize\tshade.share\t0.5\nsize\tshade.rest\t0.5\ncolor\ttext.ink\t#102030\ncolor\tbutton.ink\t#ff0000\nsize\tcontrol.radius\t6\n");
    let mut sheet = PSheet::new(&look);
    show(&PButton::new("Go", PRect::new(0.0, 0.0, 80.0, 40.0)), &mut sheet, PPoint::default());
    let lit = PColor::new(1.0, 1.0, 1.0, 0.5);
    assert_eq!(sheet.mark.first(), Some(&PMark::Blend { rect: PRect::new(0.0, 0.0, 80.0, 40.0), radius: 6.0, from: lit, to: PColor { a: 0.0, ..lit }, turn: 90.0 }));
    // Light has no colour to set an ink of its own against.
    assert!(matches!(sheet.mark.last(), Some(PMark::Text { color, .. }) if *color == look.color("text.ink").expect("in the look")));
}
