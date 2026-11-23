//! The pages around the rooms, worked as a hand works them: the hosts
//! (joining one, making one), friends, and finding somebody to begin a
//! conversation with.

use appkit::pui::core::{PEvent, PMark, PMode, PPoint};
use appkit::pui::window::{PDeed, PStage};
use appkit::theme::PLook;
use serde_json::json;

use super::tools::{icon, typed};
use super::{DESK, drawn, part, rect, shown, staged};
use crate::Phiogy;
use crate::part::bond::{ASKED, NONE, SENT, WITH};

/// The window on one of its pages, laid out.
fn on(page: &str) -> (Phiogy, PStage, std::rc::Rc<PLook>) {
    let (app, mut stage, look) = staged(DESK);
    assert!(app.desk().show(page), "{page}");
    stage.send(&PEvent::KeyUp(' '));
    stage.lay(DESK, PMode::of_width(DESK.width));
    (app, stage, look)
}

/// A press on the first line of words that reads so in a part.
fn press(stage: &mut PStage, look: &PLook, name: &str, wanted: &str) -> bool {
    let inside = rect(stage, name);
    let at = drawn(stage, look).iter().find_map(|mark| match mark {
        PMark::Text { at, text, .. } | PMark::Head { at, text, .. } if text == wanted && inside.contains(*at) => Some(at.plus(PPoint::new(2.0, 2.0))),
        _ => None,
    });
    let pressed = stage.send(&PEvent::PointerDown(at.unwrap_or_else(|| panic!("{wanted} in {name}"))));
    stage.lay(DESK, PMode::of_width(DESK.width));
    pressed
}

/// The places of the rail, by what each is called: all but the shop
/// the small site has open, which stands last.
fn rail(app: &Phiogy) -> Vec<String> {
    app.desk().rail().into_iter().map(|(label, _)| label).filter(|label| label != "Shop One").collect()
}

#[test]
fn step_25_the_hosts_are_listed_to_join_and_one_that_is_joined_stands_in_my_rail() {
    let (app, mut stage, look) = staged(DESK);
    let desk = app.desk();
    assert!(stage.window.apply(&PDeed::Show("venue".to_string())));
    stage.lay(DESK, PMode::of_width(DESK.width));
    assert_eq!(part(&stage), ["venue", "guide"]);
    // Every host of the catalogue, with how many people it has.
    let text = shown(&stage, &look);
    let word = |name: &str| desk.text.get(name).to_string();
    assert!([word("host.create"), word("host.join"), "4,102 members".to_string(), "90 members".to_string(), desk.text.fill("host.count", "n", "2")].iter().all(|line| text.contains(line)), "{text:?}");
    // One the app's document says is joined can be left; another joined.
    assert_eq!(rail(&app), ["Direct", "Phiace", "Opal Atelier"]);
    assert!(press(&mut stage, &look, "venue", &word("host.enter")));
    assert_eq!(rail(&app), ["Direct", "Phiace", "Opal Atelier", "The Swan Salon"]);
    assert_eq!(desk.kept.get("host")["phiogy/h_swan"], json!(true));
    assert!(press(&mut stage, &look, "venue", &word("host.leave")));
    assert_eq!(rail(&app), ["Direct", "Phiace", "The Swan Salon"]);
    // The field searches them by name.
    typed(&mut stage, "swan");
    let text = shown(&stage, &look);
    assert!(text.contains(&"90 members".to_string()) && !text.contains(&"4,102 members".to_string()), "{text:?}");
    typed(&mut stage, "zz");
    assert!(shown(&stage, &look).contains(&word("host.none")));
    (0..2).for_each(|_| _ = stage.send(&PEvent::KeyDown(PEvent::BACKSPACE)));
    // A press on a host's line opens its first room.
    assert!(press(&mut stage, &look, "venue", "90 members"));
    assert_eq!((desk.open.borrow().as_str(), part(&stage)), ("n_swan", vec!["title", "room", "status", "guide"]));
}

#[test]
fn step_26_i_make_a_host_of_my_own_from_a_template_and_remove_it() {
    let (app, mut stage, look) = staged(DESK);
    let desk = app.desk();
    assert!(stage.window.apply(&PDeed::Show("venue".to_string())));
    stage.lay(DESK, PMode::of_width(DESK.width));
    // Until it has a name a template makes nothing.
    assert!(!press(&mut stage, &look, "venue", "A club") && desk.made().is_empty());
    assert!(press(&mut stage, &look, "venue", desk.text.get("host.name")));
    typed(&mut stage, "Studio Nine");
    assert!(press(&mut stage, &look, "venue", "A club"));
    // Its rooms stand under their headings, and its first is opened.
    let made = desk.made();
    let room: Vec<(&str, &str, bool)> = made[0].plate.iter().map(|plate| (plate.label.as_str(), plate.head.as_str(), plate.voice)).collect();
    assert_eq!((made[0].label.as_str(), room), ("Studio Nine", vec![("welcome", "Text", false), ("news", "Text", false), ("Lounge", "Voice", true)]));
    let first = made[0].plate[0].id.clone();
    assert_eq!((desk.room(), part(&stage).contains(&"room")), (first.clone(), true));
    assert_eq!(rail(&app).last().map(String::as_str), Some("Studio Nine"));
    assert!(shown(&stage, &look).contains(&"welcome".to_string()));
    // It is kept as the web keeps it, and can be written in.
    let kept = desk.kept.get("made");
    assert_eq!((&kept["phiogy"][0]["group"]["label"], &kept["phiogy"][0]["room"][format!("{}-2", made[0].id)]["kind"]), (&json!("Studio Nine"), &json!("voice")));
    assert!(desk.write("First") && desk.said().len() == 1);
    // Removed, it is gone from the hosts and from the rail.
    assert!(stage.window.apply(&PDeed::Show("venue".to_string())));
    stage.lay(DESK, PMode::of_width(DESK.width));
    assert!(press(&mut stage, &look, "venue", desk.text.get("host.remove")));
    assert!(desk.made().is_empty() && !rail(&app).contains(&"Studio Nine".to_string()));
}

#[test]
fn step_27_friends_are_who_i_am_with_who_asked_and_whom_i_asked() {
    let (app, mut stage, look) = on("_friends");
    let desk = app.desk();
    assert!(part(&stage).contains(&"friend"));
    let word = |name: &str| desk.text.get(name).to_string();
    let tab = |name: &str, count: usize| format!("{} {count}", desk.text.get(name));
    // The direct list is led by friends, counting who asked, and by
    // the way to find anyone.
    let lead: Vec<(String, u64)> = desk.rail()[0].1.iter().take(2).map(|rung| (rung.id.clone(), rung.count)).collect();
    assert_eq!(lead, [("_friends".to_string(), 1), ("_people".to_string(), 0)]);
    // Here now: of my two friends the one who is about.
    let text = shown(&stage, &look);
    assert!([tab("friend.here", 1), tab("friend.all", 2), tab("friend.pending", 1), word("friend.add")].iter().all(|line| text.contains(line)), "{text:?}");
    assert!(press(&mut stage, &look, "friend", &tab("friend.all", 2)) && shown(&stage, &look).contains(&"Milo Brandt".to_string()));
    // Pending: who asked is accepted, and is a friend.
    assert!(press(&mut stage, &look, "friend", &tab("friend.pending", 1)));
    assert!(shown(&stage, &look).contains(&word("friend.asked")));
    let accept = icon(&stage, &look, "check").expect("the way to accept").center();
    assert!(stage.send(&PEvent::PointerDown(accept)));
    assert_eq!((desk.bond("tavi"), desk.bonded(ASKED).len(), desk.bonded(WITH).len()), (WITH.to_string(), 0, 3));
    assert!(shown(&stage, &look).contains(&word("friend.nothing")));
    // Add a friend: a search of everyone; asked, they wait.
    assert!(press(&mut stage, &look, "friend", &word("friend.add")));
    typed(&mut stage, "nina");
    let text = shown(&stage, &look);
    assert!(text.contains(&"Nina Valcourt".to_string()) && text.contains(&"@nina.v".to_string()) && !text.contains(&"Milo Brandt".to_string()), "{text:?}");
    let ask = icon(&stage, &look, "user-plus").expect("the way to ask").center();
    assert!(stage.send(&PEvent::PointerDown(ask)));
    assert_eq!(desk.bond("u_nina"), SENT);
    assert!(press(&mut stage, &look, "friend", &tab("friend.pending", 1)));
    let text = shown(&stage, &look);
    assert!(text.contains(&word("friend.yours")) && text.contains(&word("friend.waits")), "{text:?}");
    // Taken back, nothing is between us; what was said is kept.
    let back = icon(&stage, &look, "x").expect("the way to take it back").center();
    assert!(stage.send(&PEvent::PointerDown(back)));
    assert_eq!((desk.bond("u_nina"), desk.kept.get("friend")["phiogy"]["tavi"].clone()), (NONE.to_string(), json!("with")));
    // A press on a friend's line writes to them, in the room we have.
    assert!(press(&mut stage, &look, "friend", &tab("friend.all", 3)) && press(&mut stage, &look, "friend", "Lora Vale"));
    assert_eq!((desk.room(), part(&stage).contains(&"room")), ("lora".to_string(), true));
}

#[test]
fn step_28_finding_somebody_begins_a_conversation_that_stays_in_my_rail() {
    let (app, mut stage, look) = on("_people");
    let desk = app.desk();
    assert!(part(&stage).contains(&"people"));
    // Everyone but me, searched by name or handle.
    assert!(shown(&stage, &look).contains(&desk.text.fill("direct.count", "n", "4")));
    typed(&mut stage, "milo");
    let text = shown(&stage, &look);
    assert!(text.contains(&"@milo".to_string()) && !text.contains(&"@nina.v".to_string()), "{text:?}");
    // A press begins the conversation and goes to it: a room named
    // after them, which no file holds.
    assert!(press(&mut stage, &look, "people", "Milo Brandt"));
    assert_eq!((desk.open.borrow().as_str(), desk.begun()), ("dm-u_milo", vec!["u_milo".to_string()]));
    assert!(part(&stage).contains(&"room") && shown(&stage, &look).contains(&"Milo Brandt".to_string()));
    // It stands in the direct list from then on, and can be written in.
    let direct: Vec<String> = desk.rail()[0].1.iter().map(|rung| rung.id.clone()).collect();
    assert_eq!(direct, ["_friends", "_people", "lora", "notes", "dm-u_milo"]);
    assert!(desk.write("Hello Milo"));
    assert_eq!((desk.said().len(), desk.kept.get("sent")["phiogy/dm-u_milo"][0]["text"].clone()), (1, json!("Hello Milo")));
    // Somebody the site has a room with is written to there.
    assert_eq!(desk.direct_room("lora"), "lora");
}
