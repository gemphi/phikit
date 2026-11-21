// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/geometry_attributes.cpp`
//! (`GeometryManager::update_svm_attributes`, `emit_attribute_map_entry`,
//! `emit_attribute_map_terminator`) and `kernel/geom/attribute.h`
//! (`find_attr_offset`, `find_attribute`): the attribute map, in which a
//! shader looks up where an attribute of the geometry it shades is.
//!
//! A geometry's entries follow one another, `ATTR_PRIM_TYPES` apart, and
//! end at an entry with no id. Not ported: the entries OSL adds, and an
//! object's own map, which waits for object attributes.

use super::{AttributeIdMap, GeometryManager};
use crate::kernel::{RecordArray, constant};
use crate::scene::{AttributeDescriptor, AttributeElement, AttributePrimitive, AttributeRequestSet, AttributeStandard, DeviceScene, Mesh, Object};

impl GeometryManager {
    /// `update_svm_attributes`: write the map from the requests, each
    /// already told where its attribute was packed.
    pub(super) fn update_svm_attribute(dscene: &mut DeviceScene, mesh: &mut [Mesh], object: &mut [Object], request: &[AttributeRequestSet], id: &mut AttributeIdMap) {
        let stride = AttributePrimitive::count();
        let mut size = 0;
        for (mesh, request) in mesh.iter_mut().zip(request) {
            mesh.geometry.attr_map_offset = size;
            size += (request.len() + 1) * stride;
        }
        // No object has a map of its own: it reads its geometry's.
        object.iter_mut().for_each(|object| object.attr_map_offset = 0);
        let map = &mut dscene.attribute_map;
        map.alloc(size);
        for (mesh, request) in mesh.iter().zip(request) {
            for (place, request) in request.request.iter().enumerate() {
                let index = mesh.geometry.attr_map_offset + place * stride;
                let unique = match request.std == AttributeStandard::NONE {
                    true => id.id(&request.name),
                    false => AttributeIdMap::standard_id(request.std),
                };
                map.set_uint64(index, "id", unique);
                map.set_uint(index, "element", request.desc.element.0);
                map.set_int(index, "offset", request.desc.offset);
                map.set_uint(index, "type", request.kind.node_attribute() as u32);
            }
            // The entry after the last stays zero: no id, no link, the end.
        }
    }
}

/// `find_attribute`: where the attribute with `id` is, looked up as a
/// kernel does from the map entry at `offset`. With `prim_none` (the
/// lookup is not for a primitive) only an attribute of the whole object,
/// mesh or volume is found. An attribute that is not there has the
/// offset `ATTR_STD_NOT_FOUND`.
pub fn find_attribute(map: &RecordArray, offset: usize, prim_none: bool, id: u64) -> AttributeDescriptor {
    let not_found = AttributeDescriptor { element: AttributeElement::NONE, offset: constant("ATTR_STD_NOT_FOUND").unwrap_or(0) as i32 };
    let stride = AttributePrimitive::count();
    let mut at = offset;
    // `find_attr_offset`. No walk is longer than the map.
    for _ in 0..=map.len() {
        let end = at >= map.len();
        let entry = match end {
            true => 0,
            false => map.uint64(at, "id"),
        };
        match (end, entry == id, entry == 0, !end && map.uint(at, "element") != 0) {
            (true, _, _, _) | (false, false, true, false) => return not_found,
            (false, true, _, _) => break,
            // An entry with no id and an element is a link to another map.
            (false, false, true, true) => at = map.int(at, "offset") as usize,
            (false, false, false, _) => at += stride,
        }
    }
    if at >= map.len() {
        return not_found;
    }
    let element = AttributeElement(map.uint(at, "element"));
    let whole = ["MESH", "VOXEL", "OBJECT"].iter().any(|short| element.has(short));
    match (prim_none && !whole) || element == AttributeElement::NONE {
        true => not_found,
        false => AttributeDescriptor { element, offset: map.int(at, "offset") },
    }
}
