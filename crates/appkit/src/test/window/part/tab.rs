use crate::pui::core::*;
use crate::pui::window::*;
use crate::test::window::window;
use crate::theme::PLook;

const LOOK: &str = "size\ttab.width\t100\nsize\ttab.gap\t10\nsize\ttab.radius\t15\nsize\ttext.advance\t10\nsize\ttext.line\t20\n";

#[test]
fn there_is_a_tab_for_each_space_and_a_press_on_one_shows_it() {
    let spot = PTab.spot(PRect::new(50.0, 0.0, 750.0, 50.0), &window(), &PLook::read(LOOK));
    assert_eq!(spot, [(PRect::new(60.0, 10.0, 100.0, 30.0), PDeed::Show("layout".to_string())), (PRect::new(170.0, 10.0, 100.0, 30.0), PDeed::Show("shading".to_string()))]);
}

#[test]
fn tabs_narrow_to_share_a_bar_that_has_no_room_for_them() {
    let spot = PTab.spot(PRect::new(0.0, 0.0, 150.0, 50.0), &window(), &PLook::read(LOOK));
    let wide: Vec<f32> = spot.iter().map(|(rect, _)| rect.size.width).collect();
    assert_eq!(wide, [60.0, 60.0]);
}

#[test]
fn the_tab_of_the_space_shown_is_filled_and_every_label_is_in_the_middle_of_its_tab() {
    let look = PLook::read(LOOK);
    let mut sheet = PSheet::new(&look);
    PTab.draw(PRect::new(50.0, 0.0, 750.0, 50.0), &window(), &mut sheet);
    // The bar, then one pill: the first space is the one shown.
    let pill: Vec<&PMark> = sheet.mark.iter().filter(|mark| matches!(mark, PMark::Fill { radius, .. } if *radius > 0.0)).collect();
    assert_eq!(pill, [&PMark::Fill { rect: PRect::new(60.0, 10.0, 100.0, 30.0), radius: 15.0, color: PColor::default() }]);
    // "layout" is 6 letters of 10: it starts 30 before the middle, 110.
    assert!(sheet.mark.contains(&PMark::Text { at: PPoint::new(80.0, 15.0), text: "layout".to_string(), color: PColor::default() }));
    assert_eq!(sheet.text_list(), ["layout", "shading"]);
}
