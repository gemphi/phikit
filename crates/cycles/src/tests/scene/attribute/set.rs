// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use super::{element, std};
use crate::scene::*;

fn set() -> AttributeSet {
    let mut set = AttributeSet::new(GeometryKind::Mesh, AttributePrimitive::Geometry);
    set.clear_modified();
    set
}

#[test]
fn a_new_set_counts_every_array_as_changed() {
    let set = AttributeSet::new(GeometryKind::Mesh, AttributePrimitive::Geometry);
    assert!(set.modified(KernelDataType::Float) && set.modified(KernelDataType::Normal));
}

#[test]
fn adding_twice_gives_the_attribute_there_is() {
    let mut set = set();
    set.add("weight", AttributeType::Float, element("VERTEX"), 3).center.float_mut()[0] = 1.0;
    assert!(set.modified(KernelDataType::Float) && !set.modified(KernelDataType::Float3));
    set.clear_modified();
    assert_eq!(set.add("weight", AttributeType::Float, element("VERTEX"), 3).center.float()[0], 1.0);
    assert_eq!(set.attribute.len(), 1);
    assert!(!set.modified(KernelDataType::Float));
}

#[test]
fn another_type_under_the_same_name_takes_the_place() {
    let mut set = set();
    set.add("weight", AttributeType::Float, element("VERTEX"), 3);
    set.clear_modified();
    let added = set.add("weight", AttributeType::Float2, element("CORNER"), 6);
    assert_eq!((added.kind, added.size()), (AttributeType::Float2, 6));
    assert_eq!(set.attribute.len(), 1);
    // Both arrays changed: one lost an attribute, one gained it.
    assert!(set.modified(KernelDataType::Float) && set.modified(KernelDataType::Float2));
}

#[test]
fn a_standard_attribute_has_its_own_name_type_and_element() {
    let mut set = set();
    let position = set.add_standard(std("POSITION"), "", 4);
    assert_eq!((position.name.as_str(), position.kind, position.element, position.size()), ("P", AttributeType::Point, element("VERTEX"), 4));
    let uv = set.add_standard(std("UV"), "UVMap", 6);
    assert_eq!((uv.name.as_str(), uv.std), ("UVMap", std("UV")));
    assert!(set.find("UVMap").is_some() && set.find_standard(std("UV")).is_some() && set.find("uv").is_none());
}

#[test]
fn corner_normals_take_the_place_of_vertex_normals() {
    // Both are named N, so a mesh has one or the other.
    let mut set = set();
    set.add_standard(std("VERTEX_NORMAL"), "", 4);
    set.add_standard(std("CORNER_NORMAL"), "", 6);
    assert_eq!(set.attribute.len(), 1);
    assert!(set.find_standard(std("VERTEX_NORMAL")).is_none() && set.find_standard(std("CORNER_NORMAL")).is_some());
}

#[test]
fn an_attribute_is_found_and_removed_by_name_or_by_standard() {
    let mut set = set();
    set.add_standard(std("POSITION"), "", 2);
    set.add("weight", AttributeType::Float, element("VERTEX"), 2);
    set.find_mut("weight").expect("added").center.float_mut()[1] = 2.0;
    assert_eq!(set.find("weight").expect("added").center.float(), [0.0, 2.0]);
    set.clear_modified();
    set.remove("weight");
    set.remove("nothing");
    assert!(set.find("weight").is_none() && set.modified(KernelDataType::Float) && !set.modified(KernelDataType::Float3));
    set.remove_standard(std("POSITION"));
    assert!(set.attribute.is_empty() && set.modified(KernelDataType::Float3));
}

#[test]
fn a_request_finds_by_name_or_by_standard() {
    let mut set = set();
    set.add_standard(std("UV"), "UVMap", 6);
    let mut request = AttributeRequestSet::new();
    request.add_name("UVMap");
    request.add_standard(std("UV"));
    request.add_name("uv");
    let found = request.request.iter().map(|request| set.find_request(request).is_some()).collect::<Vec<bool>>();
    assert_eq!(found, [true, true, false]);
}

#[test]
fn resizing_asks_how_many_of_each_element_there_are() {
    let mut set = set();
    set.add_standard(std("POSITION"), "", 0);
    set.add_standard(std("UV"), "", 0);
    set.resize(|on| match on == element("VERTEX") {
        true => 4,
        false => 6,
    });
    assert_eq!((set.find("P").expect("added").size(), set.find("uv").expect("added").size()), (4, 6));
}

#[test]
fn a_copy_has_the_room_but_not_the_values() {
    let mut from = Attribute::new("P", AttributeType::Point, element("VERTEX"), 2);
    from.std = std("POSITION");
    from.add_motion(3);
    from.center.float3_mut()[0] = [1.0; 3];
    let mut set = set();
    let copy = set.copy(&from, 2, 3);
    assert_eq!((copy.std, copy.size(), copy.motion.len(), copy.center.float3()[0]), (std("POSITION"), 2, 2, [0.0; 3]));
}

#[test]
fn updating_takes_the_new_set_s_attributes_and_drops_the_rest() {
    let mut old = set();
    old.add_standard(std("POSITION"), "", 2);
    old.add("weight", AttributeType::Float, element("VERTEX"), 2);
    old.clear_modified();

    let mut same = set();
    same.add_standard(std("POSITION"), "", 2);
    same.add_standard(std("UV"), "", 6);
    // The positions did not change: the geometry need not be tagged.
    assert!(!old.update(same));
    assert!(old.find("weight").is_none() && old.find("uv").is_some());
    assert!(!old.find("P").expect("kept").modified);

    let mut moved = set();
    moved.add_standard(std("POSITION"), "", 2).center.float3_mut()[0] = [1.0; 3];
    assert!(old.update(moved));
    assert_eq!(old.find("P").expect("kept").center.float3()[0], [1.0; 3]);
    assert!(old.find("uv").is_none());
}

#[test]
fn clearing_can_keep_the_generated_transform() {
    let mut set = set();
    set.add_standard(std("POSITION"), "", 2);
    set.add_standard(std("GENERATED_TRANSFORM"), "", 1);
    set.clear(true);
    assert_eq!(set.attribute.iter().map(|attribute| attribute.std).collect::<Vec<_>>(), [std("GENERATED_TRANSFORM")]);
    set.clear(false);
    assert!(set.attribute.is_empty());
}
