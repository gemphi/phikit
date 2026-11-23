//! A voice room: who is in it, the way in and out, the microphone and
//! the sound of the others, on the window's own stage over a real line.

#![cfg(not(target_family = "wasm"))]

use std::rc::Rc;
use std::sync::Arc;

use appkit::pui::PApp;
use appkit::pui::core::{PEvent, PMark, PMode, PPoint};
use appkit::pui::window::PStage;
use appkit::theme::PLook;
use serde_json::json;

use super::exchange::{Page, line_on};
use super::peer::until;
use super::tools::icon;
use super::{DESK, drawn, press, shown, site};
use crate::Phiogy;

/// Where the icons of that name stand at the foot of the bar's menu,
/// from the top and the left: those no further over its lowest icon
/// than the foot of the guide is high, which has a row and its controls.
fn foot(stage: &PStage, look: &PLook, wanted: &str) -> Vec<PPoint> {
    use appkit::theme::PTheme;
    let high = look.size("row.height").unwrap_or_default() + look.size("text.line").unwrap_or_default();
    let menu = stage.frame().menu;
    let at = |mark: PMark| match mark {
        PMark::Icon { name, rect, .. } if menu.contains(rect.center()) => Some((name.to_string(), rect.center())),
        _ => None,
    };
    let all: Vec<(String, PPoint)> = drawn(stage, look).into_iter().filter_map(at).collect();
    let low = all.iter().map(|(_, at)| at.y).fold(0.0, f32::max);
    let mut found: Vec<PPoint> = all.into_iter().filter(|(name, at)| name == wanted && low - at.y < high).map(|(_, at)| at).collect();
    found.sort_by(|one, other| (one.y, one.x).partial_cmp(&(other.y, other.x)).unwrap_or(std::cmp::Ordering::Equal));
    found
}

#[test]
fn step_41_i_am_in_a_voice_room_mute_turn_the_sound_off_and_leave() {
    let board = Phiogy::board();
    let line = line_on(Arc::clone(&board));
    let look = Rc::new(Phiogy::look(""));
    let app = Phiogy::holding(Rc::clone(&look), site(), Arc::clone(&board), Ok(line.at()), None);
    let mut stage = app.stage();
    assert!(app.desk().show("lounge"));
    stage.lay(DESK, PMode::of_width(DESK.width));
    let has = |stage: &appkit::pui::window::PStage, text: &str| shown(stage, &look).contains(&text.to_string());
    let all = |stage: &appkit::pui::window::PStage| shown(stage, &look).join(" ");
    // A voice room has no call to begin in its title: its way in is on
    // its page, with who the file says is in it and how to begin.
    assert!(icon(&stage, &look, "phone").is_none() && has(&stage, "Join") && !has(&stage, "Leave"));
    assert!(all(&stage).contains("Join to talk. Your microphone is opened then."), "{}", all(&stage));
    let seat = |app: &Phiogy| crate::part::Voice::new(app.desk().clone()).seat();
    assert_eq!(seat(&app).iter().map(|seat| seat.id.as_str()).collect::<Vec<_>>(), ["tavi"]);
    // Somebody is in it on its line: they are there before this window joins.
    let page = Page::open(&line.at(), "peer-aaaaaaaa", "phiogy/lounge", "");
    assert_eq!(page.say(json!({ "join": true })).0, 200);
    until("they are seen in the room", || seat(&app).iter().any(|seat| seat.name == "Guest" && seat.said == "In the room"));
    // Nobody is rung into a voice room.
    assert!(app.desk().ringing().is_none() && !has(&stage, "Guest is calling"));
    // In: this window's person is among them, and the room has Leave,
    // the microphone and the sound of the others.
    assert!(press(&mut stage, &look, "Join"));
    until("this window is in the room", || app.desk().calling().is_some_and(|seen| seen.joined));
    assert!(seat(&app).iter().any(|seat| seat.id == app.desk().site.me), "{:?}", seat(&app));
    assert!(["Leave", "Mute", "Sound off"].iter().all(|text| has(&stage, text)) && !has(&stage, "Join"));
    assert!(all(&stage).contains("Your microphone goes to everyone in this room, and you hear them."), "{}", all(&stage));
    // It has no tiles of a call, and no camera.
    assert!(!has(&stage, "Camera on") && !has(&stage, "Calling Lounge"));
    // Muted, the person's own card says so; the sound off, the pill
    // offers it on again.
    assert!(press(&mut stage, &look, "Mute"));
    until("it is muted", || app.desk().calling().is_some_and(|seen| seen.quiet));
    assert!(has(&stage, "Unmute") && seat(&app).iter().any(|seat| seat.id == app.desk().site.me && seat.said == "Muted"));
    assert!(press(&mut stage, &look, "Sound off"));
    until("it hears nobody", || app.desk().calling().is_some_and(|seen| seen.own.deaf));
    assert!(has(&stage, "Sound on") && !has(&stage, "Sound off"));
    // It goes on while another room is looked at, and is found again.
    assert!(app.desk().show("lobby"));
    stage.lay(DESK, PMode::of_width(DESK.width));
    assert!(app.desk().call.borrow().is_some() && !has(&stage, "Leave"));
    assert!(app.desk().show("lounge"));
    stage.lay(DESK, PMode::of_width(DESK.width));
    // Leave: the line lets it go, and the way in is back.
    assert!(press(&mut stage, &look, "Leave"));
    until("the line let it go", || super::peer::calling(&board, "phiogy/lounge") == ["peer-aaaaaaaa"]);
    assert!(has(&stage, "Join") && !has(&stage, "Leave"));
    page.close();
}

#[test]
fn a_voice_room_with_no_line_says_no_sound_is_carried() {
    let look = Rc::new(Phiogy::look(""));
    let app = Phiogy::holding(Rc::clone(&look), site(), Phiogy::board(), Err("off".to_string()), None);
    let mut stage = app.stage();
    assert!(app.desk().show("lounge"));
    stage.lay(DESK, PMode::of_width(DESK.width));
    let text = shown(&stage, &look).join(" ");
    assert!(text.contains("Sound needs the line, which is not open here. This room shows who is in it."), "{text}");
}

#[test]
fn step_43_the_voice_room_i_am_in_stays_at_the_foot_of_the_menu_on_every_page() {
    let board = Phiogy::board();
    let line = line_on(Arc::clone(&board));
    let look = Rc::new(Phiogy::look(""));
    let app = Phiogy::holding(Rc::clone(&look), site(), Arc::clone(&board), Ok(line.at()), None);
    let mut stage = app.stage();
    assert!(app.desk().show("lounge"));
    stage.lay(DESK, PMode::of_width(DESK.width));
    let has = |stage: &PStage, text: &str| shown(stage, &look).contains(&text.to_string());
    // In no voice room, the menu has no foot.
    assert!(foot(&stage, &look, "phone-off").is_empty() && app.desk().within().is_none());
    assert!(press(&mut stage, &look, "Join"));
    until("this window is in the room", || app.desk().calling().is_some_and(|seen| seen.joined));
    // In one: the room, the microphone, the others' sound and the way
    // out are at the menu's foot, the room marked and named first.
    let within = app.desk().within().map(|(id, label, _)| (id, label));
    assert_eq!(within, Some(("lounge".to_string(), "Lounge".to_string())));
    // Its name is whole at the foot: once in the rail, once there.
    assert_eq!(shown(&stage, &look).iter().filter(|text| *text == "Lounge").count(), 3, "{:?}", shown(&stage, &look));
    let stands = |stage: &PStage| ["mic", "phone-off"].iter().all(|name| foot(stage, &look, name).len() == 1) && foot(stage, &look, "volume").len() == 2;
    assert!(stands(&stage), "{:?}", drawn(&stage, &look).iter().filter(|mark| matches!(mark, PMark::Icon { .. })).collect::<Vec<_>>());
    // It is there on another page, where the room's own way out is not.
    assert!(app.desk().show("lobby"));
    stage.lay(DESK, PMode::of_width(DESK.width));
    assert!(stands(&stage) && !has(&stage, "Leave") && app.desk().calling().is_none());
    // The pointer over a control says what it is called.
    let mic = foot(&stage, &look, "mic")[0];
    assert!(!has(&stage, "Mute") && stage.send(&PEvent::PointerMove(mic)) && has(&stage, "Mute"), "{:?}", shown(&stage, &look));
    // Muted from there, and the others' sound off: each control shows it.
    assert!(stage.send(&PEvent::PointerDown(mic)));
    until("it is muted", || app.desk().within().is_some_and(|(.., seen)| seen.quiet));
    assert!(foot(&stage, &look, "mic").is_empty() && foot(&stage, &look, "mic-off").len() == 1 && has(&stage, "Unmute"));
    let sound = foot(&stage, &look, "volume")[1];
    assert!(stage.send(&PEvent::PointerDown(sound)));
    until("it hears nobody", || app.desk().within().is_some_and(|(.., seen)| seen.own.deaf));
    assert!(foot(&stage, &look, "volume-x").len() == 1 && foot(&stage, &look, "volume").len() == 1);
    // A press on the room there goes back to it.
    let room = foot(&stage, &look, "volume")[0];
    assert!(stage.send(&PEvent::PointerDown(room)));
    assert_eq!(*app.desk().open.borrow(), "lounge");
    stage.lay(DESK, PMode::of_width(DESK.width));
    assert!(has(&stage, "Leave") && has(&stage, "Sound on"));
    // The way out from another page: the line lets it go, and the foot
    // is gone.
    assert!(app.desk().show("lobby"));
    stage.lay(DESK, PMode::of_width(DESK.width));
    let out = foot(&stage, &look, "phone-off")[0];
    assert!(stage.send(&PEvent::PointerDown(out)));
    until("the line let it go", || super::peer::calling(&board, "phiogy/lounge").is_empty());
    assert!(app.desk().within().is_none() && foot(&stage, &look, "phone-off").is_empty());
}
