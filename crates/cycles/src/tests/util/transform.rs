// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::util::*;

fn near(left: [f32; 3], right: [f32; 3]) -> bool {
    left.iter().zip(right).all(|(left, right)| (left - right).abs() < 1e-5)
}

#[test]
fn a_rectangle_starts_as_the_unit_square_and_can_be_given_by_its_reach() {
    assert_eq!(BoundBox2D::default(), BoundBox2D { left: 0.0, right: 1.0, bottom: 0.0, top: 1.0 });
    assert_eq!(BoundBox2D::of_radius([2.0, 1.0]), BoundBox2D { left: -2.0, right: 2.0, bottom: -1.0, top: 1.0 });
}

#[test]
fn a_point_is_scaled_and_moved_and_a_direction_only_scaled() {
    let transform = Transform::translate(1.0, 2.0, 3.0) * Transform::scale(2.0, 2.0, 2.0);
    assert_eq!(transform.point([1.0, 1.0, 1.0]), [3.0, 4.0, 5.0]);
    assert_eq!(transform.direction([1.0, 1.0, 1.0]), [2.0, 2.0, 2.0]);
    assert_eq!(Transform::default().point([4.0, 5.0, 6.0]), [4.0, 5.0, 6.0]);
}

#[test]
fn a_product_does_its_right_side_first() {
    let scale_then_move = Transform::translate(1.0, 0.0, 0.0) * Transform::scale(2.0, 1.0, 1.0);
    let move_then_scale = Transform::scale(2.0, 1.0, 1.0) * Transform::translate(1.0, 0.0, 0.0);
    assert_eq!(scale_then_move.point([1.0, 0.0, 0.0]), [3.0, 0.0, 0.0]);
    assert_eq!(move_then_scale.point([1.0, 0.0, 0.0]), [4.0, 0.0, 0.0]);
}

#[test]
fn an_inverse_undoes_its_transform() {
    let transform = Transform { x: [0.0, -2.0, 0.0, 5.0], y: [3.0, 0.0, 0.0, -1.0], z: [0.0, 0.0, 0.5, 2.0] };
    let point = [1.5, -2.5, 4.0];
    assert!(near(transform.inverse().point(transform.point(point)), point));
    assert!(near((transform * transform.inverse()).point(point), point));
    assert_eq!(Transform::translate(1.0, 2.0, 3.0).inverse(), Transform::translate(-1.0, -2.0, -3.0));
}

#[test]
fn a_matrix_with_no_inverse_still_gives_numbers() {
    let flat = Transform::scale(1.0, 0.0, 1.0).inverse();
    assert!(flat.word().iter().all(|part| part.is_finite()), "{flat:?}");
}

#[test]
fn a_viewplane_is_brought_to_the_unit_square() {
    let transform = Transform::from_viewplane(&BoundBox2D::of_radius([2.0, 1.0]));
    assert_eq!(transform.point([-2.0, -1.0, 0.0]), [0.0, 0.0, 0.0]);
    assert_eq!(transform.point([2.0, 1.0, 0.0]), [1.0, 1.0, 0.0]);
    assert_eq!(transform.point([0.0, 0.0, 7.0]), [0.5, 0.5, 7.0]);
}

#[test]
fn the_floats_go_to_a_device_row_by_row() {
    let word = Transform::translate(7.0, 8.0, 9.0).word();
    assert_eq!(word, [1.0, 0.0, 0.0, 7.0, 0.0, 1.0, 0.0, 8.0, 0.0, 0.0, 1.0, 9.0]);
}

#[test]
fn a_column_is_read_and_written_down_the_three_rows() {
    let mut transform = Transform::translate(7.0, 8.0, 9.0);
    assert_eq!((transform.column(0), transform.column(3)), ([1.0, 0.0, 0.0], [7.0, 8.0, 9.0]));
    transform.set_column(1, [4.0, 5.0, 6.0]);
    assert_eq!((transform.x[1], transform.y[1], transform.z[1]), (4.0, 5.0, 6.0));
    assert_eq!(Transform::EMPTY.word(), [0.0; 12]);
}

#[test]
fn a_mirror_has_a_negative_scale_and_a_turn_does_not() {
    assert!(Transform::scale(-1.0, 1.0, 1.0).negative_scale() && Transform::scale(-1.0, -1.0, -1.0).negative_scale());
    assert!(!Transform::scale(-1.0, -1.0, 1.0).negative_scale() && !Transform::IDENTITY.negative_scale());
}

#[test]
fn a_scale_is_uniform_when_every_axis_has_it() {
    assert_eq!(Transform::scale(2.0, 2.0, 2.0).uniform_scale(), Some(4.0));
    assert_eq!((Transform::translate(1.0, 2.0, 3.0) * Transform::scale(-3.0, 3.0, 3.0)).uniform_scale(), Some(9.0));
    assert_eq!(Transform::scale(2.0, 2.0, 1.0).uniform_scale(), None);
}

#[test]
fn a_normal_is_turned_by_the_transposed_inverse() {
    // A plane through x + y = 0, stretched along x: its normal leans to y.
    let stretch = Transform::translate(5.0, 0.0, 0.0) * Transform::scale(2.0, 1.0, 1.0);
    let normal = stretch.transposed_inverse().direction([1.0, 1.0, 0.0]);
    assert!(near(normal, [0.5, 1.0, 0.0]));
    // The surface's own direction, stretched, is still square to it.
    let along = stretch.direction([1.0, -1.0, 0.0]);
    assert!((normal[0] * along[0] + normal[1] * along[1] + normal[2] * along[2]).abs() < 1e-6);
}
