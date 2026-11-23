use crate::pui::layout::PScroll;

#[test]
fn a_content_longer_than_its_room_scrolls_to_its_ends_and_no_further() {
    let mut scroll = PScroll::default();
    scroll.fit(1000.0, 400.0);
    assert!(scroll.turn(250.0) && scroll.at() == 250.0);
    assert!(scroll.turn(5000.0) && scroll.at() == 600.0);
    assert!(scroll.turn(-9000.0) && scroll.at() == 0.0);
}

#[test]
fn a_content_that_fits_does_not_scroll_and_one_that_shrinks_is_brought_back() {
    let mut scroll = PScroll::default();
    scroll.fit(300.0, 400.0);
    assert!(!scroll.turn(100.0) && scroll.at() == 0.0);
    scroll.fit(1000.0, 400.0);
    scroll.turn(600.0);
    // Less content, or more room: what is scrolled stays within it.
    scroll.fit(700.0, 400.0);
    assert_eq!(scroll.at(), 300.0);
    scroll.home();
    assert_eq!(scroll.at(), 0.0);
}
