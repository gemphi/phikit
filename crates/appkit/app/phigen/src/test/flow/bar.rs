//! The steps of `FLOW.md` about the activity bar and the menus it shows:
//! beside it at a desk, drawn out over the chat in a hand.

use appkit::pui::core::{PEvent, PMark, PPoint, PRect};

use super::{fold, icon, part, press, row};
use crate::Phigen;
use crate::test::{DESK, HAND, drawn, lit, pictured, rect, shown, staged};

#[test]
fn step_14_the_bar_shows_my_chats_beside_it_and_puts_them_away() {
    let (_, mut stage, look) = staged(DESK);
    Phigen::key(&mut stage, "hi");
    // The bar is a column of icons; the menu of its Chat row is beside
    // it. Both are under the caption, and 8 clear of the window's foot
    // and side.
    assert_eq!((stage.frame().activity, stage.frame().menu), (PRect::new(8.0, 44.0, 56.0, 668.0), PRect::new(64.0, 44.0, 200.0, 668.0)));
    // My chats have the group under New chat and Search, to the foot.
    assert_eq!(rect(&stage, "thread"), PRect::new(72.0, 156.0, 184.0, 540.0));
    // The pin, in the caption, puts the menu away and brings it back.
    assert!(press(&mut stage, fold));
    assert_eq!((stage.frame().menu.size.width, part(&stage)), (0.0, vec!["title", "talk", "ask"]));
    assert!(press(&mut stage, fold));
    assert_eq!(part(&stage), ["title", "talk", "ask", "thread"]);
    // The Search row shows the Search window.
    assert!(row(&mut stage, &look, "Search"));
    assert_eq!(part(&stage), ["find", "thread"]);
}

#[test]
fn step_31_home_and_chat_are_at_the_top_menu_in_the_middle_and_i_am_at_the_foot() {
    let (app, mut stage, look) = staged(DESK);
    // At the bar's top, at its own middle, and the menus' rows.
    let at = ["home", "chat", "layout-grid"].map(|name| pictured(&stage, &look, name).expect("on the bar"));
    assert_eq!(at, [PPoint::new(36.0, 72.0), PPoint::new(36.0, 112.0), PPoint::new(36.0, 378.0)]);
    // Menu: what it holds is beside the bar in place of my chats, in
    // three groups, each on a ground of its own: at the top, round the
    // middle, and at the foot.
    assert!(icon(&mut stage, &look, "layout-grid"));
    let text = shown(&stage, &look);
    assert!(["Search", "Model", "Appearance", "Light", "Dark", "Glass", "Glass Light", "Settings", "Profile"].iter().all(|row| text.contains(&row.to_string())), "{text:?}");
    assert!(!part(&stage).contains(&"thread"));
    // They are lit from the right, against the bar's icons beside them
    // (appkit's shared menu, 23 November 2026).
    let ground = lit(&drawn(&stage, &look), stage.frame().menu, 90.0, true);
    assert_eq!(ground, [PRect::new(72.0, 44.0, 184.0, 96.0), PRect::new(72.0, 270.0, 184.0, 216.0), PRect::new(72.0, 608.0, 184.0, 96.0)]);
    // A look is chosen there, and the settings and my profile opened.
    assert!(row(&mut stage, &look, "Dark"));
    assert_eq!(app.desk().shade.name(), "dark");
    assert!(row(&mut stage, &look, "Light") && row(&mut stage, &look, "Profile"));
    assert_eq!(stage.window.modal(), Some("profile"));
    assert!(stage.send(&PEvent::KeyDown(PEvent::ESCAPE)));
    // Home goes to the start, a new chat, and shows the ways to begin;
    // Chat shows my chats again; a press on the row whose menu is shown
    // puts it away.
    Phigen::key(&mut stage, "hi");
    assert!(icon(&mut stage, &look, "home"));
    assert_eq!((part(&stage), app.desk().history.borrow().current().title()), (vec!["title", "hello", "ask", "card"], None));
    assert!(row(&mut stage, &look, "Search"));
    assert_eq!(part(&stage), ["find"]);
    assert!(icon(&mut stage, &look, "chat"));
    assert_eq!(part(&stage), ["title", "hello", "ask", "card", "thread"]);
    assert!(icon(&mut stage, &look, "chat"));
    assert_eq!((stage.frame().menu.size.width, part(&stage)), (0.0, vec!["title", "hello", "ask", "card"]));
}

#[test]
fn step_14_in_a_hand_the_bar_lies_under_my_thumb_and_draws_my_chats_out() {
    let (app, mut stage, look) = staged(HAND);
    assert_eq!(stage.frame().activity, PRect::new(8.0, 736.0, 374.0, 56.0));
    Phigen::key(&mut stage, "first");
    assert_eq!(part(&stage), ["title", "talk", "ask"]);
    // Chat draws its menu out over the chat: New chat, Search, my chats.
    assert!(icon(&mut stage, &look, "chat"));
    assert_eq!(part(&stage), ["thread"]);
    // New chat: the field in the middle again, and the menu put away.
    assert!(row(&mut stage, &look, "New chat"));
    assert_eq!(part(&stage), ["title", "hello", "ask", "card"]);
    assert!(icon(&mut stage, &look, "chat"));
    assert_eq!(part(&stage), ["thread"]);
    // A press on a title opens that chat and puts the list away.
    let thread = rect(&stage, "thread");
    assert!(stage.send(&PEvent::PointerDown(PPoint::new(thread.origin.x + 60.0, thread.origin.y + 2.0 * 40.0 + 20.0))));
    assert_eq!(part(&stage), ["title", "talk", "ask"]);
    assert_eq!(app.desk().history.borrow().current().title(), Some("first"));
}

#[test]
fn step_31_in_a_hand_home_goes_to_the_start_and_draws_what_it_holds_out() {
    let (app, mut stage, look) = staged(HAND);
    Phigen::key(&mut stage, "first");
    assert!(icon(&mut stage, &look, "home"));
    // A new chat is open under the menu, which lies over the body,
    // under the caption, 8 clear of the window's sides and of the bar.
    assert_eq!((app.desk().history.borrow().current().title(), stage.frame().menu), (None, PRect::new(8.0, 44.0, 374.0, 684.0)));
    let text = shown(&stage, &look);
    assert!(["New chat", "Search", "Model"].iter().all(|row| text.contains(&row.to_string())), "{text:?}");
    // Home again puts it away, and the start is there.
    assert!(icon(&mut stage, &look, "home"));
    assert_eq!(part(&stage), ["title", "hello", "ask", "card"]);
    assert_eq!(app.desk().history.borrow().list().len(), 2);
}

#[test]
fn step_32_phigen_s_mark_is_in_the_caption_over_the_bar_and_its_upper_line_is_twice_its_lower() {
    let (app, mut stage, look) = staged(DESK);
    // In the caption, over the bar's middle line, with Phigen's name
    // beside it. The mark is the pin (appkit's shared caption, 23
    // November 2026): no other pin is drawn, the pointer over the mark
    // shows the pin in its place, and a press on it puts the bar's menu
    // away and brings it back. Home is what goes to the start.
    assert_eq!(pictured(&stage, &look, "phigen-chat"), Some(PPoint::new(36.0, 22.0)));
    assert_eq!(pictured(&stage, &look, "pin-open"), None);
    assert!(drawn(&stage, &look).iter().any(|mark| matches!(mark, PMark::Text { at, text, .. } if text == "Phigen" && at.x == 80.0 && at.y == 11.0)));
    assert!(stage.send(&appkit::pui::core::PEvent::PointerMove(PPoint::new(36.0, 22.0))));
    assert_eq!(pictured(&stage, &look, "pin-open"), Some(PPoint::new(36.0, 22.0)));
    Phigen::key(&mut stage, "hi");
    assert!(press(&mut stage, fold));
    assert_eq!(stage.frame().menu.size.width, 0.0);
    assert!(press(&mut stage, fold));
    assert_eq!(stage.frame().menu.size.width, 200.0);
    assert!(icon(&mut stage, &look, "home"));
    assert_eq!((part(&stage), app.desk().history.borrow().current().title()), (vec!["title", "hello", "ask", "card"], None));
    // In a hand the mark is the caption's first square, and no pin
    // stands beside it.
    let (_, hand, look) = staged(HAND);
    assert_eq!((pictured(&hand, &look, "phigen-chat"), pictured(&hand, &look, "pin-shut")), (Some(PPoint::new(30.0, 22.0)), None));
    // Both marks script/logo.py draws are among Phigen's own assets, and
    // in each the upper of the two lines is 100% bigger than the lower.
    for name in ["phigen-chat", "phigen-bar"] {
        let text = std::fs::read_to_string(format!("{}/asset/icon/{name}.svg", env!("CARGO_MANIFEST_DIR"))).expect("drawn by script/logo.py");
        let wide: Vec<f32> = text.split(" width=\"").skip(1).filter_map(|rest| rest.split('"').next()?.parse().ok()).collect();
        assert_eq!((wide.len(), wide[0]), (2, wide[1] * 2.0), "{name}");
    }
}
