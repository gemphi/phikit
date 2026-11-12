// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::util::*;

fn near(left: [f32; 3], right: [f32; 3]) -> bool {
    left.iter().zip(right).all(|(left, right)| (left - right).abs() < 1e-4)
}

/// A field of view of 90 degrees between 1 and 11 units.
fn perspective() -> ProjectionTransform {
    ProjectionTransform::perspective(std::f32::consts::FRAC_PI_2, 1.0, 11.0)
}

#[test]
fn a_perspective_puts_the_near_plane_at_zero_and_the_far_plane_at_one() {
    assert!(near(perspective().point([0.0, 0.0, 1.0]), [0.0, 0.0, 0.0]));
    assert!(near(perspective().point([0.0, 0.0, 11.0]), [0.0, 0.0, 1.0]));
}

#[test]
fn a_perspective_divides_by_depth_and_by_the_tangent_of_half_the_view() {
    // At 90 degrees the edge of the view is as far to the side as ahead.
    assert!(near(perspective().point([2.0, -2.0, 2.0]), [1.0, -1.0, 0.55]));
    let narrow = ProjectionTransform::perspective(std::f32::consts::FRAC_PI_4, 1.0, 11.0);
    assert!((narrow.point([1.0, 0.0, 1.0])[0] - 1.0 / (std::f32::consts::FRAC_PI_8).tan()).abs() < 1e-4);
}

#[test]
fn a_point_at_the_eye_goes_nowhere() {
    assert_eq!(perspective().point([1.0, 1.0, 0.0]), [0.0; 3]);
}

#[test]
fn an_orthographic_view_keeps_x_and_y_and_scales_depth() {
    let view = ProjectionTransform::orthographic(1.0, 5.0);
    assert_eq!(view.point([3.0, -2.0, 2.0]), [3.0, -2.0, 0.5]);
    assert_eq!(view.direction([1.0, 2.0, 4.0]), [1.0, 2.0, 1.0]);
}

#[test]
fn an_inverse_undoes_its_projection() {
    let point = [0.5, -0.25, 3.0];
    assert!(near(perspective().inverse().point(perspective().point(point)), point));
    let product = perspective() * perspective().inverse();
    assert!(product.word().iter().zip(ProjectionTransform::IDENTITY.word()).all(|(left, right)| (left - right).abs() < 1e-4), "{product:?}");
}

#[test]
fn a_matrix_with_no_inverse_inverts_to_the_identity() {
    let flat = ProjectionTransform::from(Transform::scale(1.0, 0.0, 1.0));
    assert_eq!(flat.inverse(), ProjectionTransform::IDENTITY);
}

#[test]
fn a_projection_and_a_transform_multiply_either_way() {
    let moved = Transform::translate(0.0, 0.0, 1.0);
    // Moved one unit ahead first, the near plane is reached from the eye.
    assert!(near((perspective() * moved).point([0.0, 0.0, 0.0]), [0.0, 0.0, 0.0]));
    assert!(near((moved * perspective()).point([0.0, 0.0, 1.0]), [0.0, 0.0, 1.0]));
    assert_eq!(ProjectionTransform::from(moved).point([1.0, 2.0, 3.0]), moved.point([1.0, 2.0, 3.0]));
}

#[test]
fn the_floats_go_to_a_device_row_by_row() {
    let word = ProjectionTransform::default().word();
    assert_eq!(word, [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]);
}

#[test]
fn a_radius_and_an_angle_are_a_point_of_the_plane() {
    let point = polar_to_cartesian(2.0, std::f32::consts::FRAC_PI_2);
    assert!(point[0].abs() < 1e-6 && (point[1] - 2.0).abs() < 1e-6);
    assert_eq!(polar_to_cartesian(3.0, 0.0), [3.0, 0.0]);
}
