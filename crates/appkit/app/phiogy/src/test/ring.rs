//! A call somebody else began in a room: said in the room's title and
//! joined from there, by a window that serves the line and by one that
//! is on another's line; and a call begun with the camera.

#![cfg(not(target_family = "wasm"))]

use std::rc::Rc;
use std::sync::Arc;

use appkit::pui::PApp;
use appkit::pui::core::PMode;
use serde_json::json;

use super::exchange::{Page, line_on};
use super::peer::until;
use super::tools::{icon, tool};
use super::{DESK, press, shown, site};
use crate::Phiogy;
use crate::peer::Ring;

#[test]
fn step_39_a_call_somebody_began_is_said_in_the_title_and_joined_there() {
    let board = Phiogy::board();
    let line = line_on(Arc::clone(&board));
    let look = Rc::new(Phiogy::look(""));
    let app = Phiogy::holding(Rc::clone(&look), site(), Arc::clone(&board), Ok(line.at()), None);
    let mut stage = app.stage();
    stage.lay(DESK, PMode::of_width(DESK.width));
    let has = |stage: &appkit::pui::window::PStage, text: &str| shown(stage, &look).contains(&text.to_string());
    // Nobody is calling: a call can be begun, and nothing is said.
    assert!(icon(&stage, &look, "phone").is_some() && !has(&stage, "Guest is calling"));
    // A page begins a call in the room.
    let page = Page::open(&line.at(), "peer-aaaaaaaa", "phiogy/lora", "");
    assert_eq!(page.say(json!({ "join": true })).0, 200);
    until("the call is seen", || app.desk().ringing().is_some());
    // The title says who is calling, and has Join where the two tools
    // that begin a call were.
    assert!(has(&stage, "Guest is calling") && has(&stage, "Join"), "{:?}", shown(&stage, &look));
    assert!(icon(&stage, &look, "phone").is_none() && icon(&stage, &look, "video").is_none());
    assert!(press(&mut stage, &look, "Join"));
    until("this window is in the call", || app.desk().calling().is_some_and(|seen| seen.joined));
    stage.lay(DESK, PMode::of_width(DESK.width));
    // In it, the title says so no more, and the call has its buttons.
    assert!(!has(&stage, "Guest is calling") && !has(&stage, "Join") && has(&stage, "End the call"));
    // The other caller has a tile, by the name the line knows them by.
    let tile = app.desk().tile(&appkit::service::PTable::read(crate::data::PEER), 0.0);
    assert!(tile.iter().any(|tile| tile.name == "Guest" && tile.said == "In the call"), "{tile:?}");
    page.close();
}

#[test]
fn a_call_begun_with_the_camera_has_it_on_from_the_start() {
    let board = Phiogy::board();
    let line = line_on(Arc::clone(&board));
    let look = Rc::new(Phiogy::look(""));
    let app = Phiogy::holding(Rc::clone(&look), site(), Arc::clone(&board), Ok(line.at()), None);
    let mut stage = app.stage();
    stage.lay(DESK, PMode::of_width(DESK.width));
    assert!(tool(&mut stage, &look, "video"));
    until("it is in the call with its camera on", || app.desk().calling().is_some_and(|seen| seen.joined && seen.own.camera));
    stage.lay(DESK, PMode::of_width(DESK.width));
    assert!(shown(&stage, &look).contains(&"Camera off".to_string()));
}

#[test]
fn a_room_s_line_is_listened_to_from_outside_its_call() {
    let line = line_on(Phiogy::board());
    let ring = Ring::open(&line.at(), "phiogy/lora", "phiogy-listening");
    assert!(ring.caller().is_empty());
    let page = Page::open(&line.at(), "peer-aaaaaaaa", "phiogy/lora", "");
    page.say(json!({ "join": true }));
    until("the ring hears who is calling", || ring.caller() == [("peer-aaaaaaaa".to_string(), "Guest".to_string())]);
    page.say(json!({ "join": false }));
    until("and that they left", || ring.caller().is_empty());
    page.close();
}

#[test]
fn step_40_a_window_on_another_s_line_sees_a_call_there_and_joins_it() {
    // The line is another's: this window's own board never hears of it.
    let line = line_on(Phiogy::board());
    let look = Rc::new(Phiogy::look(""));
    let app = Phiogy::holding(Rc::clone(&look), site(), Phiogy::board(), Ok(line.at()), None).away();
    let mut stage = app.stage();
    stage.lay(DESK, PMode::of_width(DESK.width));
    let has = |stage: &appkit::pui::window::PStage, text: &str| shown(stage, &look).contains(&text.to_string());
    let page = Page::open(&line.at(), "peer-aaaaaaaa", "phiogy/lora", "");
    page.say(json!({ "join": true }));
    // The window listens to the open room's line while it is shown.
    stage.tick();
    assert!(app.desk().ring.borrow().is_some());
    until("the call is seen", || app.desk().ringing().is_some());
    assert!(has(&stage, "Guest is calling") && has(&stage, "Join"), "{:?}", shown(&stage, &look));
    assert!(press(&mut stage, &look, "Join"));
    until("this window is in the call", || app.desk().calling().is_some_and(|seen| seen.joined && !seen.named.is_empty()));
    // In the call it listens no more: the call holds the line.
    stage.tick();
    assert!(app.desk().ring.borrow().is_none());
    stage.lay(DESK, PMode::of_width(DESK.width));
    // Who else is in it is named as the line names them.
    let tile = app.desk().tile(&appkit::service::PTable::read(crate::data::PEER), 0.0);
    assert!(tile.iter().any(|tile| tile.name == "Guest"), "{tile:?}");
    assert!(has(&stage, "End the call"));
    page.close();
}
