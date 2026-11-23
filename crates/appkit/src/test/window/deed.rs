use super::window;
use crate::pui::window::*;

#[test]
fn a_table_names_a_deed_by_its_kind_and_its_target() {
    assert_eq!(PDeed::named("space", "chat"), Some(PDeed::Show("chat".to_string())));
    assert_eq!(PDeed::named("host", "history"), Some(PDeed::Front("history".to_string())));
    assert_eq!(PDeed::named("modal", "profile"), Some(PDeed::Open("profile".to_string())));
    assert_eq!([PDeed::named("side", "thread"), PDeed::named("fold", ""), PDeed::named("tell", "begin")], [Some(PDeed::Side), Some(PDeed::Fold), Some(PDeed::Tell("begin".to_string()))]);
    assert_eq!(PDeed::named("dance", "now"), None);
}

#[test]
fn a_deed_changes_what_the_window_shows_and_the_window_says_so() {
    let mut window = window();
    let (shading, pref) = (PDeed::Show("shading".to_string()), PDeed::Open("pref".to_string()));
    assert!(!window.shows(&shading) && window.apply(&shading) && window.shows(&shading));
    assert!(window.apply(&pref) && window.shows(&pref));
    // An open modal is made large and a square again, and opens as a
    // square the next time.
    assert!(window.apply(&PDeed::Expand) && window.wide());
    assert!(window.apply(&PDeed::Close) && window.modal().is_none() && !window.wide());
    // The bar's menu is drawn out and put away, and going to a space puts
    // it away; at a desk it is put away and brought back; a word is not
    // the window's to do.
    assert!(window.apply(&PDeed::Side) && window.drawer() && window.shows(&PDeed::Side));
    assert!(window.apply(&PDeed::Show("layout".to_string())) && !window.drawer());
    assert!(window.apply(&PDeed::Fold) && window.fold() && window.apply(&PDeed::Fold) && !window.fold());
    // Picking a row of the bar shows its menu, wherever the bar is.
    let chat = PDeed::Pick("chat".to_string());
    assert!(window.apply(&PDeed::Fold) && !window.shows(&chat) && window.apply(&chat));
    assert!(window.shows(&chat) && window.pick() == "chat" && !window.fold() && window.drawer());
    // Choosing one names it and draws nothing out.
    assert!(window.apply(&PDeed::Side) && !window.drawer());
    window.choose("menu");
    assert!(window.pick() == "menu" && !window.drawer());
    assert!(!window.apply(&PDeed::Tell("begin".to_string())));
    // A space the window does not have is not shown.
    assert!(!window.apply(&PDeed::Show("sculpting".to_string())));
}

#[test]
fn a_host_is_brought_in_front_in_whichever_space_holds_it() {
    let mut window = window();
    let (outliner, node) = (PDeed::Front("outliner".to_string()), PDeed::Front("node".to_string()));
    assert!(window.apply(&outliner) && window.shows(&outliner));
    assert_eq!(window.front(), 1);
    // The node editor is in the other space: that space is shown.
    assert!(window.apply(&node));
    assert_eq!(window.current().map(PSpace::slug), Some("shading"));
    assert!(!window.apply(&PDeed::Front("nowhere".to_string())));
}

#[test]
fn an_area_comes_to_hold_another_host() {
    let mut window = window();
    let swap = PDeed::Hold("outliner".to_string(), "buts".to_string());
    assert!(window.apply(&swap));
    assert_eq!([window.index("outliner"), window.index("buts"), window.index("view3d")], [None, Some(1), Some(0)]);
    // No area of the space shown holds it any more: nothing changes.
    assert!(!window.apply(&swap));
}
