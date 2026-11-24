//! A room as the web's own config gives it: what was said in it, the
//! headings of the rail, and pages made of blocks.

use appkit::pui::core::{PEvent, PMode, PPoint, PSize};

use super::{DESK, HAND, part, rect, row, shown, site, staged};

#[test]
fn the_site_reads_what_was_said_the_headings_and_the_pages() {
    let site = site();
    let (_, lobby) = site.plate("lobby").expect("the lobby");
    assert_eq!((lobby.head.as_str(), lobby.unread, lobby.message.len()), ("Text", 2, 3));
    assert_eq!(site.plate("lounge").map(|(_, plate)| plate.head.as_str()), Some("Voice"));
    assert_eq!(site.plate("lora").map(|(_, plate)| plate.head.as_str()), Some(""));
    // A message: who by name, when, its words, pictures, marks, what it
    // answers, and whether it is pinned.
    let (first, second) = (&lobby.message[0], &lobby.message[1]);
    assert_eq!((first.by.as_str(), first.day.as_str(), first.at.as_str(), first.pin), ("Tavi Reyes", "Yesterday", "21:15", true));
    assert_eq!(first.image, ["phiegg:worlds/w_brocade.png"]);
    assert_eq!((second.reply.as_str(), &second.mark), ("d1", &vec![("thumb".to_string(), 3), ("heart".to_string(), 1)]));
    // The pages: each with its blocks; a room with no page is its own.
    let kind = |page: &str| site.block(page).iter().map(|block| (block.kind.clone(), block.room.clone())).collect::<Vec<_>>();
    assert_eq!(kind("notes"), [("chat".to_string(), "lobby".to_string()), ("poll".to_string(), String::new())]);
    assert_eq!(kind("lounge"), [("chat".to_string(), "lounge".to_string())]);
    assert_eq!(site.view.len(), 3);
}

#[test]
fn step_12_a_room_shows_what_was_said_in_it_the_newest_at_its_foot() {
    let (app, stage, look) = staged(DESK);
    assert!(app.desk().show("lobby"));
    let text = shown(&stage, &look);
    let at = |wanted: &str| text.iter().position(|line| line == wanted).unwrap_or_else(|| panic!("{wanted} in {text:?}"));
    // Each day is named, then who said what and when, the oldest first.
    assert!(at("Yesterday") < at("The mark is drawn on the font now.") && at("The mark is drawn on the font now.") < at("Today"));
    assert!(at("Today") < at("Good. Show it at the stand up.") && at("Good. Show it at the stand up.") < at("I will bring both versions."));
    assert!(text.contains(&"Tavi Reyes".to_string()) && text.contains(&"Yesterday · 21:15".to_string()) && text.contains(&"You".to_string()));
    // Whom a message answers, how that began; its marks with their
    // counts. (Its pictures are drawn: `picture.rs`.)
    assert!(text.contains(&"Tavi Reyes: The mark is drawn on the font now.".to_string()), "{text:?}");
    assert!(text.contains(&"3".to_string()) && text.contains(&"1".to_string()));
    // A room nothing was said in says so.
    assert!(app.desk().show("lora"));
    assert!(shown(&stage, &look).contains(&app.desk().text.get("stream.none").to_string()));
}

#[test]
fn at_a_desk_the_people_stand_beside_what_was_said_and_in_a_hand_over_it() {
    for (size, beside) in [(DESK, true), (HAND, false)] {
        let (app, stage, look) = staged(size);
        assert!(app.desk().show("lobby"));
        let mark = super::drawn(&stage, &look);
        let place = |wanted: &str| mark.iter().find_map(|mark| match mark {
            appkit::pui::core::PMark::Text { at, text, .. } | appkit::pui::core::PMark::Head { at, text, .. } if text == wanted => Some(*at),
            _ => None,
        });
        let (said, people) = (place("I will bring both versions.").expect("the stream"), place("On the line").expect("the people"));
        let room = rect(&stage, "room");
        assert!(room.contains(said) && room.contains(people));
        assert_eq!((people.x > said.x + 200.0, people.y < said.y && people.x <= said.x + 1.0), (beside, !beside), "{said:?} {people:?}");
    }
}

#[test]
fn a_long_room_is_scrolled_with_the_wheel_from_its_foot() {
    let (app, mut stage, look) = staged(DESK);
    // A window too low for the three messages.
    let low = PSize { width: 1040.0, height: 260.0 };
    assert!(app.desk().show("lobby"));
    stage.lay(low, PMode::of_width(low.width));
    // It opens at its foot: the newest is there. The wheel goes back.
    let _ = shown(&stage, &look);
    let at = rect(&stage, "room").origin.plus(PPoint::new(100.0, 40.0));
    assert!(stage.send(&PEvent::Scroll(at, PPoint::new(0.0, 60.0))));
    assert!(part(&stage).contains(&"room"));
}

#[test]
fn step_13_the_rail_is_the_web_s_own_with_its_headings_and_a_page_is_its_blocks() {
    let (app, mut stage, look) = staged(DESK);
    assert!(app.desk().visit("phiace"));
    let text = shown(&stage, &look);
    // The guide's lines are the last of those words: the room's title
    // names its place and room too.
    let at = |wanted: &str| text.iter().rposition(|line| line == wanted).unwrap_or_else(|| panic!("{wanted} in {text:?}"));
    // Under a place its headings, each over its rooms.
    assert!(at("Phiace") < at("Text") && at("Text") < at("lobby") && at("lobby") < at("Voice") && at("Voice") < at("Lounge"));
    // The page `notes` is of the direct list: back there to open it.
    assert!(app.desk().show("lora"));
    // A page that is not a room of its own opens what its block names;
    // a block of a kind not drawn yet is named, not left out silently.
    assert!(row(&mut stage, &look, "notes"));
    assert_eq!((app.desk().open.borrow().as_str(), app.desk().room().as_str()), ("notes", "lobby"));
    let text = shown(&stage, &look);
    assert!(text.contains(&"I will bring both versions.".to_string()));
    assert!(text.contains(&"This page has a block this window does not draw yet: poll".to_string()), "{text:?}");
    // Its line is the room's it shows.
    assert_eq!(app.desk().key(&app.desk().room()), "phiogy/lobby");
}

#[test]
fn how_many_wait_in_a_room_stands_on_a_ground_of_its_own_in_the_rail() {
    // The lobby has two that wait: its row of the rail says so at its
    // end, on a figure's ground, and what the row is called does not.
    let (app, stage, look) = staged(DESK);
    // The lobby's place, by its other room, so the lobby stays unread.
    assert!(app.desk().show("lounge"));
    assert!(super::grounded(&stage, &look, "2") && !super::grounded(&stage, &look, "lobby"));
}
