use super::{space, window};
use crate::pui::core::*;
use crate::pui::window::*;

const SIZE: PSize = PSize { width: 1200.0, height: 800.0 };

/// A window whose one space has two zones: a viewport, and an outliner
/// over a property editor.
fn two_zone() -> PWindow {
    let mut layout = space("layout", &["view3d"]);
    layout.zone.push(PZone { area: vec![PArea::holding("outliner"), PArea::holding("buts")], ..PZone::of(PEntry::new("side", "Side", "")) });
    PWindow::new("Book", vec![layout])
}

#[test]
fn at_a_desk_the_zones_stand_side_by_side_and_a_zone_s_areas_are_stacked() {
    let place = two_zone().place(SIZE, PMode::Desktop);
    let rect: Vec<(&str, PRect)> = place.iter().map(|place| (place.host.as_str(), place.rect)).collect();
    assert_eq!(rect, [("view3d", PRect::new(0.0, 0.0, 600.0, 800.0)), ("outliner", PRect::new(600.0, 0.0, 600.0, 400.0)), ("buts", PRect::new(600.0, 400.0, 600.0, 400.0))]);
    assert!(place.iter().all(|place| place.turn == 0.0));
    assert_eq!((place[2].zone, place[2].area), (1, 1));
}

#[test]
fn in_a_hand_the_area_in_front_takes_the_whole_window() {
    let mut window = two_zone();
    let phone = PSize { width: 390.0, height: 844.0 };
    let shown = |window: &PWindow| window.place(phone, PMode::Mobile).into_iter().map(|place| (place.host, place.rect)).collect::<Vec<_>>();
    assert_eq!(shown(&window), [("view3d".to_string(), PRect::new(0.0, 0.0, 390.0, 844.0))]);
    // Another is brought in front; one the space does not have is not.
    assert!(window.bring(2));
    assert_eq!((shown(&window)[0].0.as_str(), window.front()), ("buts", 2));
    assert!(!window.bring(3));
    assert_eq!(window.front(), 2);
}

#[test]
fn in_space_every_area_is_a_panel_of_its_own_in_an_arc_around_the_person() {
    let place = two_zone().place(SIZE, PMode::Spatial);
    let turn = setting("spatial.turn");
    assert_eq!(place.iter().map(|place| place.turn).collect::<Vec<_>>(), [-turn, 0.0, turn]);
    let panel = PRect::new(0.0, 0.0, setting("spatial.width"), setting("spatial.height"));
    assert!(place.iter().all(|place| place.rect == panel));
    // Two panels stand either side of straight ahead.
    let pair = window().place(SIZE, PMode::Spatial);
    assert_eq!(pair.iter().map(|place| place.turn).collect::<Vec<_>>(), [-turn / 2.0, turn / 2.0]);
}

#[test]
fn the_same_window_is_laid_out_by_its_width() {
    let window = two_zone();
    let shown = |width: f32| window.place(PSize { width, height: 800.0 }, PMode::of_width(width)).len();
    assert_eq!((shown(1200.0), shown(400.0)), (3, 1));
}

#[test]
fn showing_another_space_puts_its_first_area_in_front() {
    let mut window = window();
    assert!(window.bring(1));
    assert!(window.show("shading"));
    assert_eq!((window.front(), window.area_count()), (0, 1));
    assert_eq!(PWindow::new("Empty", Vec::new()).place(SIZE, PMode::Desktop), []);
}

#[test]
fn at_a_desk_a_zone_is_as_wide_as_its_share_of_the_whole() {
    let mut window = two_zone();
    window.space[0].zone[0].share = 3.0;
    let rect: Vec<PRect> = window.place(SIZE, PMode::Desktop).iter().map(|place| place.rect).collect();
    assert_eq!(rect, [PRect::new(0.0, 0.0, 900.0, 800.0), PRect::new(900.0, 0.0, 300.0, 400.0), PRect::new(900.0, 400.0, 300.0, 400.0)]);
}

#[test]
fn an_area_is_found_by_the_host_it_holds() {
    let window = two_zone();
    assert_eq!([window.index("view3d"), window.index("buts"), window.index("node")], [Some(0), Some(2), None]);
}
