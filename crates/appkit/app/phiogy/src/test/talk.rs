//! A room's talk and what a person keeps of it: what they wrote, the
//! marks they gave, what they pinned, what they searched for; kept in
//! a folder, and on a line that answers as Phiora does.

use appkit::service::{PStore, PTable};
use serde_json::{Value, json};

use super::exchange::line;
use super::{DESK, staged};
use crate::data;
use crate::kept::Kept;
use crate::phiora::Phiora;
use crate::site::Message;

/// A folder of the system's own for a test, empty.
fn folder(name: &str) -> PStore {
    let path = std::env::temp_dir().join(format!("phiogy-test-{name}-{}", std::process::id()));
    _ = std::fs::remove_dir_all(&path);
    PStore::at(&path.to_string_lossy())
}

/// The message of that id in the open room.
fn one(app: &crate::Phiogy, id: &str) -> Message {
    app.desk().said().into_iter().find(|one| one.id == id).unwrap_or_else(|| panic!("the message {id}"))
}

#[test]
fn what_a_person_writes_joins_the_room_s_talk_spelled_as_the_web_spells_it() {
    let (app, ..) = staged(DESK);
    let desk = app.desk();
    assert!(desk.show("lobby"));
    assert_eq!(desk.said().len(), 3);
    // Nothing but spaces is nothing; words join the talk at once, by
    // the person, today, at an hour of the clock.
    assert!(!desk.write("   ") && desk.write("  Hello there "));
    let said = desk.said();
    let last = said.last().expect("it was said");
    assert_eq!((said.len(), last.by.as_str(), last.who.as_str(), last.day.as_str(), last.text.as_str()), (4, "You", "you", "Today", "Hello there"));
    assert!(last.id.starts_with('m') && last.at.len() == 5 && last.at.contains(':'), "{last:?}");
    // An answer names what it answers, once.
    *desk.now.answer.borrow_mut() = "d2".to_string();
    assert!(desk.write("Yes") && desk.write("And this answers nobody"));
    let said = desk.said();
    assert_eq!((said[4].reply.as_str(), said[5].reply.as_str()), ("d2", ""));
    // It is one document, under the room's name on the line, with the
    // web's own names in it; another room has none of it.
    let kept = desk.kept.get("sent");
    assert_eq!(kept["phiogy/lobby"].as_array().map(Vec::len), Some(3));
    assert_eq!((&kept["phiogy/lobby"][0]["by"], &kept["phiogy/lobby"][0]["text"], &kept["phiogy/lobby"][1]["reply"]), (&json!("you"), &json!("Hello there"), &json!("d2")));
    assert!(desk.show("lora"));
    assert!(desk.said().is_empty());
}

#[test]
fn marks_pins_and_a_search_are_the_site_s_with_the_person_s_own_over_them() {
    let (app, ..) = staged(DESK);
    let desk = app.desk();
    assert!(desk.show("lobby"));
    // A mark the site has counts the person once they give it; a like
    // where the site has none is theirs alone; either is taken back.
    let mark = |id: &str| desk.mark(&one(&app, id));
    assert_eq!(mark("d2"), [("thumb".to_string(), 3, false), ("heart".to_string(), 1, false)]);
    desk.give("d2", "heart");
    desk.give("d1", &desk.like());
    assert_eq!((mark("d2")[1].clone(), mark("d1")), (("heart".to_string(), 2, true), vec![("heart".to_string(), 1, true)]));
    desk.give("d1", "heart");
    assert!(mark("d1").is_empty());
    // What the site pinned can be unpinned, and another pinned.
    let pinned = |id: &str| desk.pinned(&one(&app, id));
    assert_eq!((pinned("d1"), pinned("d3")), (true, false));
    desk.pin(&one(&app, "d1"));
    desk.pin(&one(&app, "d3"));
    assert_eq!((pinned("d1"), pinned("d3")), (false, true));
    // A search holds the words of a message, or the name of who said it.
    let found = |find: &str| desk.sought(find).iter().map(|one| one.id.clone()).collect::<Vec<_>>();
    assert_eq!((found("Stand UP"), found("tavi"), found("nothing such"), found(" ").len()), (vec!["d2".to_string()], vec!["d1".to_string()], vec![], 3));
    // What waited in a room waits no more once it was opened.
    let (app, ..) = staged(DESK);
    assert_eq!(app.desk().waiting("lobby"), 2);
    assert!(app.desk().show("lobby"));
    assert_eq!((app.desk().waiting("lobby"), app.desk().kept.get("read")["phiogy/lobby"].clone()), (0, json!(true)));
}

#[test]
fn what_a_person_keeps_is_in_their_folder_and_there_the_next_time() {
    let store = folder("kept");
    let kept = Kept::open(Some(store.clone()), None);
    assert_eq!((kept.get("pin"), kept.at.clone()), (Value::Null, None));
    kept.put("pin", "phiogy/lobby/d3", json!(true));
    kept.set("sent", json!({ "phiogy/lobby": [{ "id": "m1", "text": "Hello" }] }));
    // One file, the documents by the names Phiora keeps them under.
    let word = PTable::read(data::KEEP);
    let file: Value = serde_json::from_str(&store.read(word.get("file")).expect("it was written")).expect("JSON");
    assert_eq!((&file["room-pin"]["phiogy/lobby/d3"], &file["room"]["phiogy/lobby"][0]["text"]), (&json!(true), &json!("Hello")));
    let again = Kept::open(Some(store), None);
    assert_eq!((again.flag("pin", "phiogy/lobby/d3"), again.flag("pin", "phiogy/lobby/d1")), (Some(true), None));
    // Kept nowhere, a document lasts as long as what holds it.
    let none = Kept::none();
    none.put("pin", "a", json!(true));
    assert_eq!(none.flag("pin", "a"), Some(true));
}

#[test]
fn step_23_with_phiora_answering_what_a_person_keeps_is_kept_there_under_their_session() {
    // The line answers who a page is and keeps their documents as
    // Phiora does: asked the same way.
    let line = line();
    let store = folder("phiora");
    let word = PTable::read(data::KEEP);
    let kept = Kept::open(Some(store.clone()), Some(Phiora::at(&line.at())));
    assert_eq!(kept.at, Some(line.at()));
    // The session it was given is kept, to be the same person again.
    let session = store.read(word.get("session")).expect("the session is kept");
    assert!(session.starts_with("phi_sid="), "{session}");
    kept.put("mark", "phiogy/lobby/d2/heart", json!(true));
    kept.settle();
    // The folder's own copy lost, the next time Phiora's is read: by
    // the same session.
    assert!(store.write(word.get("file"), "{}"));
    let again = Kept::open(Some(store.clone()), Some(Phiora::at(&line.at())));
    assert_eq!((again.flag("mark", "phiogy/lobby/d2/heart"), store.read(word.get("session"))), (Some(true), Some(session)));
    // Another person has none of it; and with nobody answering the
    // folder is all there is.
    let other = Kept::open(Some(folder("other")), Some(Phiora::at(&line.at())));
    assert_eq!(other.flag("mark", "phiogy/lobby/d2/heart"), None);
    let alone = Kept::open(Some(folder("alone")), Some(Phiora::at("127.0.0.1:9")));
    assert_eq!(alone.at, None);
}
