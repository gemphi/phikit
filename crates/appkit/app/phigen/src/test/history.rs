use crate::{History, Stand, Turn, Who};

fn history() -> History {
    History::new(Box::new(Stand::new("I heard: {said}")), "Hello.")
}

#[test]
fn a_history_opens_with_one_chat_that_greets() {
    let history = history();
    assert_eq!((history.list().len(), history.index()), (1, 0));
    assert_eq!(history.current().turn, [Turn::new(Who::Phigen, "Hello.")]);
    assert_eq!(history.current().title(), None);
}

#[test]
fn what_a_person_says_is_kept_and_answered_and_a_blank_is_not_said() {
    let mut history = history();
    assert!(history.say("  what is phikit?  "));
    assert!(!history.say("   "));
    let said: Vec<(Who, &str)> = history.current().turn.iter().skip(1).map(|turn| (turn.who, turn.text.as_str())).collect();
    assert_eq!(said, [(Who::Person, "what is phikit?"), (Who::Phigen, "I heard: what is phikit?")]);
    // A chat is called by the first thing the person said in it.
    history.say("and phiray?");
    assert_eq!(history.current().title(), Some("what is phikit?"));
}

#[test]
fn a_chat_is_begun_above_the_others_unless_the_open_one_is_still_unused() {
    let mut history = history();
    history.begin();
    assert_eq!(history.list().len(), 1);
    history.say("first");
    history.begin();
    assert_eq!((history.list().len(), history.index(), history.current().title()), (2, 0, None));
    // The earlier chat is below it, and can be opened again.
    assert!(history.open(1));
    assert_eq!(history.current().title(), Some("first"));
    assert!(!history.open(2));
    assert_eq!(history.index(), 1);
}
