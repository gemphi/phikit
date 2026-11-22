//! A room's tools, worked as a hand works them in the window: writing,
//! answering, liking, marking, pinning and searching.

use appkit::pui::core::{PEvent, PMark, PPoint, PRect};
use appkit::pui::window::PStage;
use appkit::theme::PLook;

use super::{DESK, drawn, rect, shown, staged, written};
use crate::Phiogy;

/// The lobby of the small site, open in a window at a desk.
fn lobby() -> (Phiogy, PStage, std::rc::Rc<PLook>) {
    let (app, mut stage, look) = staged(DESK);
    assert!(app.desk().show("lobby"));
    stage.send(&PEvent::KeyUp(' '));
    (app, stage, look)
}

/// Where the icon of that name was drawn last.
pub(super) fn icon(stage: &PStage, look: &PLook, wanted: &str) -> Option<PRect> {
    drawn(stage, look).iter().rev().find_map(|mark| match mark {
        PMark::Icon { name, rect, .. } if name == wanted => Some(*rect),
        _ => None,
    })
}

/// A press on the control with that icon.
pub(super) fn tool(stage: &mut PStage, look: &PLook, name: &str) -> bool {
    let at = icon(stage, look, name).unwrap_or_else(|| panic!("the control {name}")).center();
    stage.send(&PEvent::PointerDown(at))
}

/// Type some words, a key a letter.
pub(super) fn typed(stage: &mut PStage, words: &str) {
    words.chars().for_each(|key| _ = stage.send(&PEvent::KeyDown(key)));
}

/// Bring the pointer over a line of the stream.
fn over(stage: &mut PStage, look: &PLook, line: &str) -> bool {
    let at = written(stage, look, line).unwrap_or_else(|| panic!("the line {line}"));
    stage.send(&PEvent::PointerMove(at))
}

#[test]
fn step_20_what_i_write_in_a_room_is_said_in_it_at_once() {
    let (app, mut stage, look) = lobby();
    // The field at the stream's foot says what it is for until written in.
    let hint = app.desk().text.fill("stream.write", "room", "lobby");
    assert!(shown(&stage, &look).contains(&hint));
    typed(&mut stage, "Hi all");
    let text = shown(&stage, &look);
    assert!(text.contains(&"Hi all".to_string()) && !text.contains(&hint), "{text:?}");
    // Enter says it: it is in the stream, by me, today, under the last
    // message; and the field is empty again.
    assert!(stage.send(&PEvent::KeyDown(PEvent::ENTER)));
    assert_eq!(app.desk().said().last().map(|one| (one.by.clone(), one.text.clone())), Some(("You".to_string(), "Hi all".to_string())));
    let text = shown(&stage, &look);
    assert!(text.contains(&hint) && text.contains(&"Hi all".to_string()));
    let (said, before) = (written(&stage, &look, "Hi all").expect("drawn"), written(&stage, &look, "I will bring both versions.").expect("drawn"));
    assert!(said.y > before.y && rect(&stage, "room").contains(said), "{said:?} {before:?}");
    // The control beside the field says what was written as Enter does;
    // an empty field says nothing.
    typed(&mut stage, "Second");
    assert!(tool(&mut stage, &look, "send") && tool(&mut stage, &look, "send"));
    assert_eq!(app.desk().said().len(), 5);
}

#[test]
fn step_21_a_message_under_the_pointer_is_answered_liked_and_pinned() {
    let (app, mut stage, look) = lobby();
    let desk = app.desk();
    // No message has its controls until the pointer is over one.
    assert!(icon(&stage, &look, "reply").is_none());
    assert!(over(&mut stage, &look, "Good. Show it at the stand up."));
    // What a control is called is shown by it under the pointer.
    let reply = icon(&stage, &look, "reply").expect("its controls").center();
    assert!(stage.send(&PEvent::PointerMove(reply)));
    assert!(shown(&stage, &look).contains(&desk.text.get("room.reply").to_string()));
    // Answer it: the field says whom, and what is written answers it.
    assert!(stage.send(&PEvent::PointerDown(reply)));
    let whom = desk.text.fill("stream.answer", "name", "Lora Vale");
    assert!(shown(&stage, &look).contains(&whom));
    typed(&mut stage, "On it");
    assert!(stage.send(&PEvent::KeyDown(PEvent::ENTER)));
    let text = shown(&stage, &look);
    assert_eq!(desk.said().last().map(|one| one.reply.clone()), Some("d2".to_string()));
    assert!(!text.contains(&whom) && text.iter().filter(|line| line.starts_with("Lora Vale: Good.")).count() == 1, "{text:?}");
    // The way out of an answer, and Escape, answer nobody.
    assert!(over(&mut stage, &look, "Good. Show it at the stand up.") | tool(&mut stage, &look, "reply"));
    assert!(tool(&mut stage, &look, "x") && desk.now.answer.borrow().is_empty());
    // Like it, and its heart counts me; a press on the mark takes it back.
    assert!(tool(&mut stage, &look, "heart") && desk.gave("d2", "heart"));
    assert!(shown(&stage, &look).contains(&"2".to_string()));
    let thumb = icon(&stage, &look, "thumbs-up").expect("its mark").center();
    assert!(stage.send(&PEvent::PointerDown(thumb)) && shown(&stage, &look).contains(&"4".to_string()));
    assert!(stage.send(&PEvent::PointerDown(thumb)) && !shown(&stage, &look).contains(&"4".to_string()));
    // Pin it: the control turns to unpinning, and it is among the pinned.
    assert!(tool(&mut stage, &look, "pin") && desk.pinned(&desk.said()[1]));
    assert!(icon(&stage, &look, "pin-off").is_some());
}

#[test]
fn step_22_a_room_is_searched_and_its_pinned_messages_listed_from_its_title() {
    let (app, mut stage, look) = lobby();
    let desk = app.desk();
    let title = rect(&stage, "title");
    // The two tools of the title: the last icons of theirs drawn there.
    let tools = |stage: &PStage, name: &str| drawn(stage, &look).iter().find_map(|mark| match mark {
        PMark::Icon { name: known, rect, .. } if known == name && title.contains(rect.center()) => Some(rect.center()),
        _ => None,
    });
    // Pinned: the site's own, each a line that leads to its message.
    assert!(stage.send(&PEvent::PointerDown(tools(&stage, "pin").expect("the pins tool"))));
    let text = shown(&stage, &look);
    assert!(text.contains(&desk.text.get("stream.pins").to_string()) && text.iter().filter(|line| line.starts_with("Tavi Reyes: The mark")).count() == 2, "{text:?}");
    let row = drawn(&stage, &look).iter().find_map(|mark| match mark {
        PMark::Text { at, text, .. } if text.starts_with("Tavi Reyes: The mark") => Some(at.plus(PPoint::new(4.0, 4.0))),
        _ => None,
    });
    assert!(stage.send(&PEvent::PointerDown(row.expect("its row"))));
    assert!(!desk.now.pins.get() && !shown(&stage, &look).contains(&desk.text.get("stream.pins").to_string()));
    // Search: a field over the stream; only what holds the words shows.
    assert!(stage.send(&PEvent::PointerDown(tools(&stage, "search").expect("the search tool"))));
    assert!(desk.now.finding.get() && shown(&stage, &look).contains(&desk.text.fill("stream.find", "room", "lobby")));
    typed(&mut stage, "stand");
    let text = shown(&stage, &look);
    assert!(text.contains(&"Good. Show it at the stand up.".to_string()) && !text.contains(&"I will bring both versions.".to_string()), "{text:?}");
    // Nothing was written in the room by typing in the search.
    assert_eq!(desk.said().len(), 3);
    typed(&mut stage, "zzz");
    assert!(shown(&stage, &look).contains(&desk.text.get("stream.unfound").to_string()));
    // Escape leaves the search, and everything said shows again.
    assert!(stage.send(&PEvent::KeyDown(PEvent::ESCAPE)) && !desk.now.finding.get());
    stage.tick();
    let text = shown(&stage, &look);
    assert!(text.contains(&"I will bring both versions.".to_string()) && !text.contains(&"standzzz".to_string()), "{text:?}");
}
