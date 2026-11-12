// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::scene::mesh::test::*;
use crate::kernel::flag;

/// Three shaders of the scene, numbered 10, 11 and 12 on the device.
fn shader() -> Vec<Shader> {
    [10, 11, 12].map(|id| Shader { id, ..Shader::default() }).to_vec()
}

fn packed(mesh: &Mesh) -> Vec<u32> {
    let mut out = vec![0; mesh.triangle_count()];
    mesh.pack_shader(&shader(), &Shader { id: 99, ..Shader::default() }, &mut out);
    out
}

#[test]
fn a_triangle_names_its_shader_by_the_device_s_number_and_casts_shadows() {
    let mut mesh = square();
    // The mesh's slots 0 and 1 are the scene's shaders 2 and 0.
    mesh.geometry.set_used_shader(vec![2, 0]);
    mesh.geometry.node.set("shader", Value::IntArray(vec![0, 1]));
    let cast = flag("SHADER_CAST_SHADOW");
    assert_eq!(packed(&mesh), [12 | cast, 10 | cast]);
}

#[test]
fn a_smooth_triangle_says_so_in_its_shader() {
    let mut mesh = square();
    mesh.geometry.set_used_shader(vec![1]);
    mesh.geometry.node.set("smooth", Value::BooleanArray(vec![false, true]));
    let (cast, smooth) = (flag("SHADER_CAST_SHADOW"), flag("SHADER_SMOOTH_NORMAL"));
    assert_eq!(packed(&mesh), [11 | cast, 11 | cast | smooth]);
    // Corner normals are always blended, whatever the triangle says.
    mesh.add_attribute(std("CORNER_NORMAL"));
    assert_eq!(packed(&mesh), [11 | cast | smooth; 2]);
}

#[test]
fn a_shader_the_mesh_does_not_have_is_the_default_surface() {
    let mut mesh = square();
    mesh.geometry.set_used_shader(vec![1]);
    mesh.geometry.node.set("shader", Value::IntArray(vec![5, -1]));
    assert_eq!(packed(&mesh), [99 | flag("SHADER_CAST_SHADOW"); 2]);
}

#[test]
fn the_corners_are_packed_three_to_a_triangle() {
    let mesh = square();
    let mut out = vec![[9; 3]; 2];
    mesh.pack_triangle(&mut out);
    assert_eq!(out, [[0, 1, 2], [0, 2, 3]]);
}

#[test]
fn a_mesh_is_a_motion_triangle_only_when_blurred_and_moving() {
    let (still, mut moving) = (square(), moving_square());
    assert!(!still.has_motion_blur() && moving.has_motion_blur());
    assert_eq!((still.primitive_type(), moving.primitive_type()), (1, 65));
    moving.geometry.node.set("use_motion_blur", Value::Boolean(false));
    assert!(!moving.has_motion_blur());
}
