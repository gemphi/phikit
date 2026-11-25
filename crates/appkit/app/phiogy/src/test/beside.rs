//! A person beside the room: opened by a press on a face, with what
//! they say of themself, what to do, and the three panes; their
//! pictures open in the viewer.

use appkit::pui::core::{PEvent, PFit, PMark, PMode, PPoint, PRect, PSize};
use appkit::pui::window::PStage;
use appkit::theme::PLook;

use super::{DESK, HAND, drawn, press, rect, shown, staged};

/// Where a face of that cell of a sheet was drawn last.
fn face(stage: &PStage, look: &PLook, index: u32) -> Option<PRect> {
    drawn(stage, look).iter().rev().find_map(|mark| match mark {
        PMark::Picture { rect, fit: PFit::Cell { index: known, .. }, .. } if *known == index => Some(*rect),
        _ => None,
    })
}

/// Lay the window out again, as a frame does, and say what it shows.
fn laid(stage: &mut PStage, look: &PLook, size: PSize) -> Vec<String> {
    stage.lay(size, PMode::of_width(size.width));
    shown(stage, look)
}

#[test]
fn step_24_a_press_on_a_face_puts_that_person_beside_the_room() {
    let (app, mut stage, look) = staged(DESK);
    let desk = app.desk();
    assert!(desk.show("lobby"));
    let text = laid(&mut stage, &look, DESK);
    assert!(text.contains(&"On the line".to_string()) && !text.contains(&"Profile".to_string()));
    // Tavi's face, beside what they said first: a cell of a sheet of
    // their own. The room opens at its foot, so it is scrolled to.
    let room = rect(&stage, "room");
    assert!(stage.send(&PEvent::Scroll(room.origin.plus(PPoint::new(60.0, 60.0)), PPoint::new(0.0, 9999.0))));
    assert!(stage.send(&PEvent::PointerDown(face(&stage, &look, 9).expect("the face").center())));
    assert_eq!(desk.now.who.borrow().as_deref(), Some("tavi"));
    // They stand where the people of the room stood, beside the stream.
    let text = laid(&mut stage, &look, DESK);
    let words = |name: &str| desk.text.get(name).to_string();
    assert!(text.contains(&words("person.title")) && !text.contains(&"On the line".to_string()), "{text:?}");
    assert!(text.contains(&"I will bring both versions.".to_string()));
    assert!(text.contains(&"tavi · Member · Away".to_string()) && text.contains(&"Member since · Jun 2026".to_string()), "{text:?}");
    // The tabs (the kit's form, as Phiegg's profile has them) say how
    // much each holds: who they are linked to, the spaces they are in,
    // the pictures they posted.
    let pane = |name: &str, count: usize| format!("{} {count}", words(name));
    // Their picture is large at the top: the wheel lifts it away over
    // the tabs first.
    let side = rect(&stage, "room");
    assert!(stage.send(&PEvent::Scroll(PPoint::new(side.origin.x + side.size.width - 40.0, side.center().y), PPoint::new(0.0, -400.0))));
    laid(&mut stage, &look, DESK);
    // (A row of tabs shows a few at a time: its arrows bring the rest.)
    assert!(text.contains(&words("person.info")), "{text:?}");
    assert!([pane("person.linked", 1), pane("person.space", 1), pane("person.shot", 1)].iter().all(|pane| super::tab_to(&mut stage, &look, "room", pane)));
    assert!(super::tab_to(&mut stage, &look, "room", &pane("person.space", 1)) && press(&mut stage, &look, &pane("person.space", 1)) && shown(&stage, &look).contains(&"Phiace".to_string()));
    // Their pictures open in the viewer, among their own alone.
    assert!(super::tab_to(&mut stage, &look, "room", &pane("person.shot", 1)) && press(&mut stage, &look, &pane("person.shot", 1)));
    let side = rect(&stage, "room");
    let shot = drawn(&stage, &look).iter().rev().find_map(|mark| match mark {
        // Its top, which is in sight under the tabs.
        PMark::Picture { name, rect, .. } if name == "phiegg:worlds/w_brocade.png" && rect.origin.x > side.origin.x + side.size.width / 2.0 => Some(rect.origin.plus(PPoint::new(16.0, 16.0))),
        _ => None,
    });
    assert!(stage.send(&PEvent::PointerDown(shot.expect("their picture"))));
    assert_eq!((stage.window.modal(), desk.now.album.borrow().len(), desk.view.get()), (Some("view"), 1, Some(0)));
    assert!(stage.send(&PEvent::KeyDown(PEvent::ESCAPE)));
    // Who they are linked to is read when asked; a press opens them.
    assert!(super::tab_to(&mut stage, &look, "room", &pane("person.linked", 1)) && press(&mut stage, &look, &pane("person.linked", 1)));
    // Their card is under the tabs: the profile is scrolled to it.
    let tab = super::written(&stage, &look, &pane("person.linked", 1)).expect("the tab");
    stage.send(&PEvent::Scroll(tab, PPoint::new(0.0, -100000.0)));
    let linked = drawn(&stage, &look).iter().find_map(|mark| match mark {
        PMark::Text { at, text, .. } if text == "Lora Vale" && at.x > side.origin.x + side.size.width / 2.0 => Some(at.plus(PPoint::new(2.0, 2.0))),
        _ => None,
    });
    assert!(stage.send(&PEvent::PointerDown(linked.expect("who they are linked to"))));
    assert_eq!(desk.now.who.borrow().as_deref(), Some("lora"));
    let text = shown(&stage, &look);
    assert!(text.contains(&"I run Lora Land and the Spark show.".to_string()) && text.contains(&"Host".to_string()) && text.contains(&pane("person.linked", 2)), "{text:?}");
    // Widened over the room, to see all of them while talking: what was
    // said gives it its place, and comes back when it is put beside.
    assert!(press(&mut stage, &look, &words("person.wide")) && desk.now.wide.get());
    let text = laid(&mut stage, &look, DESK);
    assert!(!text.contains(&"I will bring both versions.".to_string()) && text.contains(&words("person.narrow")), "{text:?}");
    assert!(press(&mut stage, &look, &words("person.narrow")) && !desk.now.wide.get());
    laid(&mut stage, &look, DESK);
    // Follow them, and the control says so; it is kept.
    assert!(press(&mut stage, &look, &words("person.follow")));
    assert!(desk.follows("lora") && shown(&stage, &look).contains(&words("person.following")));
    // Write to them: their own room opens, and nobody stands beside it.
    assert!(press(&mut stage, &look, &words("person.message")));
    assert_eq!((desk.room(), desk.now.who.borrow().clone()), ("lora".to_string(), None));
}

#[test]
fn the_way_back_from_a_person_is_at_their_head_and_in_a_hand_they_have_the_room() {
    let (app, mut stage, look) = staged(HAND);
    let desk = app.desk();
    // In a direct room the title has their face: a press opens them.
    stage.send(&PEvent::KeyUp(' '));
    laid(&mut stage, &look, HAND);
    let title = rect(&stage, "title");
    let at = drawn(&stage, &look).iter().find_map(|mark| match mark {
        PMark::Picture { rect, fit: PFit::Cell { .. }, .. } if title.contains(rect.center()) => Some(rect.center()),
        _ => None,
    });
    assert!(stage.send(&PEvent::PointerDown(at.expect("the title's face"))));
    assert_eq!(desk.now.who.borrow().as_deref(), Some("lora"));
    // Narrow, the person has the room: what was said is not under them.
    let text = laid(&mut stage, &look, HAND);
    assert!(text.contains(&desk.text.get("person.title").to_string()) && !text.contains(&desk.text.get("stream.none").to_string()), "{text:?}");
    // The way back is at their head.
    assert!(press(&mut stage, &look, desk.text.get("person.title")));
    assert_eq!(desk.now.who.borrow().clone(), None);
    assert!(laid(&mut stage, &look, HAND).contains(&desk.text.get("stream.none").to_string()));
}
