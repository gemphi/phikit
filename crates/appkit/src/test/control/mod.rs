//! Tests of `control`, one file for each file there. This file tests the
//! trait, through a switch made for the purpose.

mod button;
mod field;
mod toggle;

use std::cell::RefCell;
use std::rc::Rc;

use crate::pui::control::*;
use crate::pui::core::*;

/// A control for the tests: it is on or off, and a press flips it.
struct Switch {
    frame: PRect,
    on: bool,
    enabled: bool,
    action: Option<PAction<bool>>,
}

impl Switch {
    fn new() -> Switch {
        Switch { frame: PRect::new(0.0, 0.0, 40.0, 20.0), on: false, enabled: true, action: None }
    }

    /// A switch that writes every value it sends into `sent`.
    fn telling(sent: &Rc<RefCell<Vec<bool>>>) -> Switch {
        let sent = Rc::clone(sent);
        Switch { action: Some(Box::new(move |value| sent.borrow_mut().push(*value))), ..Switch::new() }
    }
}

impl PDraw for Switch {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        paint.fill(PRect { origin, size: self.frame.size }, PColor::default());
    }
}

impl PView for Switch {
    fn frame(&self) -> PRect {
        self.frame
    }

    fn set_frame(&mut self, frame: PRect) {
        self.frame = frame;
    }

    fn respond(&mut self, event: &PEvent) -> bool {
        match event {
            PEvent::PointerDown(_) => self.change(!self.on),
            _ => false,
        }
    }
}

impl PControl for Switch {
    type Value = bool;

    fn value(&self) -> bool {
        self.on
    }

    fn store(&mut self, value: bool) {
        self.on = value;
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn action(&mut self) -> Option<&mut PAction<bool>> {
        self.action.as_mut()
    }

    fn set_action(&mut self, action: PAction<bool>) {
        self.action = Some(action);
    }
}

#[test]
fn a_hand_s_change_is_kept_and_sent() {
    let sent = Rc::new(RefCell::new(Vec::new()));
    let mut switch = Switch::telling(&sent);
    assert!(switch.change(true));
    assert!(switch.value());
    assert!(switch.change(false));
    assert_eq!(*sent.borrow(), [true, false]);
}

#[test]
fn a_value_it_already_holds_is_no_change() {
    let sent = Rc::new(RefCell::new(Vec::new()));
    let mut switch = Switch::telling(&sent);
    assert!(!switch.change(false));
    assert!(sent.borrow().is_empty());
}

#[test]
fn a_disabled_control_keeps_its_value_and_sends_nothing() {
    let sent = Rc::new(RefCell::new(Vec::new()));
    let mut switch = Switch { enabled: false, ..Switch::telling(&sent) };
    assert!(!switch.change(true));
    assert!(!switch.value() && sent.borrow().is_empty());
}

#[test]
fn a_value_the_app_stores_is_kept_and_not_sent() {
    let sent = Rc::new(RefCell::new(Vec::new()));
    let mut switch = Switch::telling(&sent);
    switch.store(true);
    assert!(switch.value() && sent.borrow().is_empty());
}

#[test]
fn a_control_with_no_action_still_changes() {
    let mut switch = Switch::new();
    assert!(switch.change(true) && switch.value());
}

#[test]
fn an_action_and_whether_it_is_enabled_can_be_set_later() {
    let sent = Rc::new(RefCell::new(Vec::new()));
    let mut switch = Switch::new();
    let told = Rc::clone(&sent);
    switch.set_action(Box::new(move |value| told.borrow_mut().push(*value)));
    switch.set_enabled(false);
    assert!(!switch.change(true));
    switch.set_enabled(true);
    assert!(switch.change(true));
    assert_eq!(*sent.borrow(), [true]);
}

#[test]
fn a_control_is_a_view_and_a_press_on_it_reaches_it() {
    let sent = Rc::new(RefCell::new(Vec::new()));
    let mut switch = Switch::telling(&sent);
    assert!(send(&mut switch, &PEvent::PointerDown(PPoint::new(5.0, 5.0))));
    assert!(!send(&mut switch, &PEvent::PointerDown(PPoint::new(500.0, 5.0))));
    assert!(!send(&mut switch, &PEvent::PointerMove(PPoint::new(5.0, 5.0))));
    assert_eq!(*sent.borrow(), [true]);
}
