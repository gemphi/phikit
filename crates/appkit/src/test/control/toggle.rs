use std::cell::RefCell;
use std::rc::Rc;

use crate::pui::control::*;
use crate::pui::core::*;
use crate::test::core::Board;

#[test]
fn a_press_flips_it_and_sends_what_it_became() {
    let sent = Rc::new(RefCell::new(Vec::new()));
    let told = Rc::clone(&sent);
    let mut toggle = PToggle::new(PRect::new(0.0, 0.0, 40.0, 20.0));
    toggle.set_action(Box::new(move |on| told.borrow_mut().push(*on)));
    let press = PEvent::PointerDown(PPoint::new(5.0, 5.0));
    assert!(send(&mut toggle, &press) && toggle.value());
    assert!(send(&mut toggle, &press) && !toggle.value());
    assert_eq!(*sent.borrow(), [true, false]);
}

#[test]
fn a_disabled_toggle_stays_as_it_is() {
    let mut toggle = PToggle::new(PRect::new(0.0, 0.0, 40.0, 20.0));
    toggle.set_enabled(false);
    assert!(!send(&mut toggle, &PEvent::PointerDown(PPoint::new(5.0, 5.0))) && !toggle.value());
}

#[test]
fn on_is_drawn_in_the_accent_and_off_in_the_fill() {
    let mut toggle = PToggle::new(PRect::new(0.0, 0.0, 40.0, 20.0));
    let mut board = Board::default();
    show(&toggle, &mut board, PPoint::default());
    toggle.store(true);
    show(&toggle, &mut board, PPoint::default());
    let color: Vec<PColor> = board.fill.iter().map(|(_, color)| *color).collect();
    assert_eq!(color, [board.color(FILL), board.color(ACCENT)]);
    assert_ne!(board.color(FILL), board.color(ACCENT));
}
