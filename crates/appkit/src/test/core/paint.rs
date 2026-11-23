use crate::pui::core::*;

#[test]
fn a_rectangle_holds_its_left_and_top_edge_and_not_its_right_and_bottom() {
    let rect = PRect::new(10.0, 20.0, 30.0, 40.0);
    assert!(rect.contains(PPoint::new(10.0, 20.0)) && rect.contains(PPoint::new(39.9, 59.9)));
    assert!(!rect.contains(PPoint::new(40.0, 30.0)) && !rect.contains(PPoint::new(20.0, 60.0)));
    assert!(!rect.contains(PPoint::new(9.9, 30.0)) && !rect.contains(PPoint::new(20.0, 19.9)));
}

#[test]
fn two_rectangles_side_by_side_never_both_hold_a_point() {
    let (left, right) = (PRect::new(0.0, 0.0, 10.0, 10.0), PRect::new(10.0, 0.0, 10.0, 10.0));
    for step in 0..=40 {
        let point = PPoint::new(step as f32 * 0.5, 5.0);
        assert!(!(left.contains(point) && right.contains(point)), "{point:?}");
    }
    // And the edge between them is the right one's.
    assert!(right.contains(PPoint::new(10.0, 5.0)) && !left.contains(PPoint::new(10.0, 5.0)));
}

#[test]
fn a_place_is_moved_and_seen_from_another() {
    let place = PPoint::new(3.0, 4.0);
    assert_eq!(place.plus(PPoint::new(1.0, -1.0)), PPoint::new(4.0, 3.0));
    assert_eq!(place.minus(PPoint::new(1.0, 1.0)), PPoint::new(2.0, 3.0));
    assert_eq!(place.plus(PPoint::new(7.0, 9.0)).minus(PPoint::new(7.0, 9.0)), place);
}

#[test]
fn a_moved_rectangle_keeps_its_size() {
    let rect = PRect::new(1.0, 2.0, 3.0, 4.0).moved(PPoint::new(10.0, 20.0));
    assert_eq!(rect, PRect::new(11.0, 22.0, 3.0, 4.0));
}

#[test]
fn a_new_colour_is_what_it_was_given_and_the_default_is_clear() {
    assert_eq!(PColor::new(0.1, 0.2, 0.3, 0.4), PColor { r: 0.1, g: 0.2, b: 0.3, a: 0.4 });
    assert_eq!(PColor::default().a, 0.0);
}

#[test]
fn a_whole_picture_is_never_cut_and_never_made_larger_than_it_is() {
    let room = PRect::new(100.0, 50.0, 400.0, 300.0);
    // One larger than its room is made to fit, in the room's middle.
    let (shown, lie) = PFit::Whole.place(room, PSize { width: 800.0, height: 400.0 });
    assert_eq!((shown, lie), (PRect::new(100.0, 100.0, 400.0, 200.0), PRect::new(100.0, 100.0, 400.0, 200.0)));
    // One smaller than its room stays as large as it is.
    let (shown, _) = PFit::Whole.place(room, PSize { width: 200.0, height: 100.0 });
    assert_eq!(shown, PRect::new(200.0, 150.0, 200.0, 100.0));
    // Whole is what a caller gets who says nothing.
    assert_eq!(PFit::default(), PFit::Whole);
}

#[test]
fn a_cover_fills_its_rectangle_and_cuts_what_does_not_fit_evenly() {
    let room = PRect::new(0.0, 0.0, 100.0, 100.0);
    let (shown, lie) = PFit::Cover.place(room, PSize { width: 400.0, height: 200.0 });
    // Only the rectangle shows; the picture stands as high as it and
    // twice as wide, half of what is over at each side.
    assert_eq!((shown, lie), (room, PRect::new(-50.0, 0.0, 200.0, 100.0)));
}

#[test]
fn a_cell_of_a_sheet_covers_its_rectangle() {
    // A sheet of eight by eight faces, each 100 across; the tenth face
    // is the second of the second row.
    let room = PRect::new(10.0, 20.0, 50.0, 50.0);
    let (shown, lie) = PFit::Cell { column: 8, index: 9 }.place(room, PSize { width: 800.0, height: 800.0 });
    assert_eq!(shown, room);
    // The sheet stands half its size, moved so that cell lies on the
    // rectangle: its left edge 50 and its top 50 before the rectangle's.
    assert_eq!(lie, PRect::new(-40.0, -30.0, 400.0, 400.0));
}
