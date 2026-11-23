//! The steps of `FLOW.md` that no other test already is: starting to
//! type, the chat's name, the activity bar and its menus, the profile,
//! and the window made narrow. Each works through the stage, as
//! a hand does. The bar's own steps are in `flow/bar.rs`.

mod bar;

use appkit::pui::core::{PEvent, PMode, PPoint, PRect};
use appkit::pui::window::PStage;
use appkit::theme::PLook;

use crate::Phigen;
use crate::test::{DESK, HAND, listed, pictured, rect, shown, staged};

/// A press at the place `find` gives for the stage as it is laid out now.
fn press(stage: &mut PStage, find: impl Fn(&PStage) -> PPoint) -> bool {
    let at = find(stage);
    stage.send(&PEvent::PointerDown(at))
}

/// The names of the parts shown, in order.
fn part(stage: &PStage) -> Vec<&str> {
    stage.slot().iter().map(|slot| slot.part.as_str()).collect()
}

/// How often a line of text is drawn.
fn count(stage: &PStage, look: &PLook, text: &str) -> usize {
    shown(stage, look).iter().filter(|line| *line == text).count()
}

/// A press on the bar's icon of that name, and one on the row of its
/// menu that reads so.
fn icon(stage: &mut PStage, look: &PLook, name: &str) -> bool {
    press(stage, |stage| pictured(stage, look, name).expect("on the bar"))
}

fn row(stage: &mut PStage, look: &PLook, text: &str) -> bool {
    press(stage, |stage| listed(stage, look, text).expect("a row of the menu"))
}

/// The user, at the end of the bar.
fn user(stage: &PStage) -> PPoint {
    let bar = stage.frame().activity;
    PPoint::new(bar.origin.x + bar.size.width - 28.0, bar.origin.y + bar.size.height - 28.0)
}

/// The pin, in the caption: it is Phigen's mark, over the bar, which
/// shows the pin while the pointer is over it.
fn fold(stage: &PStage) -> PPoint {
    let frame = stage.frame();
    let (high, bar) = (frame.caption.size.height, frame.activity);
    PPoint::new(bar.origin.x + bar.size.width / 2.0, high / 2.0)
}

#[test]
fn step_2_i_can_type_at_once_without_pressing_anything() {
    let (_, mut stage, look) = staged(DESK);
    assert_eq!(stage.focus(), Some("ask"));
    assert!(stage.send(&PEvent::KeyDown('h')));
    assert!(shown(&stage, &look).contains(&"h".to_string()));
}

#[test]
fn step_10_the_chat_is_called_by_my_first_question_over_the_talk_and_in_the_list() {
    let (_, mut stage, look) = staged(DESK);
    Phigen::key(&mut stage, "what is phikit?");
    Phigen::key(&mut stage, "and phiray?");
    // In the list, over the talk, and as the bubble it was said in.
    assert_eq!(count(&stage, &look, "what is phikit?"), 3);
}

#[test]
fn step_15_my_row_opens_my_profile_as_a_square_in_the_middle_of_the_window() {
    for size in [DESK, HAND] {
        let (_, mut stage, look) = staged(size);
        assert!(press(&mut stage, user));
        let panel = stage.panel().expect("the profile is open");
        assert_eq!(panel.rect.size.width, panel.rect.size.height);
        // It fills half of the room under the caption, where that is no
        // more than 92 of a hundred of that room's shorter side.
        let (wide, tall) = (size.width, size.height - stage.frame().caption.size.height);
        let (half, most) = ((wide * tall / 2.0).sqrt(), wide.min(tall) * 0.92);
        assert!((panel.rect.size.width - half.min(most)).abs() < 0.01, "{:?}", panel.rect);
        let (middle, wanted) = (panel.rect.center(), PPoint::new(wide / 2.0, size.height - tall / 2.0));
        assert!((middle.x - wanted.x).abs() < 0.01 && (middle.y - wanted.y).abs() < 0.01, "{middle:?}");
        // Who I am is the rows of data/user.tsv; under them what is true
        // now, asked once a frame, and the looks to choose among.
        stage.tick();
        let text = shown(&stage, &look);
        assert!(["Profile", "Guest", "not signed in", "About", "Model", "No model", "Look", "Light", "Chats", "Appearance", "Dark", "Glass", "Glass Light"].iter().all(|row| text.contains(&row.to_string())), "{text:?}");
    }
}

#[test]
fn step_15_the_profile_grows_to_the_window_and_back_and_closes_three_ways() {
    let (app, mut stage, _) = staged(DESK);
    // The whole of what is under the caption, which stays clear.
    let whole = PRect::new(0.0, 44.0, DESK.width, DESK.height - 44.0);
    press(&mut stage, user);
    let square = stage.panel().expect("open").rect;
    assert!(press(&mut stage, |stage| stage.panel().expect("open").expand.center()));
    assert_eq!(stage.panel().map(|panel| panel.rect), Some(whole));
    assert!(press(&mut stage, |stage| stage.panel().expect("open").expand.center()));
    assert_eq!(stage.panel().map(|panel| panel.rect), Some(square));
    // While it is open what is typed is not said in the chat.
    Phigen::key(&mut stage, "not said");
    assert_eq!(app.desk().history.borrow().current().title(), None);
    // Its own button, the Escape key and a press beside it each close it.
    assert!(press(&mut stage, |stage| stage.panel().expect("open").close.center()));
    assert_eq!(stage.panel(), None);
    press(&mut stage, user);
    assert!(stage.send(&PEvent::KeyDown('\u{1b}')) && stage.panel().is_none());
    press(&mut stage, user);
    assert!(stage.send(&PEvent::PointerDown(PPoint::new(5.0, 5.0))) && stage.panel().is_none());
    // It opens as a square again after it was closed large.
    press(&mut stage, user);
    press(&mut stage, |stage| stage.panel().expect("open").expand.center());
    press(&mut stage, |stage| stage.panel().expect("open").close.center());
    press(&mut stage, user);
    assert_eq!(stage.panel().map(|panel| panel.rect), Some(square));
}

#[test]
fn step_16_made_narrow_the_same_window_is_a_mobile_app_with_my_chat_still_in_it() {
    let (_, mut stage, look) = staged(DESK);
    Phigen::key(&mut stage, "still here");
    stage.lay(HAND, PMode::of_width(HAND.width));
    assert_eq!(part(&stage), ["title", "talk", "ask"]);
    assert!(shown(&stage, &look).contains(&"still here".to_string()));
    // And wide again it is as it was.
    stage.lay(DESK, PMode::of_width(DESK.width));
    assert_eq!(part(&stage), ["title", "talk", "ask", "thread"]);
}

#[test]
fn step_23_more_chats_than_fit_are_scrolled_in_the_bar_and_in_the_search_window() {
    let (_, mut stage, look) = staged(DESK);
    for number in 0..30 {
        Phigen::key(&mut stage, &format!("chat number {number}"));
        row(&mut stage, &look, "New chat");
    }
    // The newest are at the top of the list; the first is far below it.
    let (first, last) = ("chat number 0".to_string(), "chat number 29".to_string());
    assert!(shown(&stage, &look).contains(&last) && !shown(&stage, &look).contains(&first));
    let thread = rect(&stage, "thread");
    assert!(stage.send(&PEvent::Scroll(thread.center(), PPoint::new(0.0, -100000.0))));
    assert!(shown(&stage, &look).contains(&first) && !shown(&stage, &look).contains(&last));
    // A press on a title that was scrolled to opens that chat: the last
    // row is the first chat.
    assert!(stage.send(&PEvent::PointerDown(PPoint::new(thread.origin.x + 60.0, thread.origin.y + thread.size.height - 20.0))));
    assert!(shown(&stage, &look).iter().filter(|text| **text == first).count() >= 2);
    // The Search window scrolls its chats under its pills in the same way.
    row(&mut stage, &look, "Search");
    let find = rect(&stage, "find");
    let listed = |stage: &PStage, text: &String| shown(stage, &look).iter().filter(|line| *line == text).count();
    let before = listed(&stage, &last);
    assert!(stage.send(&PEvent::Scroll(find.center(), PPoint::new(0.0, -100000.0))));
    assert!(listed(&stage, &last) < before && shown(&stage, &look).contains(&"Find a chat".to_string()));
}

#[test]
fn step_24_a_new_chat_has_the_field_in_the_middle_and_ways_to_begin_under_it() {
    let (app, mut stage, look) = staged(DESK);
    assert_eq!(part(&stage), ["title", "hello", "ask", "card", "thread"]);
    // The field is halfway down what the header over it leaves.
    let (ask, body, head) = (rect(&stage, "ask"), stage.frame().body, rect(&stage, "title"));
    assert_eq!(ask.center().y, body.center().y + head.size.height / 2.0);
    let text = shown(&stage, &look);
    assert!(["What shall we work on?", "Ask a question", "See what Phigen can do", "Think something through"].iter().all(|line| text.contains(&line.to_string())), "{text:?}");
    // The cards are side by side under the field; a press on the second
    // says what it stands for, and the chat is shown.
    let card = rect(&stage, "card");
    assert!(stage.send(&PEvent::PointerDown(PPoint::new(card.center().x, card.origin.y + 40.0))));
    assert_eq!(app.desk().history.borrow().current().title(), Some("What can you help me with?"));
    assert_eq!(part(&stage), ["title", "talk", "ask", "thread"]);
    // New chat brings the start back.
    row(&mut stage, &look, "New chat");
    assert_eq!(part(&stage), ["title", "hello", "ask", "card", "thread"]);
}

#[test]
fn step_25_my_first_time_i_am_welcomed_in_a_square_with_the_ways_to_begin() {
    use appkit::pui::window::PDeed;
    let open = |stage: &mut PStage| {
        stage.window.apply(&PDeed::Open("welcome".to_string()));
        stage.lay(DESK, PMode::Desktop);
    };
    let (app, mut stage, look) = staged(DESK);
    assert!(app.fresh());
    open(&mut stage);
    let panel = stage.panel().expect("the welcome is open");
    assert_eq!(panel.rect.size.width, panel.rect.size.height);
    let text = shown(&stage, &look);
    assert!(["Welcome", "Welcome to Phigen", "Ask a question", "Chat now"].iter().all(|line| text.contains(&line.to_string())), "{text:?}");
    // Its button closes it and says nothing.
    let body = panel.body;
    assert!(stage.send(&PEvent::PointerDown(PPoint::new(body.origin.x + body.size.width - 50.0, body.origin.y + body.size.height - 30.0))));
    assert!(stage.panel().is_none() && app.fresh());
    // The first card closes it and says nothing either: it lets me type.
    // The cards are one under the other, 76 down and 16 apart, sharing
    // what the heading and the button, 44 high with 16 around it, leave,
    // and none higher than a card is: 96.
    let high = ((body.size.height - 76.0 - 76.0 - 32.0) / 3.0).min(96.0);
    let card = |index: usize| PPoint::new(body.center().x, body.origin.y + 76.0 + index as f32 * (high + 16.0) + high / 2.0);
    open(&mut stage);
    assert!(stage.send(&PEvent::PointerDown(card(0))));
    assert!(stage.panel().is_none() && app.fresh());
    // The third closes it and says what it stands for.
    open(&mut stage);
    assert!(stage.send(&PEvent::PointerDown(card(2))));
    assert!(stage.panel().is_none() && !app.fresh());
    assert_eq!(part(&stage), ["title", "talk", "ask", "thread"]);
}

#[test]
fn step_33_the_settings_come_as_a_drawer_at_the_right_and_the_models_as_a_sheet_at_the_foot() {
    use appkit::pui::window::PDeed;
    let rect = |size, modal: &str| {
        let (_, mut stage, _) = staged(size);
        stage.window.apply(&PDeed::Open(modal.to_string()));
        stage.lay(size, PMode::of_width(size.width));
        stage.panel().expect("open").rect
    };
    // At a desk: 400 wide down the right edge, and half the window high
    // across the body's column at the foot, each 8 clear of the edges
    // and of the caption, whose 44 are not theirs.
    assert_eq!([rect(DESK, "setting"), rect(DESK, "model")], [PRect::new(632.0, 52.0, 400.0, 660.0), PRect::new(132.0, 374.0, 776.0, 338.0)]);
    // In a hand each is as wide as the window leaves.
    assert_eq!([rect(HAND, "setting"), rect(HAND, "model")], [PRect::new(8.0, 52.0, 374.0, 740.0), PRect::new(8.0, 414.0, 374.0, 378.0)]);
    // A press beside a drawer closes it, as it closes a square.
    let (_, mut stage, _) = staged(DESK);
    stage.window.apply(&PDeed::Open("setting".to_string()));
    stage.lay(DESK, PMode::Desktop);
    assert!(stage.send(&PEvent::PointerDown(PPoint::new(300.0, 300.0))) && stage.panel().is_none());
}
