use appkit::pui::core::{PEvent, PPoint};
use appkit::pui::window::PStage;
use appkit::theme::PLook;

use crate::Phigen;
use crate::test::{DESK, listed, rect, staged};

/// A press on New chat, the first row of the menu beside the bar.
fn begin(stage: &mut PStage, look: &PLook) -> bool {
    let row = listed(stage, look, "New chat").expect("a row of the menu");
    stage.send(&PEvent::PointerDown(row))
}

/// A press on the title of that number in the thread, under its heading.
fn title(stage: &mut PStage, row: usize) -> bool {
    let thread = rect(stage, "thread");
    stage.send(&PEvent::PointerDown(PPoint::new(thread.origin.x + 60.0, thread.origin.y + (row + 1) as f32 * 40.0 + 20.0)))
}

#[test]
fn the_menu_s_row_begins_a_chat_above_the_others_once_the_open_one_is_used() {
    let (app, mut stage, look) = staged(DESK);
    // The open chat has had nothing said in it: it is the new chat.
    assert!(begin(&mut stage, &look));
    assert_eq!(app.desk().history.borrow().list().len(), 1);
    Phigen::key(&mut stage, "first");
    assert!(begin(&mut stage, &look));
    let history = app.desk().history.borrow();
    assert_eq!((history.list().len(), history.index(), history.current().title()), (2, 0, None));
}

#[test]
fn a_press_on_a_title_opens_that_chat() {
    let (app, mut stage, look) = staged(DESK);
    Phigen::key(&mut stage, "first");
    begin(&mut stage, &look);
    // The menu was pressed last and takes no keys, so what is typed still
    // goes to the ask.
    Phigen::key(&mut stage, "second");
    assert_eq!(app.desk().history.borrow().current().title(), Some("second"));
    assert!(title(&mut stage, 1));
    assert_eq!(app.desk().history.borrow().current().title(), Some("first"));
    // There is no third title to press, and the heading opens nothing.
    assert!(!title(&mut stage, 2));
    let thread = rect(&stage, "thread");
    assert!(!stage.send(&PEvent::PointerDown(PPoint::new(thread.origin.x + 60.0, thread.origin.y + 20.0))));
    assert_eq!(app.desk().history.borrow().index(), 1);
}
