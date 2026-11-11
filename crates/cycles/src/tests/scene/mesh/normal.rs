// SPDX-License-Identifier: Apache-2.0
use super::*;
use crate::util::PackedNormal;
use crate::util::vector::{dot, normalize};

fn vertex_normal(mesh: &Mesh, step: usize) -> Vec<[f32; 3]> {
    let normal = mesh.geometry.attribute.find_standard(std("VERTEX_NORMAL")).expect("added");
    normal.data(step).normal().iter().map(|normal| normal.decode()).collect()
}

fn along(normal: [f32; 3], expect: [f32; 3]) -> bool {
    dot(normal, normalize(expect)) > 0.9999
}

#[test]
fn a_flat_mesh_s_vertices_all_face_the_way_it_does() {
    let mut mesh = square();
    mesh.add_vertex_normal();
    assert!(vertex_normal(&mesh, 0).into_iter().all(|normal| along(normal, [0.0, 0.0, 1.0])));
}

#[test]
fn a_vertex_between_two_faces_faces_between_them() {
    // Two triangles that meet at a right angle along the edge from 0 to 1.
    let mut mesh = mesh_of(&[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]], &[[0, 1, 2], [0, 3, 1]]);
    mesh.add_vertex_normal();
    let normal = vertex_normal(&mesh, 0);
    assert!(along(normal[0], [0.0, 1.0, 1.0]) && along(normal[1], [0.0, 1.0, 1.0]), "{normal:?}");
    assert!(along(normal[2], [0.0, 0.0, 1.0]) && along(normal[3], [0.0, 1.0, 0.0]), "{normal:?}");
}

#[test]
fn a_mirrored_mesh_s_normals_are_turned_round() {
    let mut mesh = square();
    mesh.geometry.transform_negative_scaled = true;
    mesh.add_vertex_normal();
    assert!(vertex_normal(&mesh, 0).into_iter().all(|normal| along(normal, [0.0, 0.0, -1.0])));
}

#[test]
fn normals_the_mesh_came_with_are_kept() {
    let mut mesh = square();
    let given = PackedNormal::encode([1.0, 0.0, 0.0]);
    mesh.add_attribute(std("VERTEX_NORMAL")).center.normal_mut().fill(given);
    mesh.add_vertex_normal();
    assert_eq!(mesh.geometry.attribute.find_standard(std("VERTEX_NORMAL")).expect("given").center.normal(), [given; 4]);
}

#[test]
fn a_mesh_with_corner_normals_gets_no_vertex_normals() {
    let mut mesh = square();
    mesh.add_attribute(std("CORNER_NORMAL"));
    mesh.add_vertex_normal();
    assert!(mesh.geometry.attribute.find_standard(std("VERTEX_NORMAL")).is_none());
}

#[test]
fn a_blurred_mesh_drops_corner_normals_that_do_not_move() {
    let mut mesh = moving_square();
    mesh.add_attribute(std("CORNER_NORMAL"));
    mesh.add_vertex_normal();
    assert!(mesh.geometry.attribute.find_standard(std("CORNER_NORMAL")).is_none());
    assert!(mesh.geometry.attribute.find_standard(std("VERTEX_NORMAL")).is_some());
}

#[test]
fn a_blurred_mesh_gets_normals_at_every_motion_step() {
    let mut mesh = moving_square();
    // Tilt the step before: its vertex 2 and 3 are lifted, so it faces back along y.
    let position = mesh.geometry.attribute.find_standard_mut(std("POSITION")).expect("a mesh has positions");
    position.data_mut(1).float3_mut()[2][2] = 0.0;
    position.data_mut(1).float3_mut()[3][2] = 0.0;
    mesh.add_vertex_normal();
    let normal = mesh.geometry.attribute.find_standard(std("VERTEX_NORMAL")).expect("added");
    assert_eq!(normal.motion.len(), 2);
    assert!(vertex_normal(&mesh, 0).into_iter().all(|normal| along(normal, [0.0, 0.0, 1.0])));
    assert!(vertex_normal(&mesh, 1).into_iter().all(|normal| along(normal, [0.0, -1.0, 1.0])));
    assert!(vertex_normal(&mesh, 2).into_iter().all(|normal| along(normal, [0.0, 0.0, 1.0])));
}

#[test]
fn an_empty_mesh_gets_no_normals() {
    let mut mesh = Mesh::new();
    mesh.add_vertex_normal();
    assert!(mesh.geometry.attribute.find_standard(std("VERTEX_NORMAL")).is_none());
}

#[test]
fn the_faces_to_subdivide_get_normals_of_their_own() {
    let mut mesh = Mesh::new();
    mesh.resize_subd_face(1, 4);
    mesh.geometry.node.set("subd_num_corners", Value::IntArray(vec![4]));
    mesh.geometry.node.set("subd_face_corners", Value::IntArray(vec![0, 1, 2, 3]));
    let position = mesh.subd_attribute.add_standard(std("POSITION"), "", 4);
    position.center.float3_mut().copy_from_slice(&[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]]);
    assert_eq!(mesh.subd_face_normal(&mesh.subd_face(0)), [0.0, 0.0, 1.0]);
    // The mesh itself has the same four vertices before it is diced.
    mesh.resize_mesh(4, 0);
    mesh.add_vertex_normal();
    let normal = mesh.subd_attribute.find_standard(std("VERTEX_NORMAL")).expect("added");
    assert!(normal.center.normal().iter().all(|normal| along(normal.decode(), [0.0, 0.0, 1.0])));
}
