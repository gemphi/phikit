// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Tests of this module. It also holds the meshes the others, and the tests of
//! the objects and the geometry manager, are run on.


pub(crate) use crate::graph::Value;
pub(crate) use crate::scene::*;

pub(crate) fn std(short: &str) -> AttributeStandard {
    AttributeStandard::of(short)
}

/// A mesh of these vertices and triangles.
pub(crate) fn mesh_of(point: &[[f32; 3]], triangle: &[[i32; 3]]) -> Mesh {
    let mut mesh = Mesh::new();
    mesh.resize_mesh(point.len(), triangle.len());
    mesh.geometry.position_mut().copy_from_slice(point);
    mesh.geometry.node.set("triangles", Value::IntArray(triangle.iter().flatten().copied().collect()));
    mesh
}

/// A square of side 1 in the plane of x and y, two triangles that face
/// along z.
pub(crate) fn square() -> Mesh {
    mesh_of(&[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]], &[[0, 1, 2], [0, 2, 3]])
}

/// The square with a motion step before and after, one unit lower and
/// one higher along z.
pub(crate) fn moving_square() -> Mesh {
    let mut mesh = square();
    mesh.geometry.node.set("motion_steps", Value::Uint(3));
    mesh.geometry.node.set("use_motion_blur", Value::Boolean(true));
    let center = mesh.geometry.position().to_vec();
    let position = mesh.geometry.attribute.find_standard_mut(std("POSITION")).expect("a mesh has positions");
    position.add_motion(3);
    for (step, lift) in [(1, -1.0), (2, 1.0)] {
        let moved: Vec<[f32; 3]> = center.iter().map(|point| [point[0], point[1], point[2] + lift]).collect();
        position.data_mut(step).float3_mut().copy_from_slice(&moved);
    }
    mesh
}

#[test]
fn a_new_mesh_is_empty_but_for_its_position_attribute() {
    let mesh = Mesh::new();
    assert_eq!((mesh.vert_count(), mesh.triangle_count(), mesh.subd_face_count()), (0, 0, 0));
    assert_eq!(mesh.geometry.attribute.attribute.len(), 1);
    assert_eq!((mesh.geometry.kind, mesh.geometry.position_attribute().map(|position| position.std)), (GeometryKind::Mesh, Some(std("POSITION"))));
    assert!(mesh.geometry.node.is_modified() && !mesh.is_subdivided());
}

#[test]
fn resizing_makes_room_in_the_sockets_and_in_every_attribute() {
    let mut mesh = Mesh::new();
    mesh.add_attribute(std("UV"));
    mesh.add_attribute(std("VERTEX_NORMAL"));
    mesh.resize_mesh(4, 2);
    assert_eq!((mesh.vert_count(), mesh.triangle_count()), (4, 2));
    let node = &mesh.geometry.node;
    assert_eq!((node.int_array("triangles").len(), node.int_array("shader").len(), node.boolean_array("smooth").len()), (6, 2, 2));
    let size = |short: &str| mesh.geometry.attribute.find_standard(std(short)).map(|attribute| attribute.size());
    // A UV for each corner, a normal for each vertex.
    assert_eq!((size("UV"), size("VERTEX_NORMAL")), (Some(6), Some(4)));
}

#[test]
fn an_attribute_added_later_fits_the_mesh_as_it_is() {
    let mut mesh = square();
    assert_eq!((mesh.add_attribute(std("UV")).size(), mesh.add_attribute(std("PTEX_FACE_ID")).size(), mesh.add_attribute(std("GENERATED_TRANSFORM")).size()), (6, 2, 1));
    assert_eq!(mesh.triangle(1), Triangle { v: [0, 2, 3] });
    assert_eq!(mesh.triangle(9), Triangle { v: [0; 3] });
}

#[test]
fn the_number_of_values_follows_the_element_and_the_primitive() {
    let size = MeshSize { vert: 10, triangle: 4, subd_base_vert: 8, subd_face: 3, subd_corner: 12 };
    let of = |short: &str, prim: AttributePrimitive| size.element_size(AttributeElement::of(short), prim);
    let (geometry, subd) = (AttributePrimitive::Geometry, AttributePrimitive::Subd);
    assert_eq!([of("VERTEX", geometry), of("VERTEX_NORMAL", geometry), of("FACE", geometry), of("CORNER", geometry), of("CORNER_BYTE", geometry)], [10, 10, 4, 12, 12]);
    assert_eq!([of("VERTEX", subd), of("FACE", subd), of("CORNER_NORMAL", subd)], [8, 3, 12]);
    assert_eq!([of("MESH", geometry), of("OBJECT", subd), of("CURVE", geometry), of("NONE", geometry)], [1, 1, 0, 0]);
}

#[test]
fn clearing_leaves_the_shaders_only_when_asked() {
    let mut mesh = square();
    mesh.add_attribute(std("UV"));
    mesh.geometry.set_used_shader(vec![2]);
    mesh.geometry.transform_applied = true;
    mesh.geometry.node.clear_modified();
    mesh.clear(true);
    assert_eq!((mesh.vert_count(), mesh.triangle_count(), mesh.geometry.attribute.attribute.len()), (0, 0, 1));
    assert!(mesh.geometry.node.is_modified() && !mesh.geometry.transform_applied);
    assert_eq!(mesh.geometry.used_shader, [2]);
    mesh.clear(false);
    assert!(mesh.geometry.used_shader.is_empty());
}

#[test]
fn a_face_to_subdivide_is_read_out_of_the_sockets() {
    let mut mesh = Mesh::new();
    mesh.resize_subd_face(2, 7);
    assert_eq!((mesh.subd_face_count(), mesh.size().subd_corner), (2, 7));
    mesh.geometry.node.set("subd_start_corner", Value::IntArray(vec![0, 4]));
    mesh.geometry.node.set("subd_num_corners", Value::IntArray(vec![4, 3]));
    mesh.geometry.node.set("subd_smooth", Value::BooleanArray(vec![false, true]));
    let (quad, triangle) = (mesh.subd_face(0), mesh.subd_face(1));
    assert_eq!((quad.is_quad(), quad.ptex_face_count(), triangle.start_corner, triangle.smooth, triangle.ptex_face_count()), (true, 1, 4, true, 3));
}

#[test]
fn a_crease_is_two_vertices_and_how_sharp() {
    let mut mesh = Mesh::new();
    mesh.add_edge_crease(1, 2, 0.5);
    mesh.add_vertex_crease(3, 1.0);
    let node = &mesh.geometry.node;
    assert_eq!((node.int_array("subd_creases_edge"), node.float_array("subd_creases_weight")), (&[1, 2][..], &[0.5][..]));
    assert_eq!((node.int_array("subd_vert_creases"), node.float_array("subd_vert_creases_weight")), (&[3][..], &[1.0][..]));
}

#[test]
fn only_a_subdivided_mesh_whose_dicing_changed_is_tessellated() {
    let mut mesh = square();
    assert!(!mesh.need_tesselation());
    mesh.geometry.node.set_enum("subdivision_type", "catmull_clark");
    assert!(mesh.is_subdivided() && mesh.need_tesselation());
    mesh.geometry.node.clear_modified();
    mesh.geometry.attribute.clear_modified();
    assert!(!mesh.need_tesselation());
    mesh.geometry.node.set("subd_dicing_rate", Value::Float(0.5));
    assert!(mesh.need_tesselation());
}
