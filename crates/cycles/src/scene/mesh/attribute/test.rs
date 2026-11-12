// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::scene::mesh::test::*;
use crate::util::PackedNormal;

#[test]
fn a_motion_step_can_be_made_to_stand_still_at_the_centre() {
    let mut mesh = moving_square();
    mesh.add_vertex_normal();
    mesh.copy_center_to_motion_step(0);
    let position = mesh.geometry.position_attribute().expect("a mesh has positions");
    assert_eq!([0, 1, 2].map(|step| position.data(step).float3()[0][2]), [0.0, 0.0, 1.0]);
    let normal = mesh.geometry.attribute.find_standard(std("VERTEX_NORMAL")).expect("added");
    assert_eq!(normal.data(1), normal.data(0));
}

#[test]
fn the_tiles_of_a_uv_map_are_found_by_its_name_or_as_the_standard_one() {
    let mut mesh = square();
    mesh.add_attribute(std("UV")).center.float2_mut().fill([0.5, 0.5]);
    let size = mesh.size().triangle * 3;
    let other = mesh.geometry.attribute.add("second", AttributeType::Float2, AttributeElement::of("CORNER"), size);
    other.center.float2_mut().fill([1.5, 0.5]);
    assert_eq!(mesh.uv_tile_list("").into_iter().collect::<Vec<i32>>(), [1001]);
    assert_eq!(mesh.uv_tile_list("second").into_iter().collect::<Vec<i32>>(), [1002]);
    assert!(mesh.uv_tile_list("third").is_empty());
}

#[test]
fn generated_coordinates_are_the_positions_when_they_are_needed() {
    let mut mesh = square();
    mesh.update_generated(false);
    assert!(mesh.geometry.attribute.find_standard(std("GENERATED")).is_none());
    mesh.update_generated(true);
    let generated = mesh.geometry.attribute.find_standard(std("GENERATED")).expect("added");
    assert_eq!(generated.center.float3(), mesh.geometry.position());
    // Ones the mesh came with are kept.
    mesh.geometry.position_mut()[0] = [9.0; 3];
    mesh.update_generated(true);
    assert_eq!(mesh.geometry.attribute.find_standard(std("GENERATED")).expect("kept").center.float3()[0], [0.0; 3]);
}

#[test]
fn an_empty_mesh_gets_no_generated_coordinates() {
    let mut mesh = Mesh::new();
    mesh.update_generated(true);
    assert!(mesh.geometry.attribute.find_standard(std("GENERATED")).is_none());
}

#[test]
fn the_positions_and_normals_before_displacement_are_kept_when_needed() {
    let mut mesh = square();
    mesh.add_vertex_normal();
    mesh.add_undisplaced(false, false);
    assert_eq!(mesh.geometry.attribute.attribute.len(), 2);
    mesh.add_undisplaced(true, true);
    let find = |short: &str| mesh.geometry.attribute.find_standard(std(short)).expect("added");
    assert_eq!(find("POSITION_UNDISPLACED").center.float3(), mesh.geometry.position());
    assert_eq!((find("NORMAL_UNDISPLACED").name.as_str(), find("NORMAL_UNDISPLACED").element), ("undisplaced_N", AttributeElement::of("VERTEX_NORMAL")));
    assert_eq!(find("NORMAL_UNDISPLACED").center, find("VERTEX_NORMAL").center);
}

#[test]
fn undisplaced_normals_are_the_corner_normals_when_the_mesh_has_them() {
    let mut mesh = square();
    mesh.add_attribute(std("CORNER_NORMAL")).center.normal_mut().fill(PackedNormal::encode([0.0, 1.0, 0.0]));
    mesh.add_undisplaced(false, true);
    let kept = mesh.geometry.attribute.find_standard(std("NORMAL_UNDISPLACED")).expect("added");
    assert_eq!((kept.element, kept.size()), (AttributeElement::of("CORNER_NORMAL"), 6));
}
