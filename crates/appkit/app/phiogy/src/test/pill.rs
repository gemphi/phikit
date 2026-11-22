//! The pills of a room's call, pressed on the window's own stage: this
//! window joins over a real line, mutes and leaves.

use std::sync::Arc;

use super::peer::{calling, until};
use crate::board::lock;

#[cfg(not(target_family = "wasm"))]
#[test]
fn step_11_i_join_a_room_s_call_from_its_title_mute_and_leave() {
    use std::rc::Rc;

    use appkit::pui::PApp;
    use appkit::pui::core::PMode;

    use super::{DESK, press, shown, site};
    use crate::Phiogy;
    let board = Phiogy::board();
    let line = super::exchange::line_on(Arc::clone(&board));
    let look = Rc::new(Phiogy::look(""));
    let app = Phiogy::holding(Rc::clone(&look), site(), Arc::clone(&board), Ok(line.at()), None);
    let mut stage = app.stage();
    stage.lay(DESK, PMode::of_width(DESK.width));
    let has = |stage: &appkit::pui::window::PStage, text: &str| shown(stage, &look).contains(&text.to_string());
    // Out of the call the room's title has one pill.
    assert!(has(&stage, "Join") && !has(&stage, "Leave"));
    assert!(press(&mut stage, &look, "Join"));
    // The line has this window as a caller of the room, as it has a page.
    until("this window is in the call", || app.desk().calling().is_some_and(|seen| seen.joined));
    let me = app.desk().me().expect("it has a name on the line");
    assert_eq!(calling(&board, "phiogy/lora"), std::slice::from_ref(&me));
    // The room says so: this window among the callers, what carries the
    // call, and the pills to mute and to leave.
    let text = shown(&stage, &look);
    assert!(["In the call", "this window", "Mute", "Leave", "Carried by stand-in."].iter().all(|line| text.contains(&line.to_string())) && text.contains(&me), "{text:?}");
    assert!(!has(&stage, "Join"));
    // Mute, and the pill offers the way back.
    assert!(press(&mut stage, &look, "Mute"));
    until("it is muted", || app.desk().calling().is_some_and(|seen| seen.quiet));
    assert!(has(&stage, "Unmute") && !has(&stage, "Mute"));
    // In another room it is not in that room's call: Join is offered.
    assert!(app.desk().show("lobby"));
    assert!(has(&stage, "Join") && !has(&stage, "Leave"));
    assert!(app.desk().show("lora"));
    // Leave: the line lets it go.
    assert!(press(&mut stage, &look, "Leave"));
    until("the line let it go", || lock(&board).open().is_empty());
    assert!(has(&stage, "Join") && !has(&stage, "this window"));
}

#[test]
fn with_no_line_there_is_no_call_to_join() {
    use appkit::pui::PApp;
    // A window that says its line is where nothing listens.
    let look = std::rc::Rc::new(crate::Phiogy::look(""));
    let app = crate::Phiogy::holding(std::rc::Rc::clone(&look), super::site(), crate::Phiogy::board(), Ok("localhost:9".to_string()), None);
    let mut stage = app.stage();
    stage.lay(super::DESK, appkit::pui::core::PMode::of_width(super::DESK.width));
    app.desk().join();
    assert!(app.desk().calling().is_none());
    let text = super::shown(&stage, &look).join(" ");
    assert!(text.contains("The call could not be joined"), "{text}");
    app.desk().leave();
    assert!(!super::shown(&stage, &look).join(" ").contains("could not be joined"));
}
