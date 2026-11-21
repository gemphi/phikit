// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use super::*;
use crate::util::BoundBox;

const POINT: [[f32; 3]; 3] = [[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]];

#[test]
fn a_triangle_faces_the_way_its_corners_turn() {
    assert_eq!(Triangle { v: [0, 1, 2] }.compute_normal(&POINT), [0.0, 0.0, 1.0]);
    assert_eq!(Triangle { v: [0, 2, 1] }.compute_normal(&POINT), [0.0, 0.0, -1.0]);
}

#[test]
fn a_triangle_with_no_area_faces_along_x() {
    assert_eq!(Triangle { v: [0, 1, 1] }.compute_normal(&POINT), [1.0, 0.0, 0.0]);
}

#[test]
fn a_vertex_the_mesh_does_not_have_is_at_the_origin() {
    assert_eq!(Triangle { v: [1, 7, -1] }.corner(&POINT), [[2.0, 0.0, 0.0], [0.0; 3], [0.0; 3]]);
}

#[test]
fn a_corner_that_is_not_a_number_makes_it_invalid() {
    let odd = [[0.0; 3], [f32::NAN, 0.0, 0.0], [0.0, f32::INFINITY, 0.0]];
    assert!(Triangle { v: [0, 0, 0] }.valid(&odd));
    assert!(!Triangle { v: [0, 1, 0] }.valid(&odd) && !Triangle { v: [0, 0, 2] }.valid(&odd));
}

#[test]
fn a_box_grows_by_the_three_corners() {
    let mut bound = BoundBox::EMPTY;
    Triangle { v: [0, 1, 2] }.bound_grow(&POINT, &mut bound);
    assert_eq!((bound.min, bound.max), ([0.0; 3], [2.0, 2.0, 0.0]));
}

#[test]
fn between_two_motion_steps_the_corners_are_blended() {
    let mesh = moving_square();
    let position = mesh.geometry.position_attribute().expect("a mesh has positions");
    let triangle = mesh.triangle(0);
    // In the order of time: one lower, the centre, one higher.
    assert_eq!([0, 1, 2].map(|step| triangle.vert_for_step(position, step)[0][2]), [-1.0, 0.0, 1.0]);
    let height = |time: f32| triangle.motion_vert(position, 3, time).map(|corner| corner[2]);
    assert_eq!((height(0.0), height(0.25), height(0.5), height(1.0)), ([-1.0; 3], [-0.5; 3], [0.0; 3], [1.0; 3]));
}
