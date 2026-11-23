use appkit::pui::core::{PEvent, PPoint};
use appkit::pui::window::PStage;

use crate::Phigen;
use crate::test::{DESK, rect, shown, staged};

#[test]
fn what_is_typed_shows_in_the_field_and_enter_says_it_and_empties_the_field() {
    let (app, mut stage, look) = staged(DESK);
    "hi there".chars().for_each(|key| _ = stage.send(&PEvent::KeyDown(key)));
    assert!(shown(&stage, &look).contains(&"hi there".to_string()));
    assert!(!shown(&stage, &look).contains(&"Ask Phigen".to_string()));
    assert!(stage.send(&PEvent::KeyDown('\n')));
    assert_eq!(app.desk().history.borrow().current().title(), Some("hi there"));
    // The field is empty again and says what it is for.
    assert!(shown(&stage, &look).contains(&"Ask Phigen".to_string()));
}

#[test]
fn backspace_takes_the_last_letter_away() {
    let (_, mut stage, look) = staged(DESK);
    "hix\u{8}".chars().for_each(|key| _ = stage.send(&PEvent::KeyDown(key)));
    assert!(shown(&stage, &look).contains(&"hi".to_string()));
}

#[test]
fn a_press_on_send_says_what_the_field_holds() {
    let (app, mut stage, _) = staged(DESK);
    "by button".chars().for_each(|key| _ = stage.send(&PEvent::KeyDown(key)));
    // The button is at the right of the ask, a gap in from its edges.
    let ask = rect(&stage, "ask");
    let on_send = PPoint::new(ask.origin.x + ask.size.width - 40.0, ask.origin.y + 40.0);
    assert!(stage.send(&PEvent::PointerDown(on_send)));
    assert_eq!(app.desk().history.borrow().current().title(), Some("by button"));
}

#[test]
fn nothing_is_said_when_the_field_is_blank() {
    let (app, mut stage, _) = staged(DESK);
    Phigen::key(&mut stage, "   ");
    assert_eq!(app.desk().history.borrow().current().turn.len(), 1);
}

#[test]
fn step_18_a_caret_stands_after_what_i_have_typed() {
    let (_, mut stage, look) = staged(DESK);
    // The caret is as wide as the theme's line, 2, and as high as a line
    // of text, 22.
    let caret = |stage: &PStage| {
        let mark = crate::test::drawn(stage, &look);
        let left = |mark: &appkit::pui::core::PMark| match mark {
            appkit::pui::core::PMark::Fill { rect, .. } if rect.size.width == 2.0 && rect.size.height == 22.0 => Some(rect.origin.x),
            _ => None,
        };
        mark.iter().find_map(left).expect("the field has a caret")
    };
    let before = caret(&stage);
    "ab".chars().for_each(|key| _ = stage.send(&PEvent::KeyDown(key)));
    // Two letters, 7.5 wide each as a test counts them.
    assert_eq!(caret(&stage), before + 15.0);
}

#[test]
fn step_22_i_can_go_back_in_what_i_typed_and_mend_it() {
    let (_, mut stage, look) = staged(DESK);
    "helo wrld".chars().for_each(|key| _ = stage.send(&PEvent::KeyDown(key)));
    // Back over "o wrld" to put the l in, then on past "o w" for the o.
    (0..6).for_each(|_| _ = stage.send(&PEvent::KeyDown(PEvent::LEFT)));
    stage.send(&PEvent::KeyDown('l'));
    (0..3).for_each(|_| _ = stage.send(&PEvent::KeyDown(PEvent::RIGHT)));
    stage.send(&PEvent::KeyDown('o'));
    assert!(shown(&stage, &look).contains(&"hello world".to_string()));
    // Home and Delete take the first letter away; End and Backspace the last.
    [PEvent::HOME, PEvent::DELETE, PEvent::END, PEvent::BACKSPACE].into_iter().for_each(|key| _ = stage.send(&PEvent::KeyDown(key)));
    assert!(shown(&stage, &look).contains(&"ello worl".to_string()));
}
