use crate::pui::core::*;
use crate::pui::window::*;
use crate::theme::PLook;

const LOOK: &str = "size\ttab.height\t40\n";
const SIZE: PSize = PSize { width: 800.0, height: 600.0 };

#[test]
fn at_a_desk_the_bar_stands_at_the_left_and_the_tabs_lie_over_the_body() {
    let frame = PFrame::of(SIZE, PMode::Desktop, &PLook::read(LOOK), 50.0, 0.0, true);
    assert_eq!((frame.activity, frame.tab, frame.body), (PRect::new(0.0, 0.0, 50.0, 600.0), PRect::new(50.0, 0.0, 750.0, 40.0), PRect::new(50.0, 40.0, 750.0, 560.0)));
    // The menu the bar shows stands beside it, and the rest moves over.
    let frame = PFrame::of(SIZE, PMode::Desktop, &PLook::read(LOOK), 50.0, 200.0, true);
    assert_eq!((frame.menu, frame.tab, frame.body), (PRect::new(50.0, 0.0, 200.0, 600.0), PRect::new(250.0, 0.0, 550.0, 40.0), PRect::new(250.0, 40.0, 550.0, 560.0)));
}

#[test]
fn in_a_hand_the_tabs_are_at_the_top_and_the_bar_lies_at_the_bottom() {
    let frame = PFrame::of(SIZE, PMode::Mobile, &PLook::read(LOOK), 50.0, 0.0, true);
    assert_eq!((frame.tab, frame.body, frame.activity), (PRect::new(0.0, 0.0, 800.0, 40.0), PRect::new(0.0, 40.0, 800.0, 510.0), PRect::new(0.0, 550.0, 800.0, 50.0)));
    assert_eq!(frame.menu, PRect::default());
    // A menu is drawn out over the body, which keeps its place.
    let frame = PFrame::of(SIZE, PMode::Mobile, &PLook::read(LOOK), 50.0, 200.0, true);
    assert_eq!((frame.menu, frame.body), (PRect::new(0.0, 40.0, 800.0, 510.0), PRect::new(0.0, 40.0, 800.0, 510.0)));
}

#[test]
fn what_an_app_has_not_takes_no_room() {
    let frame = PFrame::of(SIZE, PMode::Desktop, &PLook::read(LOOK), 0.0, 0.0, false);
    assert_eq!(frame.body, PRect::new(0.0, 0.0, 800.0, 600.0));
}

#[test]
fn the_bar_and_its_menu_stay_a_margin_clear_of_the_window_s_edges() {
    let look = PLook::read(&format!("{LOOK}size\tactivity.margin\t10\n"));
    let frame = PFrame::of(SIZE, PMode::Desktop, &look, 50.0, 200.0, false);
    // Clear of the top, the foot and the side; the body starts where
    // the menu ends, and has the window's whole height.
    assert_eq!((frame.activity, frame.menu, frame.body), (PRect::new(10.0, 10.0, 50.0, 580.0), PRect::new(60.0, 10.0, 200.0, 580.0), PRect::new(260.0, 0.0, 540.0, 600.0)));
    // Lying, it is clear of the foot and the sides, the body ends a
    // margin over it, and a menu drawn out is clear of the top as well.
    let frame = PFrame::of(SIZE, PMode::Mobile, &look, 50.0, 200.0, false);
    assert_eq!((frame.activity, frame.body, frame.menu), (PRect::new(10.0, 540.0, 780.0, 50.0), PRect::new(0.0, 0.0, 800.0, 530.0), PRect::new(10.0, 10.0, 780.0, 520.0)));
    // An app with no bar has no margin round one.
    assert_eq!(PFrame::of(SIZE, PMode::Desktop, &look, 0.0, 0.0, false).body, PRect::new(0.0, 0.0, 800.0, 600.0));
}

#[test]
fn under_a_caption_everything_else_starts_lower_and_the_bar_has_no_margin_at_its_top() {
    let look = PLook::read(&format!("{LOOK}size\tactivity.margin\t10\nsize\tcaption.height\t30\n"));
    let frame = PFrame::of(SIZE, PMode::Desktop, &look, 50.0, 200.0, true);
    assert_eq!((frame.caption, frame.activity, frame.menu), (PRect::new(0.0, 0.0, 800.0, 30.0), PRect::new(10.0, 30.0, 50.0, 560.0), PRect::new(60.0, 30.0, 200.0, 560.0)));
    assert_eq!((frame.tab, frame.body), (PRect::new(260.0, 30.0, 540.0, 40.0), PRect::new(260.0, 70.0, 540.0, 530.0)));
    // In a hand the tabs are under it, and a menu drawn out starts where
    // the body does.
    let frame = PFrame::of(SIZE, PMode::Mobile, &look, 50.0, 200.0, true);
    assert_eq!((frame.caption, frame.tab, frame.body), (PRect::new(0.0, 0.0, 800.0, 30.0), PRect::new(0.0, 30.0, 800.0, 40.0), PRect::new(0.0, 70.0, 800.0, 460.0)));
    assert_eq!((frame.menu, frame.activity), (PRect::new(10.0, 70.0, 780.0, 460.0), PRect::new(10.0, 540.0, 780.0, 50.0)));
}
