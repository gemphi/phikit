use crate::pui::core::{PRect, PSize};
use crate::pui::layout::{PRun, places};

fn size(width: f32, height: f32) -> Option<PSize> {
    Some(PSize { width, height })
}

#[test]
fn across_the_start_is_at_the_left_the_end_at_the_right_and_the_middle_in_the_middle_of_the_whole() {
    let row = PRect::new(0.0, 0.0, 600.0, 40.0);
    // However wide the sides, the middle is the whole's middle.
    let [start, middle, end] = places(row, PRun::Across, [size(200.0, 20.0), size(100.0, 40.0), size(40.0, 20.0)], 8.0);
    assert_eq!(start, Some(PRect::new(0.0, 10.0, 200.0, 20.0)));
    assert_eq!(middle, Some(PRect::new(250.0, 0.0, 100.0, 40.0)));
    assert_eq!(end, Some(PRect::new(560.0, 10.0, 40.0, 20.0)));
    // An empty place is nothing, and the others stand where they would.
    let [start, middle, end] = places(row, PRun::Across, [None, size(100.0, 40.0), size(40.0, 20.0)], 8.0);
    assert_eq!((start, middle.map(|rect| rect.origin.x), end.map(|rect| rect.origin.x)), (None, Some(250.0), Some(560.0)));
}

#[test]
fn a_middle_that_would_run_into_a_side_keeps_clear_of_it() {
    let row = PRect::new(0.0, 0.0, 600.0, 40.0);
    let [_, middle, _] = places(row, PRun::Across, [size(280.0, 20.0), size(100.0, 20.0), size(40.0, 20.0)], 8.0);
    assert_eq!(middle.map(|rect| rect.origin.x), Some(288.0));
}

#[test]
fn down_the_start_is_at_the_top_the_end_at_the_foot_and_each_in_the_middle_across() {
    let column = PRect::new(10.0, 0.0, 100.0, 300.0);
    let [top, middle, foot] = places(column, PRun::Down, [size(60.0, 30.0), size(80.0, 50.0), size(100.0, 20.0)], 0.0);
    assert_eq!(top, Some(PRect::new(30.0, 0.0, 60.0, 30.0)));
    assert_eq!(middle, Some(PRect::new(20.0, 125.0, 80.0, 50.0)));
    assert_eq!(foot, Some(PRect::new(10.0, 280.0, 100.0, 20.0)));
}
