use std::rc::Rc;

use appkit::pui::PApp;
use appkit::pui::core::PMode;
use appkit::service::PStore;

use crate::test::{DESK, shown};
use crate::{History, Phigen, Stand, Turn, Who};

fn stand() -> Box<Stand> {
    Box::new(Stand::new("I heard: {said}"))
}

fn store(name: &str) -> PStore {
    let root = std::env::temp_dir().join("phigen-test-keep").join(name);
    std::fs::remove_dir_all(&root).ok();
    PStore::at(&root.to_string_lossy())
}

#[test]
fn a_history_is_written_a_turn_a_line_and_read_again_as_it_was() {
    let mut history = History::new(stand(), "Hello.").timed(|| 1_700_000_000);
    history.say("first");
    history.begin();
    history.say("a line\nand another, with a\ttab and a \\ in it");
    history.begin();
    let text = history.text();
    // The chat nothing was said in is not written; a turn is one line,
    // after one that says when the chat began.
    assert_eq!(text.lines().count(), 8);
    assert!(text.starts_with("0\twhen\t1700000000\n"), "{text}");
    let again = History::read(&text, stand(), "Hello.");
    // A new chat is open over the two that were kept.
    assert_eq!((again.list().len(), again.index(), again.current().title()), (3, 0, None));
    assert_eq!(again.list()[1].title(), Some("a line\nand another, with a\ttab and a \\ in it"));
    assert_eq!((again.list()[1].when, again.list()[2].when), (1_700_000_000, 1_700_000_000));
    assert_eq!(again.list()[2].turn, [Turn::new(Who::Phigen, "Hello."), Turn::new(Who::Person, "first"), Turn::new(Who::Phigen, "I heard: first")]);
}

#[test]
fn a_table_that_cannot_be_read_gives_a_history_with_one_new_chat() {
    let history = History::read("nonsense\n\n3\tnobody\thello\n", stand(), "Hello.");
    assert_eq!((history.list().len(), history.current().title()), (1, None));
}

#[test]
fn step_19_what_i_said_is_there_when_phigen_opens_again() {
    let store = store("again");
    let look = Rc::new(Phigen::look(""));
    let kept = |store: &PStore| History::read(&store.read("history.tsv").unwrap_or_default(), stand(), "Hello.").kept(store.clone(), "history.tsv");
    // The first time there is nothing kept. Something is said.
    let app = Phigen::holding(Rc::clone(&look), kept(&store));
    let mut stage = app.stage();
    stage.lay(DESK, PMode::Desktop);
    Phigen::key(&mut stage, "remember this");
    // Phigen is opened again: a new chat is open, and the earlier one is
    // in the list.
    let app = Phigen::holding(Rc::clone(&look), kept(&store));
    let mut stage = app.stage();
    stage.lay(DESK, PMode::Desktop);
    assert_eq!(app.desk().history.borrow().current().title(), None);
    assert!(shown(&stage, &look).contains(&"remember this".to_string()));
}
