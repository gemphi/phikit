// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::util::*;

fn near(left: Transform, right: Transform) -> bool {
    left.word().iter().zip(right.word()).all(|(left, right)| (left - right).abs() < 1e-4)
}

/// A quarter turn about z.
fn quarter() -> Transform {
    Transform { x: [0.0, -1.0, 0.0, 0.0], y: [1.0, 0.0, 0.0, 0.0], z: [0.0, 0.0, 1.0, 0.0] }
}

#[test]
fn a_transform_taken_apart_goes_back_together() {
    let all = [Transform::IDENTITY, Transform::translate(1.0, -2.0, 3.0), Transform::scale(2.0, 3.0, 4.0), quarter(), Transform::translate(5.0, 0.0, 1.0) * quarter() * Transform::scale(1.0, 2.0, 0.5)];
    for transform in all {
        let decomp = motion_decompose(&[transform]);
        assert!(near(compose(&decomp[0]), transform), "{transform:?} -> {:?}", compose(&decomp[0]));
    }
}

#[test]
fn the_parts_are_the_turn_the_move_and_the_scale() {
    let decomp = motion_decompose(&[Transform::translate(1.0, 2.0, 3.0) * Transform::scale(2.0, 3.0, 4.0)])[0];
    // No turn: the quaternion is 0 0 0 1.
    assert_eq!(decomp.x, [0.0, 0.0, 0.0, 1.0]);
    assert_eq!(decomp.y, [1.0, 2.0, 3.0, 2.0]);
    assert_eq!((decomp.z[3], decomp.w[3]), (3.0, 4.0));
}

#[test]
fn a_mirror_is_a_negative_scale_not_a_turn() {
    let mirror = Transform::scale(-1.0, 1.0, 1.0);
    let decomp = motion_decompose(&[mirror])[0];
    assert!(decomp.y[3] < 0.0 || decomp.z[3] < 0.0 || decomp.w[3] < 0.0);
    assert!(near(compose(&decomp), mirror));
}

#[test]
fn halfway_between_two_places_is_the_middle() {
    let motion = motion_decompose(&[Transform::translate(0.0, 0.0, 0.0), Transform::translate(2.0, 4.0, 6.0)]);
    assert!(near(motion_array_interpolate(&motion, 0.5), Transform::translate(1.0, 2.0, 3.0)));
    assert!(near(motion_array_interpolate(&motion, 0.0), Transform::IDENTITY));
}

#[test]
fn halfway_through_a_quarter_turn_is_an_eighth_turn() {
    let motion = motion_decompose(&[Transform::IDENTITY, quarter()]);
    let half = motion_array_interpolate(&motion, 0.5);
    let root = std::f32::consts::FRAC_1_SQRT_2;
    assert!(near(half, Transform { x: [root, -root, 0.0, 0.0], y: [root, root, 0.0, 0.0], z: [0.0, 0.0, 1.0, 0.0] }), "{half:?}");
}

#[test]
fn three_steps_are_blended_two_at_a_time() {
    let motion = motion_decompose(&[Transform::translate(0.0, 0.0, 0.0), Transform::translate(1.0, 0.0, 0.0), Transform::translate(1.0, 4.0, 0.0)]);
    assert!(near(motion_array_interpolate(&motion, 0.25), Transform::translate(0.5, 0.0, 0.0)));
    assert!(near(motion_array_interpolate(&motion, 0.75), Transform::translate(1.0, 2.0, 0.0)));
}

#[test]
fn a_step_with_no_size_takes_its_neighbour_s_turn() {
    let motion = motion_decompose(&[quarter(), Transform::scale(0.0, 0.0, 0.0)]);
    assert_eq!(motion[1].x, motion[0].x);
}
