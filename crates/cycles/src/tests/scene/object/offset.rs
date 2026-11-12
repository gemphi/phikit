// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use super::*;
use crate::kernel::constant;

#[test]
fn an_object_whose_offsets_are_right_is_left_alone() {
    let mut rig = Rig::new(vec![square(), square()]);
    rig.update();
    assert!(!ObjectManager::device_update_geom_offset(&mut rig.dscene, &rig.object, &rig.mesh));
    // Told again after its mesh moved in the map.
    rig.mesh[1].geometry.attr_map_offset = 0;
    assert!(ObjectManager::device_update_geom_offset(&mut rig.dscene, &rig.object, &rig.mesh));
    assert_eq!((rig.dscene.object.uint(1, "attribute_map_offset"), rig.dscene.object.int(1, "position_offset")), (0, 0));
}

#[test]
fn corner_normals_are_looked_for_before_vertex_normals() {
    let mut rig = Rig::new(vec![square(), square()]);
    rig.mesh[1].add_attribute(std("CORNER_NORMAL"));
    rig.update();
    // The first mesh's four vertex normals, then the second's six corner
    // normals, which a kernel reads from its global corner 6.
    assert_eq!((rig.dscene.attribute_normal.len(), rig.dscene.object.int(0, "normal_offset"), rig.dscene.object.int(1, "normal_offset")), (10, 0, 4 - 6));
}

#[test]
fn an_empty_mesh_has_no_positions_to_point_at() {
    let mut rig = Rig::new(vec![Mesh::new()]);
    rig.update();
    let not_found = constant("ATTR_STD_NOT_FOUND").expect("in data/constant.tsv") as i32;
    assert_eq!((rig.dscene.object.int(0, "normal_offset"), rig.dscene.object.int(0, "numverts")), (not_found, 0));
}
