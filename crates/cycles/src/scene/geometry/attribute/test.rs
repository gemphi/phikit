// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::scene::geometry::test::*;
use crate::util::Transform;

/// The offset the map gives the attribute with `id` on the mesh at `mesh`.
fn offset(rig: &Rig, mesh: usize, id: u64) -> i32 {
    find_attribute(&rig.dscene.attribute_map, rig.mesh[mesh].geometry.attr_map_offset, false, id).offset
}

#[test]
fn positions_go_to_their_own_array_and_normals_to_the_packed_one() {
    let mut rig = Rig::new(vec![square()]);
    rig.update();
    assert_eq!(rig.dscene.tri_vert, rig.mesh[0].geometry.position());
    assert_eq!(rig.dscene.attribute_normal.len(), 4);
    assert!(rig.dscene.attribute_float3.is_empty() && rig.dscene.attribute_float.is_empty());
}

#[test]
fn only_what_is_asked_for_goes_to_the_device() {
    let mut rig = Rig::new(vec![square()]);
    rig.mesh[0].add_attribute(std("UV"));
    rig.update();
    assert!(rig.dscene.attribute_float2.is_empty());
    // A UV pass asks every mesh for its UVs.
    rig.global.add_standard(std("UV"));
    rig.geometry.tag_update(GeometryManager::flag("UV_PASS_NEEDED"));
    rig.update();
    assert_eq!(rig.dscene.attribute_float2.len(), 6);
}

#[test]
fn each_type_has_its_array() {
    let mut rig = Rig::new(vec![square()]);
    let corner = AttributeElement::of("CORNER");
    rig.mesh[0].geometry.attribute.add("weight", AttributeType::Float, AttributeElement::of("VERTEX"), 4).center.float_mut().fill(0.5);
    rig.mesh[0].geometry.attribute.add("tint", AttributeType::Rgba, corner, 6);
    rig.mesh[0].add_attribute(std("VERTEX_COLOR"));
    rig.mesh[0].add_attribute(std("GENERATED_TRANSFORM")).center.matrix_mut()[0] = Transform::translate(1.0, 2.0, 3.0);
    ["weight", "tint"].iter().for_each(|name| rig.global.add_name(name));
    ["VERTEX_COLOR", "GENERATED_TRANSFORM"].iter().for_each(|short| rig.global.add_standard(std(short)));
    rig.update();
    let dscene = &rig.dscene;
    assert_eq!((dscene.attribute_float.as_slice(), dscene.attribute_uchar4.len()), (&[0.5f32; 4][..], 6));
    // The colour, then the matrix as its three rows.
    assert_eq!(dscene.attribute_float4.len(), 6 + 3);
    assert_eq!(dscene.attribute_float4[6..], [[1.0, 0.0, 0.0, 1.0], [0.0, 1.0, 0.0, 2.0], [0.0, 0.0, 1.0, 3.0]]);
    assert_eq!(offset(&rig, 0, AttributeIdMap::standard_id(std("GENERATED_TRANSFORM"))), 6);
}

#[test]
fn a_moving_attribute_s_steps_follow_its_centre() {
    let mut rig = Rig::new(vec![moving_square()]);
    rig.need_motion = MotionType::Blur;
    rig.update();
    // Centre, the step before, the step after.
    assert_eq!(rig.dscene.tri_vert.len(), 12);
    assert_eq!([0, 4, 8].map(|at| rig.dscene.tri_vert[at][2]), [0.0, -1.0, 1.0]);
    assert_eq!(rig.dscene.attribute_normal.len(), 12);
}

#[test]
fn a_corner_or_face_offset_allows_for_the_meshes_before() {
    let mut rig = Rig::new(vec![square(), square()]);
    rig.global.add_standard(std("UV"));
    rig.global.add_standard(std("PTEX_FACE_ID"));
    rig.mesh.iter_mut().for_each(|mesh| {
        mesh.add_attribute(std("UV"));
        mesh.add_attribute(std("PTEX_FACE_ID"));
    });
    rig.update();
    // The kernel adds a global triangle number: the second mesh's first
    // triangle is 2, its first corner 6, and its values start at 2 and 6.
    let (uv, face) = (AttributeIdMap::standard_id(std("UV")), AttributeIdMap::standard_id(std("PTEX_FACE_ID")));
    assert_eq!((offset(&rig, 0, uv), offset(&rig, 1, uv)), (0, 0));
    assert_eq!((offset(&rig, 0, face), offset(&rig, 1, face)), (0, 0));
    assert_eq!((rig.dscene.attribute_float2.len(), rig.dscene.attribute_float.len()), (12, 4));
    // A vertex is numbered within its mesh: its offset is where the values are.
    assert_eq!(offset(&rig, 1, AttributeIdMap::standard_id(std("POSITION"))), 4);
}

#[test]
fn the_objects_are_told_where_the_positions_and_normals_are() {
    let mut rig = Rig::new(vec![square(), square()]);
    rig.object.push(Object::new(1));
    rig.update();
    let object = &rig.dscene.object;
    assert_eq!([0, 1, 2].map(|index| object.int(index, "position_offset")), [0, 4, 4]);
    assert_eq!([0, 1, 2].map(|index| object.int(index, "normal_offset")), [0, 4, 4]);
    assert_eq!([0, 1, 2].map(|index| object.uint(index, "attribute_map_offset")), [0, 6, 6]);
    assert_eq!([0, 1, 2].map(|index| object.int(index, "numverts")), [4; 3]);
    assert_eq!(rig.mesh.iter().map(|mesh| mesh.geometry.index).collect::<Vec<_>>(), [0, 1]);
}
