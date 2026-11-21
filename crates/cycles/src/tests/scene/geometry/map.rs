// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use super::*;
use crate::kernel::{RecordArray, constant};

fn not_found() -> i32 {
    constant("ATTR_STD_NOT_FOUND").expect("in data/constant.tsv") as i32
}

/// A map written by hand: (id, element, offset) for each entry, two
/// entries to an attribute.
fn map(entry: &[(u64, u32, i32)]) -> RecordArray {
    let mut map = RecordArray::of("AttributeMap").expect("in data/record.tsv");
    map.alloc(entry.len() * 2);
    for (place, (id, element, offset)) in entry.iter().enumerate() {
        map.set_uint64(place * 2, "id", *id);
        map.set_uint(place * 2, "element", *element);
        map.set_int(place * 2, "offset", *offset);
    }
    map
}

#[test]
fn an_attribute_is_found_by_walking_the_geometry_s_entries() {
    let vertex = AttributeElement::of("VERTEX");
    let map = map(&[(1, vertex.0, 40), (5, AttributeElement::of("CORNER").0, 12), (0, 0, 0)]);
    assert_eq!(find_attribute(&map, 0, false, 5), AttributeDescriptor { element: AttributeElement::of("CORNER"), offset: 12 });
    assert_eq!(find_attribute(&map, 0, false, 1).offset, 40);
    // Not asked for by any shader: the walk reaches the end.
    assert_eq!(find_attribute(&map, 0, false, 9), AttributeDescriptor { element: AttributeElement::NONE, offset: not_found() });
}

#[test]
fn an_entry_with_no_id_and_an_element_is_a_link_to_another_map() {
    // An object's own map at entry 0, which goes on in its mesh's at entry 2.
    let map = map(&[(40, AttributeElement::of("OBJECT").0, 3), (0, 1, 4), (5, AttributeElement::of("CORNER").0, 12), (0, 0, 0)]);
    assert_eq!(find_attribute(&map, 0, false, 40).offset, 3);
    assert_eq!(find_attribute(&map, 0, false, 5).offset, 12);
    assert_eq!(find_attribute(&map, 0, false, 6).offset, not_found());
}

#[test]
fn off_a_primitive_only_an_attribute_of_the_whole_is_found() {
    let map = map(&[(1, AttributeElement::of("VERTEX").0, 40), (12, AttributeElement::of("MESH").0, 8), (0, 0, 0)]);
    assert_eq!((find_attribute(&map, 0, true, 1).offset, find_attribute(&map, 0, true, 12).offset), (not_found(), 8));
}

#[test]
fn an_attribute_the_mesh_lacks_is_in_the_map_as_not_there() {
    let map = map(&[(5, 0, 0), (0, 0, 0)]);
    assert_eq!(find_attribute(&map, 0, false, 5).offset, not_found());
    assert_eq!(find_attribute(&map, 99, false, 5).offset, not_found());
}

#[test]
fn an_update_writes_each_mesh_s_entries_one_mesh_after_another() {
    let mut rig = Rig::new(vec![square(), square()]);
    rig.shader[0].attribute.add_name("weight");
    rig.mesh[1].geometry.set_used_shader(vec![0]);
    rig.update();
    // The first mesh: positions, normals and the end. The second: the
    // shader's attribute, which it lacks, then positions, normals, the end.
    assert_eq!((rig.mesh[0].geometry.attr_map_offset, rig.mesh[1].geometry.attr_map_offset, rig.dscene.attribute_map.len()), (0, 6, 14));
    let map = &rig.dscene.attribute_map;
    let id = |index: usize| map.uint64(index, "id");
    assert_eq!([id(0), id(2), id(4)], [1, 3, 0]);
    assert_eq!([id(6), id(8), id(10), id(12)], [35, 1, 3, 0]);
    assert_eq!((map.uint(0, "type"), map.uint(0, "element"), map.uint(2, "element")), (2, AttributeElement::of("VERTEX").0, AttributeElement::of("VERTEX_NORMAL").0));
    assert_eq!(find_attribute(map, 6, false, 35).offset, not_found());
    // The second mesh's positions follow the first's four.
    assert_eq!((find_attribute(map, 0, false, 1).offset, find_attribute(map, 6, false, 1).offset), (0, 4));
}
