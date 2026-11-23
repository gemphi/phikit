use std::rc::Rc;

use appkit::pui::PApp;
use appkit::pui::core::{PEvent, PMode, PPoint};
use appkit::pui::window::{PDeed, PStage};

use crate::test::{DESK, rect, shown};
use crate::{History, Phigen, Stand};

/// The time the tests run at, and a chat kept from twenty days before.
const NOW: u64 = 1_700_000_000;
const KEPT: &str = "0\twhen\t1698272000\n0\tphigen\tHello.\n0\tperson\tan old question\n0\tphigen\tI heard: an old question\n";

/// Phigen with one chat from before and one from today, on the Search
/// window.
fn searching() -> (Phigen, PStage, Rc<appkit::theme::PLook>) {
    let look = Rc::new(Phigen::look(""));
    let history = History::read(KEPT, Box::new(Stand::new("I heard: {said}")), "Hello.").timed(|| NOW);
    let app = Phigen::holding(Rc::clone(&look), history);
    let mut stage = app.stage();
    stage.lay(DESK, PMode::Desktop);
    Phigen::key(&mut stage, "a new question");
    stage.window.apply(&PDeed::Show("search".to_string()));
    stage.lay(DESK, PMode::Desktop);
    (app, stage, look)
}

/// A press in the Search window's column: `across` from its left edge and
/// `down` from the window's top. What is pressed is what was drawn, so it
/// is drawn first. The column is 744 wide in a part of 776.
fn press(stage: &mut PStage, look: &appkit::theme::PLook, across: f32, down: f32) -> bool {
    shown(stage, look);
    let find = rect(stage, "find");
    stage.send(&PEvent::PointerDown(PPoint::new(find.origin.x + 24.0 + across, find.origin.y + down)))
}

/// The lines the Search window shows, without the bar's.
fn listed(stage: &PStage, look: &appkit::theme::PLook) -> Vec<String> {
    shown(stage, look).into_iter().filter(|line| line.contains("question") || line.contains("turns")).collect()
}

/// The pills are 218 down and 38 high; the chats begin 272 down, 72 apart.
const PILL: f32 = 237.0;
const FIRST: f32 = 308.0;

#[test]
fn step_13_the_search_window_lists_my_chats_with_the_day_each_began() {
    let (_, stage, look) = searching();
    let text = shown(&stage, &look);
    assert!(["Find a chat", "Search chats", "All", "Today", "Earlier", "Newest"].iter().all(|line| text.contains(&line.to_string())), "{text:?}");
    // The newest first: each title in the bar's list and in the window,
    // and under it in the window the day it began and how much was said.
    let window: Vec<String> = text.into_iter().filter(|line| line.contains("turns")).collect();
    assert_eq!(window, ["Today · turns: 3", "Oct 25 · turns: 3"]);
}

#[test]
fn step_13_what_i_type_searches_the_titles_and_a_press_opens_the_chat() {
    let (app, mut stage, look) = searching();
    "OLD".chars().for_each(|key| _ = stage.send(&PEvent::KeyDown(key)));
    assert_eq!(listed(&stage, &look).iter().filter(|line| line.contains("turns")).count(), 1);
    assert!(press(&mut stage, &look, 100.0, FIRST));
    assert_eq!(app.desk().history.borrow().current().title(), Some("an old question"));
    let part: Vec<&str> = stage.slot().iter().map(|slot| slot.part.as_str()).collect();
    assert_eq!(part, ["title", "talk", "ask", "thread"]);
}

#[test]
fn step_13_a_pill_narrows_the_list_to_a_day_and_the_last_turns_the_order() {
    let (_, mut stage, look) = searching();
    let under = |stage: &PStage| listed(stage, &look).into_iter().filter(|line| line.contains("turns")).collect::<Vec<_>>();
    // "All" is 54.5 wide; "Today" begins 8 after it and is 69.5 wide;
    // "Earlier" begins 8 after that.
    assert!(press(&mut stage, &look, 97.0, PILL));
    assert_eq!(under(&stage), ["Today · turns: 3"]);
    assert!(press(&mut stage, &look, 170.0, PILL));
    assert_eq!(under(&stage), ["Oct 25 · turns: 3"]);
    assert!(press(&mut stage, &look, 20.0, PILL));
    assert_eq!(under(&stage).len(), 2);
    // The pill at the right end turns the order, and reads what it is.
    assert!(press(&mut stage, &look, 744.0 - 20.0, PILL));
    assert_eq!(under(&stage), ["Oct 25 · turns: 3", "Today · turns: 3"]);
    assert!(shown(&stage, &look).contains(&"Oldest".to_string()));
}
