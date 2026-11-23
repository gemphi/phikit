use crate::pui::core::*;
use crate::pui::window::*;
use crate::theme::PLook;

const LOOK: &str = "size\tmodal.share\t0.5\nsize\tmodal.head\t40\nsize\tmodal.radius\t20\nsize\ticon.size\t20\n";
const SIZE: PSize = PSize { width: 800.0, height: 600.0 };

#[test]
fn a_modal_is_a_square_in_the_middle_of_the_window() {
    let panel = PPanel::of(SIZE, PShape::Square, false, false, &PLook::read(LOOK));
    // Half of the shorter side, 600.
    assert_eq!(panel.rect, PRect::new(250.0, 150.0, 300.0, 300.0));
    assert_eq!(panel.body, PRect::new(250.0, 190.0, 300.0, 260.0));
    assert_eq!((panel.expand, panel.close), (PRect::new(470.0, 150.0, 40.0, 40.0), PRect::new(510.0, 150.0, 40.0, 40.0)));
}

#[test]
fn made_large_it_is_the_whole_window_and_its_corners_are_square() {
    let look = PLook::read(LOOK);
    let panel = PPanel::of(SIZE, PShape::Square, true, false, &look);
    assert_eq!(panel.rect, PRect::new(0.0, 0.0, 800.0, 600.0));
    let radius = |panel: PPanel| {
        let mut sheet = PSheet::new(&look);
        panel.draw("Profile", &mut sheet);
        match &sheet.mark[1] {
            PMark::Fill { radius, .. } => *radius,
            other => panic!("the panel is the second thing drawn, not {other:?}"),
        }
    };
    assert_eq!((radius(panel), radius(PPanel::of(SIZE, PShape::Square, false, false, &look))), (0.0, 20.0));
}

#[test]
fn a_press_on_a_button_does_its_deed_one_beside_the_panel_closes_it_and_one_on_it_is_the_content_s() {
    let panel = PPanel::of(SIZE, PShape::Square, false, false, &PLook::read(LOOK));
    assert_eq!(panel.deed(panel.close.center()), Some(PDeed::Close));
    assert_eq!(panel.deed(panel.expand.center()), Some(PDeed::Expand));
    assert_eq!(panel.deed(PPoint::new(10.0, 10.0)), Some(PDeed::Close));
    assert_eq!(panel.deed(panel.body.center()), None);
}

#[test]
fn in_a_hand_the_square_takes_the_share_the_theme_gives_a_hand() {
    let look = PLook::read(&format!("{LOOK}size\tmodal.hand\t0.9\n"));
    let side = |hand: bool, look: &PLook| PPanel::of(SIZE, PShape::Square, false, hand, look).rect.size.width;
    let short = SIZE.width.min(SIZE.height);
    assert_eq!([side(false, &look), side(true, &look)], [short * 0.5, short * 0.9]);
    // A theme that says nothing for a hand gives it the desk's share.
    assert_eq!(side(true, &PLook::read(LOOK)), short * 0.5);
}

#[test]
fn where_the_theme_says_how_much_of_the_window_it_fills_the_square_has_that_much_of_its_space() {
    // Half of 800 by 600 is 240,000: a square of about 490.
    let look = PLook::read(&format!("{LOOK}size\tmodal.area\t0.5\nsize\tmodal.share\t0.9\n"));
    let panel = PPanel::of(SIZE, PShape::Square, false, false, &look);
    let side = (SIZE.width * SIZE.height * 0.5).sqrt();
    assert_eq!((panel.rect.size.width, panel.rect.size.height), (side, side));
    assert!((panel.rect.center().x - 400.0).abs() < 0.01 && (panel.rect.center().y - 300.0).abs() < 0.01);
    // It never takes more of the shorter side than the share given: in a
    // tall window half the space would be wider than the window.
    let tall = PSize { width: 400.0, height: 1000.0 };
    assert_eq!(PPanel::of(tall, PShape::Square, false, false, &look).rect.size.width, 360.0);
}

#[test]
fn a_drawer_stands_at_the_right_edge_and_a_sheet_lies_at_the_foot_each_clear_of_the_window() {
    let look = PLook::read(&format!("{LOOK}size\tmodal.margin\t10\nsize\tdrawer.width\t300\nsize\tsheet.share\t0.4\nsize\tsheet.width\t500\n"));
    let drawer = PPanel::of(SIZE, PShape::Drawer, false, false, &look);
    // As high as the window leaves, a margin from its top, foot and side.
    assert_eq!((drawer.rect, drawer.body), (PRect::new(490.0, 10.0, 300.0, 580.0), PRect::new(490.0, 50.0, 300.0, 540.0)));
    assert_eq!((drawer.close, drawer.deed(PPoint::new(100.0, 300.0))), (PRect::new(750.0, 10.0, 40.0, 40.0), Some(PDeed::Close)));
    // Four tenths of the window's height, in the middle of its width.
    let sheet = PPanel::of(SIZE, PShape::Sheet, false, false, &look);
    assert_eq!(sheet.rect, PRect::new(150.0, 350.0, 500.0, 240.0));
    // In a window narrower than either, each is as wide as it leaves.
    let narrow = PSize { width: 320.0, height: 600.0 };
    let wide = |shape: PShape| PPanel::of(narrow, shape, false, false, &look).rect;
    assert_eq!([wide(PShape::Drawer), wide(PShape::Sheet)], [PRect::new(10.0, 10.0, 300.0, 580.0), PRect::new(10.0, 350.0, 300.0, 240.0)]);
    // Made large, either is the whole window, as a square is.
    assert_eq!(PPanel::of(SIZE, PShape::Drawer, true, false, &look).rect, PRect::new(0.0, 0.0, 800.0, 600.0));
    // A theme that says nothing of them gives a drawer the window's width
    // and a sheet half its height.
    let plain = PLook::read(LOOK);
    assert_eq!(PPanel::of(SIZE, PShape::Sheet, false, false, &plain).rect, PRect::new(0.0, 300.0, 800.0, 300.0));
}

#[test]
fn a_modal_s_shape_is_the_cell_after_its_icon_and_a_square_when_it_says_none() {
    let set = PSet::<PModal>::read("about\tAbout\tinfo\nfilter\tFilter\tfilter\tdrawer\nshare\tShare\tarrow-up\tsheet\nodd\tOdd\tbox\tround\n");
    let shape: Vec<PShape> = set.list().iter().map(|modal| modal.shape).collect();
    assert_eq!(shape, [PShape::Square, PShape::Drawer, PShape::Sheet, PShape::Square]);
}

#[test]
fn a_modal_of_the_whole_shape_fills_the_window() {
    use crate::pui::window::{PPanel, PShape};
    let look = crate::theme::PLook::read("size\tmodal.head\t56\nsize\tmodal.margin\t8\n");
    let size = PSize { width: 1040.0, height: 720.0 };
    assert_eq!(PShape::named("whole"), PShape::Whole);
    let panel = PPanel::of(size, PShape::Whole, false, false, &look);
    // The whole window, its head across the top and its body under it.
    assert_eq!(panel.rect, PRect::new(0.0, 0.0, 1040.0, 720.0));
    assert_eq!(panel.body, PRect::new(0.0, 56.0, 1040.0, 664.0));
}

#[test]
fn a_bare_modal_s_panel_has_no_head_and_all_of_it_is_its_content_s() {
    use crate::pui::window::{PModal, PPanel, PSet, PShape};
    let set = PSet::<PModal>::read("view\tPicture\timage\twhole\tbare\nabout\tAbout\tinfo\twhole\n");
    let bare: Vec<(PShape, bool)> = set.list().iter().map(|modal| (modal.shape, modal.bare)).collect();
    assert_eq!(bare, [(PShape::Whole, true), (PShape::Whole, false)]);
    let look = crate::theme::PLook::read("size\tmodal.head\t56\nsize\ticon.size\t20\n");
    let size = PSize { width: 1040.0, height: 720.0 };
    let headed = PPanel::of(size, PShape::Whole, false, false, &look);
    let panel = headed.bared(true);
    // Its body is all of it; where the close stood is the content's now.
    assert_eq!((panel.body, panel.rect), (headed.rect, headed.rect));
    assert_eq!((panel.deed(headed.close.center()), panel.deed(headed.expand.center())), (None, None));
    assert_eq!(headed.bared(false), headed);
    // It draws its veil and its ground, and neither a title nor a button.
    let drawn = |panel: PPanel| {
        let mut sheet = PSheet::new(&look);
        panel.draw("Picture", &mut sheet);
        sheet.mark.iter().filter(|mark| matches!(mark, PMark::Text { .. } | PMark::Icon { .. })).count()
    };
    assert_eq!((drawn(headed), drawn(panel)), (3, 0));
    // Under a caption it is moved down as any panel is, and stays bare.
    assert!(panel.lower(40.0).bare && panel.lower(40.0).body.origin.y == 40.0);
}
