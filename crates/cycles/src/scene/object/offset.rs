// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/object.cpp`
//! (`ObjectManager::device_update_geom_offsets`): once the attributes are
//! packed, each object is told where its attribute map is and, as these
//! are read so often, where its positions and normals are.

use super::{Object, ObjectManager};
use crate::kernel::constant;
use crate::scene::{AttributeStandard, DeviceScene, Mesh, find_attribute};

impl ObjectManager {
    /// `device_update_geom_offsets`. `true` when an object changed and
    /// the device must take the objects again.
    pub fn device_update_geom_offset(dscene: &mut DeviceScene, object: &[Object], mesh: &[Mesh]) -> bool {
        let not_found = constant("ATTR_STD_NOT_FOUND").unwrap_or(0) as i32;
        let mut update = false;
        let count = dscene.object.len();
        for object in object.iter().filter(|object| object.index < count) {
            let Some(mesh) = mesh.get(object.geometry) else {
                continue;
            };
            // An object's own map never starts at 0: the meshes' come first.
            let map = match object.attr_map_offset {
                0 => mesh.geometry.attr_map_offset,
                offset => offset,
            };
            let offset = |std: &str| find_attribute(&dscene.attribute_map, map, false, AttributeStandard::of(std).0 as u64).offset;
            let position = offset("POSITION");
            // The corner normals when the mesh has them, else the vertex normals.
            let normal = [offset("CORNER_NORMAL"), offset("VERTEX_NORMAL")].into_iter().find(|found| *found != not_found).unwrap_or(not_found);
            debug_assert!(mesh.triangle_count() == 0 || (position != not_found && normal != not_found));

            let (index, out) = (object.index, &mut dscene.object);
            let before = (out.uint(index, "attribute_map_offset"), out.int(index, "position_offset"), out.int(index, "normal_offset"), out.int(index, "numverts"));
            let after = (map as u32, position, normal, mesh.vert_count() as i32);
            out.set_uint(index, "attribute_map_offset", after.0);
            out.set_int(index, "position_offset", after.1);
            out.set_int(index, "normal_offset", after.2);
            out.set_int(index, "numverts", after.3);
            update |= before != after;
        }
        update
    }
}

#[cfg(test)]
mod test;
