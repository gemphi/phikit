//! A call with pictures: the tiles a room shows of it, the camera and
//! the screen turned on and off from the window, and what the mesh
//! keeps of the pictures another caller sends.

use std::sync::Arc;

use serde_json::{Value, json};

use super::peer::until;
use crate::peer::{Engine, Kind, Link, Made, Mesh};

#[cfg(not(target_family = "wasm"))]
#[test]
fn step_38_in_a_call_i_see_its_tiles_and_turn_my_camera_and_a_screen_on_and_off() {
    use std::rc::Rc;

    use appkit::pui::PApp;
    use appkit::pui::core::PMode;

    use super::{DESK, press, shown, site};
    use crate::Phiogy;
    use crate::part::call::Shown;
    let board = Phiogy::board();
    let line = super::exchange::line_on(Arc::clone(&board));
    let look = Rc::new(Phiogy::look(""));
    let app = Phiogy::holding(Rc::clone(&look), site(), Arc::clone(&board), Ok(line.at()), None);
    let mut stage = app.stage();
    stage.lay(DESK, PMode::of_width(DESK.width));
    let has = |stage: &appkit::pui::window::PStage, text: &str| shown(stage, &look).contains(&text.to_string());
    // Out of a call the room shows none.
    assert!(!has(&stage, "Camera on") && app.desk().tile(&appkit::service::PTable::read(crate::data::PEER), 0.0).is_empty());
    assert!(press(&mut stage, &look, "Join"));
    until("this window is in the call", || app.desk().calling().is_some_and(|seen| seen.joined));
    stage.lay(DESK, PMode::of_width(DESK.width));
    // The call stands over the stream: the room called, this window's
    // own tile, the person of the room ringing, and what the call does.
    let text = shown(&stage, &look);
    let wanted = ["Calling Lora Vale", "You", "Ringing", "Camera on", "Share a screen"];
    assert!(wanted.iter().all(|line| text.contains(&line.to_string())), "{text:?}");
    // What the call does is said whole, broken over as many lines as it needs.
    assert!(text.join(" ").contains("Your microphone, camera and screen go to everyone who joins this call."), "{text:?}");
    let table = appkit::service::PTable::read(crate::data::PEER);
    let tile = app.desk().tile(&table, 0.0);
    assert_eq!(tile.len(), 2, "{tile:?}");
    assert!(matches!(&tile[0].shown, Shown::Face(id, _) if *id == app.desk().site.me) && tile[1].said == "Ringing");
    // The camera on: the tile is the film's picture of this window's
    // own, and the pill offers the way back.
    assert!(press(&mut stage, &look, "Camera on"));
    until("the camera is on", || app.desk().calling().is_some_and(|seen| seen.own.camera));
    // The window lays itself out for every frame: the call has grown.
    stage.lay(DESK, PMode::of_width(DESK.width));
    assert!(has(&stage, "Camera off") && !has(&stage, "Camera on"));
    assert_eq!(app.desk().tile(&table, 0.0)[0].shown, Shown::Film("call/own/main".to_string()));
    // A screen shared: a tile of its own, after this window's camera.
    assert!(press(&mut stage, &look, "Share a screen"));
    until("a screen is shared", || app.desk().calling().is_some_and(|seen| seen.own.screen));
    stage.lay(DESK, PMode::of_width(DESK.width));
    let tile = app.desk().tile(&table, 0.0);
    assert!(tile[1].scene && tile[1].shown == Shown::Film("call/own/screen".to_string()), "{tile:?}");
    assert!(has(&stage, "Stop sharing"));
    // Both off again; muted, the tile says so.
    assert!(press(&mut stage, &look, "Stop sharing"));
    until("the screen is not shared", || app.desk().calling().is_some_and(|seen| !seen.own.screen));
    stage.lay(DESK, PMode::of_width(DESK.width));
    assert!(press(&mut stage, &look, "Camera off"));
    until("both are off", || app.desk().calling().is_some_and(|seen| !seen.own.camera && !seen.own.screen));
    assert!(press(&mut stage, &look, "Mute"));
    until("it is muted", || app.desk().calling().is_some_and(|seen| seen.quiet));
    assert_eq!(app.desk().tile(&table, 0.0)[0].said, "Muted");
    // Leave: the call is gone from the room.
    assert!(press(&mut stage, &look, "Leave"));
    stage.lay(DESK, PMode::of_width(DESK.width));
    assert!(!has(&stage, "Camera on") && !has(&stage, "Calling Lora Vale"));
}

#[cfg(not(target_family = "wasm"))]
#[test]
fn the_tiles_of_a_call_fold_where_the_room_is_narrow() {
    use std::rc::Rc;

    use super::site;
    use crate::Phiogy;
    use crate::part::Calling;
    use crate::part::call::{Shown, Tile};
    let app = Phiogy::holding(Rc::new(Phiogy::look("")), site(), Phiogy::board(), Err(String::new()), None);
    let calling = Calling::new(app.desk().clone());
    let film = |scene: bool| Tile { shown: Shown::Film("call/a/main".to_string()), scene, name: "Ada".to_string(), said: "In the call".to_string(), loud: false };
    let face = Tile { shown: Shown::Face(String::new(), "You".to_string()), scene: false, name: String::new(), said: "You".to_string(), loud: false };
    let tile = [face, film(false), film(true)];
    // At a desk they stand in one row: a face, a camera at four to
    // three, a screen at sixteen to nine and wider.
    let (rect, high) = calling.lay(&tile, 900.0, 1.0);
    assert!(rect.iter().all(|rect| rect.origin.y == 0.0), "{rect:?}");
    assert_eq!((rect[1].size.width, rect[1].size.height, rect[2].size.width, rect[2].size.height), (240.0, 180.0, 384.0, 216.0));
    assert!(rect[0].origin.x < rect[1].origin.x && rect[1].origin.x < rect[2].origin.x);
    // In a hand each folds under the last, none wider than the room.
    let (narrow, tall) = calling.lay(&tile, 300.0, 1.0);
    assert!(narrow[1].origin.y > narrow[0].origin.y && narrow[2].origin.y > narrow[1].origin.y, "{narrow:?}");
    assert!(narrow.iter().all(|rect| rect.origin.x + rect.size.width <= 300.0) && tall > high);
    // Where the room is low the tiles are made smaller, each by the same.
    let (small, low) = calling.lay(&tile, 900.0, 0.5);
    assert_eq!((small[1].size.width, small[1].size.height, small[2].size.width), (120.0, 90.0, 192.0));
    assert!(low < high);
    // With no call there is nothing to stand over the stream.
    assert_eq!(calling.high(900.0, 600.0), 0.0);
}

/// A link that makes what a test gives it.
struct Given(Vec<Made>);

impl Link for Given {
    fn offer(&mut self) {}

    fn accept(&mut self, _kind: Kind, _sdp: &str) {}

    fn candidate(&mut self, _body: &Value) {}

    fn take(&mut self) -> Vec<Made> {
        std::mem::take(&mut self.0)
    }

    fn close(&mut self) {}
}

/// An engine whose first link makes those things.
struct Giving(Vec<Made>);

impl Engine for Giving {
    fn link(&mut self, _far: &str, _ice: &[Value]) -> Box<dyn Link> {
        Box::new(Given(std::mem::take(&mut self.0)))
    }

    fn name(&self) -> &str {
        "giving"
    }
}

#[test]
fn a_caller_s_pictures_are_kept_by_the_stream_each_comes_under() {
    let mut mesh = Mesh::new("peer-zzzz");
    let seen = |stream: &str, count: u64, width: u32| Made::Seen(stream.to_string(), count, width, width * 3 / 4);
    let mut engine = Giving(vec![seen("cam", 1, 640), seen("shared", 1, 1920), seen("cam", 2, 640)]);
    let roster = ["peer-zzzz".to_string(), "peer-aaaa".to_string()];
    // This caller says under which stream its own screen goes.
    let told = mesh.sync(&roster, &mut engine);
    assert_eq!(told, [("peer-aaaa".to_string(), json!({ "state": { "quiet": false, "screen": "screen" } }))]);
    mesh.turn();
    // Not told which stream is a screen, every picture is a camera's.
    let far = mesh.far().remove(0);
    assert_eq!((far.picture.len(), far.seen, far.camera().map(|picture| picture.stream.as_str()), far.scene()), (2, 3, Some("cam"), None));
    // Told, the screen is told from the camera.
    mesh.hear("peer-aaaa", &json!({ "state": { "quiet": true, "screen": "shared" } }));
    let far = mesh.far().remove(0);
    assert!(far.quiet);
    assert_eq!((far.camera().map(|picture| (picture.stream.as_str(), picture.seen)), far.scene().map(|picture| picture.size)), (Some(("cam", 2)), Some((1920, 1440))));
}

#[test]
fn a_stream_that_brings_no_more_pictures_is_over() {
    let mut mesh = Mesh::new("peer-zzzz");
    let mut engine = Giving(vec![Made::Seen("cam".to_string(), 1, 640, 480), Made::Gone("cam".to_string())]);
    mesh.sync(&["peer-zzzz".to_string(), "peer-aaaa".to_string()], &mut engine);
    mesh.turn();
    let far = mesh.far().remove(0);
    // The camera is off: no picture arrives, though one was counted.
    assert!(far.picture.is_empty() && far.camera().is_none() && far.seen == 1);
}

#[test]
fn a_picture_of_a_call_is_named_by_whose_it_is_and_its_stream() {
    let table = appkit::service::PTable::read(crate::data::PEER);
    assert_eq!(crate::peer::film(&table, "peer-aaaa", "cam"), "call/peer-aaaa/cam");
    assert_eq!(crate::peer::film(&table, table.get("film.own"), table.get("scene")), "call/own/screen");
    assert_eq!(crate::peer::shown("live/", "a", "b"), "live/a/b");
}
