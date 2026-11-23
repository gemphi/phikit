use crate::pui::core::*;
use crate::theme::PLook;

/// A look for the tests: one colour, and how wide a counted letter is.
const LOOK: &str = "size\ttext.advance\t8\ncolor\tink\t#000000\n";

#[test]
fn a_sheet_keeps_what_was_drawn_on_it_in_order() {
    let look = PLook::read(LOOK);
    let mut sheet = PSheet::new(&look);
    let ink = sheet.color("ink");
    let (square, round, mark) = (PRect::new(0.0, 0.0, 10.0, 10.0), PRect::new(1.0, 1.0, 8.0, 8.0), PRect::new(0.0, 0.0, 16.0, 16.0));
    sheet.fill(square, ink);
    sheet.round(round, 4.0, ink);
    sheet.text(PPoint::new(2.0, 2.0), "hello", ink);
    sheet.icon("send", mark, ink);
    let drawn = [
        PMark::Fill { rect: square, radius: 0.0, color: ink },
        PMark::Fill { rect: round, radius: 4.0, color: ink },
        PMark::Text { at: PPoint::new(2.0, 2.0), text: "hello".to_string(), color: ink },
        PMark::Icon { name: "send".to_string(), rect: mark, color: ink },
    ];
    assert_eq!(sheet.mark, drawn);
    assert_eq!(sheet.text_list(), ["hello"]);
}

#[test]
fn a_sheet_measures_by_counting_until_a_screen_measures_for_it() {
    let look = PLook::read(LOOK);
    assert_eq!(PSheet::new(&look).measure("hello"), 40.0);
    let gauge = |text: &str| text.len() as f32 * 3.0;
    assert_eq!(PSheet::gauged(&look, &gauge).measure("hello"), 15.0);
}

#[test]
fn a_clip_and_its_end_are_marks_like_the_rest() {
    let look = PLook::read(LOOK);
    let mut sheet = PSheet::new(&look);
    let (part, ink) = (PRect::new(10.0, 10.0, 50.0, 50.0), sheet.color("ink"));
    sheet.clip(part);
    sheet.fill(PRect::new(0.0, 0.0, 100.0, 100.0), ink);
    sheet.unclip();
    assert_eq!((sheet.mark.first(), sheet.mark.last(), sheet.mark.len()), (Some(&PMark::Clip(part)), Some(&PMark::Unclip), 3));
}

#[test]
fn a_heading_is_a_mark_of_its_own_and_is_measured_as_much_wider_as_it_is_larger() {
    let look = PLook::read("size\ttext.advance\t8\nsize\ttext.size\t16\nsize\thead.size\t32\n");
    let mut sheet = PSheet::new(&look);
    let ink = sheet.color("ink");
    sheet.head(PPoint::new(4.0, 4.0), "Find", ink);
    assert_eq!(sheet.mark, [PMark::Head { at: PPoint::new(4.0, 4.0), text: "Find".to_string(), color: ink }]);
    assert_eq!((sheet.measure("Find"), sheet.measure_head("Find")), (32.0, 64.0));
    assert_eq!(sheet.text_list(), ["Find"]);
}

#[test]
fn a_picture_is_kept_as_a_mark_with_how_it_lies() {
    let look = crate::theme::PLook::read("");
    let mut sheet = PSheet::new(&look);
    let rect = PRect::new(1.0, 2.0, 30.0, 40.0);
    sheet.picture("worlds/one.png", rect, 6.0, PFit::Cover);
    assert_eq!(sheet.mark, [PMark::Picture { name: "worlds/one.png".to_string(), rect, radius: 6.0, fit: PFit::Cover }]);
}

#[test]
fn a_sheet_told_how_large_pictures_are_says_so_and_one_that_was_not_says_nothing() {
    let look = crate::theme::PLook::read("");
    let known = |name: &str| (name == "worlds/one.png").then_some(PSize { width: 640.0, height: 360.0 });
    let sheet = PSheet::new(&look).sized(&known);
    assert_eq!((sheet.natural("worlds/one.png"), sheet.natural("worlds/two.png")), (Some(PSize { width: 640.0, height: 360.0 }), None));
    assert_eq!(PSheet::new(&look).natural("worlds/one.png"), None);
}
