use appkit::pui::core::{PEvent, PPoint};
use appkit::theme::PTheme;

use crate::Phigen;
use crate::test::{DESK, HAND, drawn, lit, rect, shown, staged};

#[test]
fn the_person_s_turn_is_on_the_right_and_phigen_s_on_the_left() {
    for size in [DESK, HAND] {
        let (app, mut stage, look) = staged(size);
        Phigen::key(&mut stage, "hello");
        let (mark, talk) = (drawn(&stage, &look), rect(&stage, "talk"));
        // The turns keep to the body's column: a gap in, in a hand.
        let (start, wide) = app.desk().column(talk.size.width);
        // A turn is a band of the label's light: mine is lit from its right.
        let person = lit(&mark, talk, 90.0, true);
        assert_eq!(person.len(), 1);
        let right = person[0].origin.x + person[0].size.width;
        assert!((right - (talk.origin.x + start + wide)).abs() < 0.01, "{right}");
        // The greeting and the answer.
        let phigen = lit(&mark, talk, 90.0, false);
        assert_eq!(phigen.len(), 2);
        assert!(phigen.iter().all(|turn| turn.origin.x == talk.origin.x + start));
        // No bubble is wider than the column, and the newest is the lowest.
        assert!(person.iter().chain(&phigen).all(|turn| turn.size.width <= wide));
        assert!(phigen[0].origin.y > person[0].origin.y && person[0].origin.y > phigen[1].origin.y);
    }
}

#[test]
fn a_turn_too_long_for_a_line_is_set_in_several() {
    let (_, mut stage, look) = staged(HAND);
    let long = "one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen";
    Phigen::key(&mut stage, long);
    let person = lit(&drawn(&stage, &look), rect(&stage, "talk"), 90.0, true);
    let line = look.size("text.line").expect("in the theme");
    assert!(person[0].size.height > line * 3.0, "{person:?}");
}

#[test]
fn step_17_a_long_chat_is_scrolled_back_with_the_wheel_and_comes_back_when_i_say_something() {
    let (app, mut stage, look) = staged(DESK);
    (0..12).for_each(|number| Phigen::key(&mut stage, &format!("question {number}")));
    let heard = |number: usize| app.desk().text.fill("reply", "said", &format!("question {number}"));
    // The newest answer is shown; the first is above the talk.
    let text = shown(&stage, &look);
    assert!(text.contains(&heard(11)) && !text.contains(&heard(0)));
    let talk = rect(&stage, "talk");
    assert!(stage.send(&PEvent::Scroll(talk.center(), PPoint::new(0.0, 100000.0))));
    let text = shown(&stage, &look);
    assert!(text.contains(&heard(0)) && !text.contains(&heard(11)));
    // The wheel the other way goes forward again, and no further than
    // the newest turn.
    stage.send(&PEvent::Scroll(talk.center(), PPoint::new(0.0, -100000.0)));
    assert!(shown(&stage, &look).contains(&heard(11)));
    stage.send(&PEvent::Scroll(talk.center(), PPoint::new(0.0, 100000.0)));
    Phigen::key(&mut stage, "the last");
    assert!(shown(&stage, &look).contains(&"the last".to_string()));
}

#[test]
fn step_17_a_chat_that_fits_does_not_scroll() {
    let (_, mut stage, look) = staged(DESK);
    Phigen::key(&mut stage, "a short chat");
    let before = shown(&stage, &look);
    let talk = rect(&stage, "talk");
    assert!(!stage.send(&PEvent::Scroll(talk.center(), PPoint::new(0.0, 500.0))));
    assert_eq!(shown(&stage, &look), before);
}
