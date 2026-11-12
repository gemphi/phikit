// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use super::*;
use crate::util::vector::dot;
use crate::util::{PackedNormal, Transform};

#[test]
fn the_box_is_around_every_vertex() {
    let mut mesh = square();
    mesh.compute_bound();
    assert_eq!((mesh.geometry.bound.min, mesh.geometry.bound.max), ([0.0; 3], [1.0, 1.0, 0.0]));
}

#[test]
fn an_empty_mesh_is_a_point_at_the_origin() {
    let mut mesh = Mesh::new();
    mesh.compute_bound();
    assert_eq!((mesh.geometry.bound.min, mesh.geometry.bound.max), ([0.0; 3], [0.0; 3]));
}

#[test]
fn a_vertex_that_is_infinite_is_left_out_of_the_box() {
    let mut mesh = square();
    mesh.geometry.position_mut()[2] = [f32::INFINITY, 1.0, 0.0];
    mesh.compute_bound();
    assert!(mesh.geometry.bound.valid());
    assert_eq!(mesh.geometry.bound.max, [1.0, 1.0, 0.0]);
}

#[test]
fn a_blurred_mesh_s_box_is_around_every_motion_step() {
    let mut mesh = moving_square();
    mesh.compute_bound();
    assert_eq!((mesh.geometry.bound.min[2], mesh.geometry.bound.max[2]), (-1.0, 1.0));
    mesh.geometry.node.set("use_motion_blur", Value::Boolean(false));
    mesh.compute_bound();
    assert_eq!((mesh.geometry.bound.min[2], mesh.geometry.bound.max[2]), (0.0, 0.0));
}

#[test]
fn a_transform_moves_the_vertices_and_tags_them() {
    let mut mesh = square();
    mesh.geometry.node.clear_modified();
    mesh.geometry.attribute.clear_modified();
    mesh.apply_transform(&Transform::translate(1.0, 2.0, 3.0), false);
    assert_eq!((mesh.geometry.position()[0], mesh.geometry.position()[2]), ([1.0, 2.0, 3.0], [2.0, 3.0, 3.0]));
    assert!(mesh.geometry.position_is_modified() && mesh.geometry.node.is_modified());
}

#[test]
fn normals_turn_with_the_surface_not_with_the_scale() {
    let mut mesh = square();
    mesh.add_vertex_normal();
    // Stretched along x and turned so z goes to x: the surface now faces x.
    let turn = Transform { x: [0.0, 0.0, 1.0, 0.0], y: [0.0, 1.0, 0.0, 0.0], z: [-1.0, 0.0, 0.0, 0.0] } * Transform::scale(5.0, 1.0, 1.0);
    mesh.apply_transform(&turn, false);
    let normal = mesh.geometry.attribute.find_standard(std("VERTEX_NORMAL")).expect("added").center.normal()[0].decode();
    assert!(dot(normal, [1.0, 0.0, 0.0]) > 0.9999, "{normal:?}");
    assert_eq!(mesh.geometry.transform_normal, turn.transposed_inverse());
}

#[test]
fn motion_steps_move_only_when_asked() {
    let height = |apply: bool| {
        let mut mesh = moving_square();
        mesh.apply_transform(&Transform::translate(0.0, 0.0, 10.0), apply);
        let position = mesh.geometry.position_attribute().expect("a mesh has positions");
        [0, 1, 2].map(|step| position.data(step).float3()[0][2])
    };
    assert_eq!((height(false), height(true)), ([10.0, -1.0, 1.0], [10.0, 9.0, 11.0]));
}

#[test]
fn corner_normals_turn_too() {
    let mut mesh = square();
    mesh.add_attribute(std("CORNER_NORMAL")).center.normal_mut().fill(PackedNormal::encode([0.0, 0.0, 1.0]));
    mesh.apply_transform(&Transform::scale(1.0, 1.0, -1.0), false);
    let normal = mesh.geometry.attribute.find_standard(std("CORNER_NORMAL")).expect("added").center.normal()[5].decode();
    assert!(dot(normal, [0.0, 0.0, -1.0]) > 0.9999, "{normal:?}");
}
