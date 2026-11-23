//! The guide's tabs, the star of a room's title, and a bar with more
//! hosts than it has room for.

use appkit::pui::core::{PEvent, PMode, PPoint};

use super::{DESK, press, shown, staged};

#[test]
fn step_46_the_guide_s_tabs_keep_the_recent_the_starred_and_the_older_rooms() {
    let (app, mut stage, look) = staged(DESK);
    let desk = app.desk();
    let has = |stage: &appkit::pui::window::PStage, text: &str| shown(stage, &look).contains(&text.to_string());
    // With no tab chosen every room is listed, under the three tabs.
    assert!(["Recent", "Favorites", "Older", "lobby", "front-row"].iter().all(|text| has(&stage, text)));
    assert_eq!(desk.tabs(), (vec!["recent".to_string(), "favorite".to_string(), "older".to_string()], String::new()));
    // Nothing is starred yet, and the tab says how to.
    assert!(press(&mut stage, &look, "Favorites"));
    assert!(!has(&stage, "front-row") && has(&stage, "No favorites yet"));
    // A room starred is among the favorites, and so is the room of a
    // person I follow.
    let front = desk.rail().into_iter().flat_map(|(_, rung)| rung).find(|rung| rung.label == "front-row").map(|rung| rung.id).expect("the room");
    desk.star(&front, true);
    assert!(desk.starred(&front) && has(&stage, "front-row") && !has(&stage, "notes"));
    // A room I opened is a recent one; the others are the older ones.
    assert!(desk.show("notes"));
    stage.lay(DESK, PMode::of_width(DESK.width));
    assert!(press(&mut stage, &look, "Recent"));
    assert_eq!(desk.tabs().1, "recent");
    let recent = desk.sifted().into_iter().flat_map(|(_, rung)| rung).map(|rung| rung.id).collect::<Vec<_>>();
    assert!(recent.contains(&"notes".to_string()) && !recent.contains(&front), "{recent:?}");
    assert!(press(&mut stage, &look, "Older"));
    let older = desk.sifted().into_iter().flat_map(|(_, rung)| rung).map(|rung| rung.id).collect::<Vec<_>>();
    assert!(older.contains(&front) && !older.contains(&"notes".to_string()), "{older:?}");
    assert!(recent.iter().all(|id| !older.contains(id)));
    // The chosen tab again lets every room back.
    assert!(press(&mut stage, &look, "Older"));
    assert!(desk.tabs().1.is_empty() && has(&stage, "front-row") && has(&stage, "notes"));
}

#[test]
fn step_47_more_hosts_than_the_bar_has_room_for_are_rolled_to_with_the_wheel() {
    let (app, mut stage, _) = staged(DESK);
    let desk = app.desk();
    let template = desk.site.extra.template.first().cloned().expect("a template");
    // Hosts I made stand in the bar by their letters: they have no mark.
    (0..24).for_each(|number| _ = desk.create(&format!("host {number}"), &template));
    assert!(stage.tick());
    let place = desk.place();
    assert!(place.len() >= 24 && place.iter().rev().take(24).all(|row| row.entry.icon.is_empty() && row.face.is_none()));
    // They do not all fit: the wheel over the bar rolls them, a row a
    // notch, down to the last and back, and no further.
    let bar = stage.frame().activity.center();
    let down = PEvent::Scroll(bar, PPoint::new(0.0, -40.0));
    let up = PEvent::Scroll(bar, PPoint::new(0.0, 40.0));
    assert!(!stage.send(&up));
    let mut turned = 0;
    while stage.send(&down) {
        turned += 1;
        assert!(turned < 100);
    }
    assert!(turned > 0);
    assert_eq!((0..turned).filter(|_| stage.send(&up)).count(), turned);
    assert!(!stage.send(&up));
}
