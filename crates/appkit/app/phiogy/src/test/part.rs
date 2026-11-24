//! The parts, each for what it shows.


use super::{AT, DESK, HAND, joining, opened, press, shown, staged};
use crate::board::lock;
use crate::comp::Limit;
use crate::system::{leave, post};

#[test]
fn the_title_names_the_room_and_says_its_topic() {
    let (app, stage, look) = staged(DESK);
    assert!(app.desk().show("lobby"));
    let text = shown(&stage, &look);
    // Before its name the host it is a room of; after it its topic, cut
    // where the search field needs the room.
    assert!(text.contains(&"lobby".to_string()) && text.contains(&"Phiace".to_string()) && text.iter().any(|said| said.starts_with("Say hello, tell us")), "{text:?}");
}

#[test]
fn a_room_with_nobody_on_its_line_says_so_and_where_a_page_reaches_it() {
    let (app, stage, look) = staged(DESK);
    let text = shown(&stage, &look).join(" ");
    assert!(text.contains(app.desk().text.get("present.none")), "{text}");
    assert!(shown(&stage, &look).concat().contains(&format!("http://{AT}/call/phiogy/lora")), "{text}");
}

#[test]
fn who_is_on_the_line_is_shown_as_the_board_has_them_now() {
    let (app, mut stage, look) = staged(DESK);
    let word = |name: &str| app.desk().text.get(name).to_string();
    // A page opens the room: it has the room open, and is not in a call.
    let a = opened(&mut lock(&app.desk().board), "phiogy/lora", "peer-aaaaaaaa", 1);
    // The line is open: nothing moves by itself, so no frame is asked
    // for at once, but the window is looked at again on the theme's beat.
    assert!(!stage.tick() && stage.beat());
    let text = shown(&stage, &look);
    assert!(text.contains(&word("present.watch")) && !text.contains(&word("present.call")), "{text:?}");
    assert!(text.contains(&"Guest".to_string()) && text.contains(&"peer-aaaaaaaa".to_string()));
    assert!(!text.contains(&word("present.none")));
    // It joins, and another page opens the room.
    post(&mut lock(&app.desk().board), "phiogy/lora", "guest_1", &joining("peer-aaaaaaaa", true)).unwrap();
    opened(&mut lock(&app.desk().board), "phiogy/lora", "peer-bbbbbbbb", 2);
    let text = shown(&stage, &look);
    let at = |wanted: &str| text.iter().position(|line| line == wanted).unwrap_or_else(|| panic!("{wanted} in {text:?}"));
    // The callers first, then who only has the room open.
    assert!(at(&word("present.call")) < at("peer-aaaaaaaa") && at("peer-aaaaaaaa") < at(&word("present.watch")) && at(&word("present.watch")) < at("peer-bbbbbbbb"));
    // A page on another room's line is not shown in this one.
    opened(&mut lock(&app.desk().board), "phiogy/lobby", "peer-cccccccc", 3);
    assert!(!shown(&stage, &look).contains(&"peer-cccccccc".to_string()));
    // The caller's page goes away.
    leave::close(&mut lock(&app.desk().board), a);
    let text = shown(&stage, &look);
    assert!(!text.contains(&"peer-aaaaaaaa".to_string()) && !text.contains(&word("present.call")), "{text:?}");
}

#[test]
fn a_room_s_own_people_are_named_under_who_is_on_its_line() {
    let (app, stage, look) = staged(DESK);
    assert!(app.desk().show("lobby"));
    let text = shown(&stage, &look);
    assert!(text.contains(&app.desk().text.get("present.member").to_string()));
    // Each is a card with their face and their name, not a line of names.
    assert!(["You", "Lora Vale", "Tavi Reyes"].iter().all(|name| text.contains(&name.to_string())), "{text:?}");
    assert!(!text.iter().any(|line| line.contains("Lora Vale") && line.contains("Tavi Reyes")), "{text:?}");
    let faces = super::drawn(&stage, &look).iter().filter(|mark| matches!(mark, appkit::pui::core::PMark::Picture { rect, .. } if rect.size.width == app.desk().size(appkit::pui::block::FACE))).count();
    assert!(faces >= 2, "{faces}");
}

#[test]
fn the_status_says_where_the_line_is_and_what_it_holds() {
    let (app, stage, look) = staged(DESK);
    let (text, limit) = (shown(&stage, &look), Limit::own());
    assert!(text.contains(&format!("The line is open at {AT}")), "{text:?}");
    // The limits are the table's.
    assert!(text.contains(&format!("{} in a call · {} pages on a line · a beat every {} s", limit.caller, limit.watch, limit.beat / 1000)), "{text:?}");
    // In a hand there is room for where it is, and the limits are left out.
    let (_, stage, look) = staged(HAND);
    let text = shown(&stage, &look);
    assert!(text.contains(&format!("The line is open at {AT}")) && !text.iter().any(|line| line.contains("in a call")), "{text:?}");
    drop(app);
}

#[test]
fn the_guide_lists_the_places_i_joined_with_their_rooms_and_opens_one() {
    let (app, mut stage, look) = staged(DESK);
    // One place at a time, the one the rail chose: a heading with its
    // rooms under it; one I did not join is never there.
    let at = |text: &Vec<String>, wanted: &str| text.iter().position(|line| line == wanted).unwrap_or_else(|| panic!("{wanted} in {text:?}"));
    assert!(app.desk().visit("phiace"));
    let text = shown(&stage, &look);
    assert!(at(&text, "Phiace") < at(&text, "Lounge") && !text.contains(&"Opal Atelier".to_string()));
    assert!(app.desk().visit("h_opal"));
    let text = shown(&stage, &look);
    assert!(at(&text, "Opal Atelier") < at(&text, "front-row") && !text.contains(&"The Swan Salon".to_string()));
    assert!(app.desk().visit("phiace"));
    // A room with pages on its line says how many.
    opened(&mut lock(&app.desk().board), "phiogy/lounge", "peer-aaaaaaaa", 1);
    opened(&mut lock(&app.desk().board), "phiogy/lounge", "peer-bbbbbbbb", 2);
    assert!(shown(&stage, &look).contains(&"2".to_string()));
    // A press on a room opens it; one on a heading does nothing.
    assert!(press(&mut stage, &look, "Lounge"));
    assert_eq!(app.desk().open.borrow().as_str(), "lounge");
    assert!(!press(&mut stage, &look, "Phiace"));
    assert_eq!(app.desk().open.borrow().as_str(), "lounge");
}

// The hosts' page has its tests in `hosts.rs`.
