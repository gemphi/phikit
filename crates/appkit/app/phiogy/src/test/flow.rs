//! The steps of `FLOW.md`: what a person does in Phiogy, each a test.

use std::rc::Rc;

use appkit::pui::PApp;
use appkit::pui::core::{PEvent, PMode};
use appkit::service::PStore;

use super::{AT, DESK, HAND, icon, joining, opened, part, pictured, press, row, shown, site, staged};
use crate::board::lock;
use crate::system::post;
use crate::{Phiogy, fact};

#[test]
fn step_1_it_opens_on_the_room_i_begin_in_with_my_rooms_beside_it() {
    let (app, stage, look) = staged(DESK);
    assert_eq!(app.desk().open.borrow().as_str(), "lora");
    assert_eq!(part(&stage), ["title", "room", "status", "guide"]);
    let text = shown(&stage, &look);
    // The room over the body, and beside the bar the places I joined.
    assert!(["Lora Vale", "On the line", "Start", "Hosts", "Direct", "Phiace", "lobby", "Lounge", "Opal Atelier", "front-row"].iter().all(|line| text.contains(&line.to_string())), "{text:?}");
}

#[test]
fn step_3_a_press_on_a_room_in_the_bar_opens_it() {
    let (app, mut stage, look) = staged(DESK);
    assert!(row(&mut stage, &look, "lobby"));
    assert_eq!(app.desk().open.borrow().as_str(), "lobby");
    let text = shown(&stage, &look);
    assert!(text.contains(&"Say hello, tell us what you are building.".to_string()), "{text:?}");
    // Its line is the room's own.
    assert!(text.concat().contains(&format!("http://{AT}/call/phiogy/lobby")));
}

#[test]
fn step_4_the_hosts_row_lists_every_place_and_a_press_on_one_opens_its_first_room() {
    let (app, mut stage, look) = staged(DESK);
    assert!(icon(&mut stage, &look, "globe"));
    assert_eq!(part(&stage), ["venue", "guide"]);
    assert!(press(&mut stage, &look, "The Swan Salon"));
    assert_eq!((app.desk().open.borrow().as_str(), part(&stage)), ("n_swan", vec!["title", "room", "status", "guide"]));
}

#[test]
fn step_5_start_brings_me_back_to_the_room_i_begin_in() {
    let (app, mut stage, look) = staged(DESK);
    assert!(row(&mut stage, &look, "lobby") && icon(&mut stage, &look, "globe"));
    assert!(row(&mut stage, &look, "Start"));
    assert_eq!((app.desk().open.borrow().as_str(), part(&stage)), ("lora", vec!["title", "room", "status", "guide"]));
}

#[test]
fn step_6_a_look_is_chosen_in_the_menu_and_phiogy_opens_the_way_i_left_it() {
    let root = std::env::temp_dir().join("phiogy-test-choice");
    std::fs::remove_dir_all(&root).ok();
    let kept = PStore::at(&root.to_string_lossy());
    let open = |store: Option<PStore>| Phiogy::holding(Rc::new(Phiogy::look("")), site(), Phiogy::board(), Ok(AT.to_string()), store);
    let left = |app: &Phiogy| (app.desk().shade.name().to_string(), app.desk().open.borrow().clone());
    let app = open(Some(kept.clone()));
    assert_eq!(left(&app), ("light".to_string(), "lora".to_string()));
    let mut stage = app.stage();
    stage.lay(DESK, PMode::of_width(DESK.width));
    let look = Rc::new(Phiogy::look(""));
    // The Menu row holds the looks; a room is opened from the Rooms row.
    assert!(icon(&mut stage, &look, "layout-grid") && row(&mut stage, &look, "Dark"));
    assert!(icon(&mut stage, &look, "chat") && row(&mut stage, &look, "lobby"));
    assert_eq!(left(&app), ("dark".to_string(), "lobby".to_string()));
    // A look there is not, and a room there is not, change nothing.
    assert!(!app.desk().dress("dusk") && !app.desk().show("nowhere"));
    assert_eq!(left(&open(Some(kept))), ("dark".to_string(), "lobby".to_string()));
    // Opened with nowhere to keep things, it is as it ships.
    assert_eq!(left(&open(None)), ("light".to_string(), "lora".to_string()));
}

#[test]
fn step_7_my_profile_says_where_the_line_is_and_how_many_are_on_it() {
    let (app, mut stage, look) = staged(DESK);
    opened(&mut lock(&app.desk().board), "phiogy/lora", "peer-aaaaaaaa", 1);
    opened(&mut lock(&app.desk().board), "phiogy/lobby", "peer-bbbbbbbb", 2);
    post(&mut lock(&app.desk().board), "phiogy/lobby", "guest_2", &joining("peer-bbbbbbbb", true)).unwrap();
    let said = fact::fact(app.desk());
    assert_eq!((said.get("Line"), said.get("Rooms open"), said.get("In calls"), said.get("Hosts"), said.get("Look")), (AT, "2", "1", "4", "Light"));
    assert_eq!(said.get("Kept in"), app.desk().text.get("fact.nowhere"));
    // Where its hosts and rooms were read from: here, nobody was asked.
    assert_eq!(said.get("Read from"), app.desk().text.get("fact.file"));
    // The profile is a row of the Menu, and shows them.
    assert!(icon(&mut stage, &look, "layout-grid") && row(&mut stage, &look, "Profile"));
    assert_eq!(stage.window.modal(), Some("profile"));
    stage.tick();
    let text = shown(&stage, &look);
    assert!(["Guest", "About", "Line", AT, "Rooms open", "Appearance"].iter().all(|line| text.contains(&line.to_string())), "{text:?}");
    assert!(stage.send(&PEvent::KeyDown(PEvent::ESCAPE)));
    assert_eq!(stage.window.modal(), None);
}

#[test]
fn step_8_in_a_hand_the_room_is_alone_and_the_rooms_row_draws_my_rooms_out() {
    let (app, mut stage, look) = staged(HAND);
    assert_eq!(part(&stage), ["title", "room", "status"]);
    assert!(icon(&mut stage, &look, "chat"));
    assert_eq!(part(&stage), ["guide"]);
    assert!(press(&mut stage, &look, "lobby"));
    assert_eq!((app.desk().open.borrow().as_str(), part(&stage)), ("lobby", vec!["title", "room", "status"]));
}

#[test]
fn step_9_phiogy_s_mark_is_in_the_caption_over_the_bar_and_is_the_pin() {
    use appkit::pui::core::PMark;
    let (app, mut stage, look) = staged(DESK);
    let (mark, room) = (pictured(&stage, &look, "phiogy").expect("the mark"), pictured(&stage, &look, "chat").expect("the Rooms row"));
    assert!(stage.frame().caption.contains(mark) && mark.y < room.y && mark.x == room.x, "{mark:?} {room:?}");
    // The mark and every icon of the bar under it stand on grounds of
    // one size, and are drawn as large as each other.
    let ground = |stage: &appkit::pui::window::PStage, name: &str| {
        let mark = super::drawn(stage, &look);
        mark.windows(2).find_map(|pair| match (&pair[0], &pair[1]) {
            (PMark::Blend { rect, .. }, PMark::Icon { name: known, rect: icon, .. }) if known == name => Some((rect.size.height, icon.size)),
            _ => None,
        })
    };
    let (own, row) = (ground(&stage, "phiogy").expect("the mark's ground"), ground(&stage, "chat").expect("a row's ground"));
    assert_eq!(own, row);
    assert!(own.0 > own.1.height);
    // Its name is beside it, in one case, while the menu is shown.
    let named = |stage: &appkit::pui::window::PStage| shown(stage, &look).contains(&"phiogy".to_string());
    assert!(named(&stage) && !shown(&stage, &look).contains(&"Phiogy".to_string()));
    // The mark is the pin: no other pin stands in the caption, and
    // with the pointer over it, it shows the pin.
    assert!(pictured(&stage, &look, "pin-open").is_none() && pictured(&stage, &look, "pin-shut").is_none());
    assert!(stage.send(&PEvent::PointerMove(mark)));
    assert_eq!(pictured(&stage, &look, "pin-open"), Some(mark));
    assert!(pictured(&stage, &look, "phiogy").is_none());
    // A press on it puts the menu away, and the name with it; another
    // brings both back.
    assert!(stage.send(&PEvent::PointerDown(mark)));
    assert_eq!(stage.frame().menu.size.width, 0.0);
    assert!(!named(&stage) && pictured(&stage, &look, "pin-shut") == Some(mark));
    assert!(stage.send(&PEvent::PointerDown(mark)));
    assert_eq!(stage.frame().menu.size.width, 200.0);
    assert!(named(&stage));
    // The pointer gone from it, it is the mark again.
    assert!(stage.send(&PEvent::PointerMove(room)));
    assert_eq!(pictured(&stage, &look, "phiogy"), Some(mark));
    // A row of the bar still goes where it says.
    assert!(icon(&mut stage, &look, "globe"));
    assert!(part(&stage).contains(&"venue") && !part(&stage).contains(&"room"));
    // The mark is a file of Phiogy's own, drawn on the font.
    let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("asset/icon/phiogy.svg");
    assert!(std::fs::read_to_string(file).is_ok_and(|text| text.contains("<path")));
    assert_eq!(app.name(), "Phiogy");
}

#[cfg(not(target_family = "wasm"))]
#[test]
fn step_2_a_page_that_opens_a_room_over_the_line_is_in_the_window_at_once() {
    use super::exchange::{Page, line_on};
    use serde_json::json;
    let (app, stage, look) = staged(DESK);
    // The line is served on the board the window reads.
    let line = line_on(std::sync::Arc::clone(&app.desk().board));
    let at = line.at();
    let mut page = Page::open(&at, "peer-aaaaaaaa", "phiogy/lora", "");
    assert!(page.next().is_some());
    let text = shown(&stage, &look);
    assert!(text.contains(&"Has the room open".to_string()) && text.contains(&"peer-aaaaaaaa".to_string()), "{text:?}");
    // It joins the call, and is told so on its line.
    assert_eq!(page.say(json!({ "join": true })).0, 200);
    assert_eq!(page.next().map(|event| event["roster"].as_array().map(Vec::len)), Some(Some(1)));
    assert!(shown(&stage, &look).contains(&"In the call".to_string()));
    // The page goes away; the line notices, and the window with it.
    page.close();
    let gone = |_| {
        std::thread::sleep(std::time::Duration::from_millis(20));
        lock(&app.desk().board).open().is_empty()
    };
    assert!((0..250).any(gone), "the page was not dropped");
    assert!(shown(&stage, &look).contains(&"Nobody has this room open.".to_string()));
}

#[test]
fn step_44_the_hosts_i_am_in_stand_in_the_bar_each_with_its_mark() {
    use appkit::pui::core::PMark;
    let (app, mut stage, look) = staged(DESK);
    // After the bar's own rows, one for each host I am in, in the
    // site's order; the direct list is no host.
    let place = app.desk().place();
    assert_eq!(place.iter().map(|row| row.entry.label.as_str()).collect::<Vec<_>>(), ["Phiace", "Opal Atelier"]);
    assert!(place.iter().all(|row| !row.on), "{place:?}");
    // Each stands in the bar on a ground as the icons do, under them.
    let bar = stage.frame().activity;
    let rows = |stage: &appkit::pui::window::PStage| {
        let mark = super::drawn(stage, &look);
        mark.iter().filter(|mark| matches!(mark, PMark::Blend { rect, .. } if bar.contains(rect.center()) && rect.size.height < bar.size.width)).count()
    };
    let chat = pictured(&stage, &look, "chat").expect("the Rooms row");
    let all = rows(&stage);
    assert!(all >= 3 + 1 + 2, "the three rows that go somewhere, Menu, and two hosts: {all}");
    // A press on a host's row goes to its first room, and marks it.
    let row = stage.theme().size("activity.row").unwrap_or_default();
    let first = appkit::pui::core::PPoint::new(chat.x, chat.y + row * 3.0);
    assert!(stage.send(&PEvent::PointerDown(first)));
    assert_eq!(app.desk().open.borrow().as_str(), "lobby");
    assert_eq!(app.desk().place().iter().map(|row| row.on).collect::<Vec<_>>(), [true, false]);
    // Left, a host has no row any more: the bar is asked again.
    let opal = app.desk().site.venue.iter().find(|venue| venue.label == "Opal Atelier").map(|venue| venue.id.clone()).expect("the host");
    app.desk().enter(&opal, false);
    assert!(stage.tick());
    assert_eq!(rows(&stage), all - 1);
    assert_eq!(app.desk().place().len(), 1);
}
