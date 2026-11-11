// SPDX-License-Identifier: Apache-2.0
use crate::util::*;

fn unit() -> BoundBox {
    BoundBox { min: [0.0; 3], max: [1.0; 3] }
}

#[test]
fn an_empty_box_holds_nothing_until_a_point_is_grown_into_it() {
    let mut bound = BoundBox::EMPTY;
    assert!(!bound.valid());
    bound.grow([1.0, 2.0, 3.0]);
    assert!(bound.valid());
    assert_eq!((bound.min, bound.max, bound.size()), ([1.0, 2.0, 3.0], [1.0, 2.0, 3.0], [0.0; 3]));
    bound.grow([-1.0, 5.0, 3.0]);
    assert_eq!((bound.min, bound.max, bound.center()), ([-1.0, 2.0, 3.0], [1.0, 5.0, 3.0], [0.0, 3.5, 3.0]));
}

#[test]
fn a_point_that_is_not_a_number_leaves_the_box_and_an_infinite_one_spoils_it() {
    let mut bound = unit();
    bound.grow([f32::NAN; 3]);
    assert_eq!(bound, unit());
    bound.grow_safe([f32::INFINITY, 0.0, 0.0]);
    assert_eq!(bound, unit());
    bound.grow([f32::INFINITY, 0.0, 0.0]);
    assert!(!bound.valid());
}

#[test]
fn a_box_grows_by_another_box() {
    let mut bound = unit();
    bound.grow_box(&BoundBox { min: [-2.0, 0.5, 0.5], max: [0.5, 0.5, 4.0] });
    assert_eq!((bound.min, bound.max), ([-2.0, 0.0, 0.0], [1.0, 1.0, 4.0]));
}

#[test]
fn a_transformed_box_fits_its_eight_corners() {
    let moved = unit().transformed(&Transform::translate(1.0, 2.0, 3.0));
    assert_eq!((moved.min, moved.max), ([1.0, 2.0, 3.0], [2.0, 3.0, 4.0]));
    // Mirrored along x, the corners change sides.
    let mirrored = unit().transformed(&Transform::scale(-2.0, 1.0, 1.0));
    assert_eq!((mirrored.min, mirrored.max), ([-2.0, 0.0, 0.0], [0.0, 1.0, 1.0]));
    assert!(!BoundBox::EMPTY.transformed(&Transform::IDENTITY).valid());
}

#[test]
fn boxes_that_touch_intersect_and_boxes_apart_do_not() {
    let beside = |x: f32| BoundBox { min: [x, 0.0, 0.0], max: [x + 1.0, 1.0, 1.0] };
    assert!(unit().intersects(&beside(0.5)) && unit().intersects(&beside(1.0)));
    assert!(!unit().intersects(&beside(1.5)));
}

#[test]
fn the_area_of_a_box_is_that_of_its_six_faces() {
    let bound = BoundBox { min: [0.0; 3], max: [1.0, 2.0, 3.0] };
    assert_eq!((bound.half_area(), bound.area(), bound.safe_area()), (11.0, 22.0, 22.0));
    assert_eq!(bound.center2(), [1.0, 2.0, 3.0]);
    // An empty box has no area that can be trusted: the safe one is zero.
    assert_eq!((BoundBox::EMPTY.safe_area(), BoundBox::EMPTY.half_area()), (0.0, f32::INFINITY));
}

#[test]
fn two_boxes_have_a_box_around_both_and_a_box_in_both() {
    let (first, second) = (unit(), BoundBox { min: [0.5, 0.5, -1.0], max: [2.0, 3.0, 0.5] });
    let around = first.merge(&second);
    assert_eq!((around.min, around.max), ([0.0, 0.0, -1.0], [2.0, 3.0, 1.0]));
    let mut within = first;
    within.intersect(&second);
    assert_eq!((within.min, within.max), ([0.5, 0.5, 0.0], [1.0, 1.0, 0.5]));
    // Boxes that do not meet leave one that ends before it starts.
    let mut apart = first;
    apart.intersect(&BoundBox { min: [5.0; 3], max: [6.0; 3] });
    assert!(!apart.valid() && apart.safe_area() == 0.0);
}
