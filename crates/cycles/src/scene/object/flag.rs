// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/object.cpp`
//! (`ObjectManager::device_update_flags`, `apply_static_transforms`,
//! `device_update_prim_offsets`): the flags of each object that depend on
//! its geometry and its neighbours, and the transforms that are put into
//! the geometry so the BVH need not instance it.

use super::{Object, ObjectManager};
use crate::kernel::flag;
use crate::scene::{AttributeStandard, DeviceScene, Mesh, MotionType, Shader};

/// `mask` with the flag named so set when `on`, else cleared.
fn put(mask: &mut u32, name: &str, on: bool) {
    match on {
        true => *mask |= flag(name),
        false => *mask &= !flag(name),
    }
}

impl ObjectManager {
    /// `device_update_flags`: volumes, shadow catchers, corner normals,
    /// and which objects touch a volume. Before the bounds are final
    /// (`bounds_valid` false) every object is taken to touch one when
    /// there is any, and the flags stay due.
    pub fn device_update_flag(&mut self, dscene: &mut DeviceScene, object: &mut [Object], mesh: &[Mesh], bounds_valid: bool) {
        if !self.need_update() && !self.need_flag_update {
            return;
        }
        if bounds_valid {
            self.clear_update();
            self.need_flag_update = false;
        }
        let has_volume = |object: &Object| mesh.get(object.geometry).is_some_and(|mesh| mesh.geometry.has_volume);
        let volume: Vec<(usize, crate::util::BoundBox)> = object.iter().filter(|object| has_volume(object)).map(|object| (object.index, object.bound)).collect();
        let corner = AttributeStandard::of("CORNER_NORMAL");
        for object in object.iter_mut() {
            let Some((mesh, mask)) = mesh.get(object.geometry).zip(dscene.object_flag.get_mut(object.index)) else {
                continue;
            };
            let voxel = mesh.geometry.has_volume && mesh.geometry.has_voxel_attribute();
            put(mask, "SD_OBJECT_HAS_VOLUME", mesh.geometry.has_volume);
            put(mask, "SD_OBJECT_HAS_VOLUME_ATTRIBUTES", voxel);
            put(mask, "SD_OBJECT_SHADOW_CATCHER", object.node.boolean("is_shadow_catcher"));
            // Subdivision and displacement may have taken the corner normals away.
            put(mask, "SD_OBJECT_HAS_CORNER_NORMALS", mesh.geometry.attribute.find_standard(corner).is_some());
            match bounds_valid {
                true => {
                    object.intersects_volume = volume.iter().any(|(index, bound)| *index != object.index && object.bound.intersects(bound));
                    *mask |= flag("SD_OBJECT_INTERSECTS_VOLUME") * u32::from(object.intersects_volume);
                }
                false => *mask |= flag("SD_OBJECT_INTERSECTS_VOLUME") * u32::from(!volume.is_empty()),
            }
        }
    }

    /// `apply_static_transforms`: an object that alone uses its mesh, and
    /// neither moves, scatters under its surface, displaces nor
    /// subdivides, has its transform put into the mesh.
    pub fn apply_static_transform(dscene: &mut DeviceScene, object: &mut [Object], mesh: &mut [Mesh], shader: &[Shader], need_motion: MotionType) {
        let mut user = vec![0usize; mesh.len()];
        for object in object.iter() {
            user.get_mut(object.geometry).into_iter().for_each(|count| *count += 1);
        }
        let motion_blur = need_motion == MotionType::Blur;
        for (place, object) in object.iter_mut().enumerate() {
            let Some(mesh) = mesh.get_mut(object.geometry) else {
                continue;
            };
            let alone = user[object.geometry] == 1 && !mesh.geometry.has_surface_bssrdf && !mesh.geometry.has_true_displacement(shader);
            if !alone || mesh.is_subdivided() || (motion_blur && object.use_motion()) {
                continue;
            }
            if !mesh.geometry.transform_applied {
                object.apply_transform(mesh, !need_motion.is_pass());
                mesh.geometry.transform_applied = true;
            }
            dscene.object_flag.get_mut(place).into_iter().for_each(|mask| *mask |= flag("SD_OBJECT_TRANSFORM_APPLIED"));
        }
    }

    /// `device_update_prim_offsets`: each object's first primitive, for
    /// the devices whose BVH cannot hold it (Metal and HIP-RT, or any
    /// with the light tree). The caller knows whether this device wants
    /// it.
    pub fn device_update_prim_offset(dscene: &mut DeviceScene, object: &[Object], mesh: &[Mesh]) {
        dscene.object_prim_offset = vec![0; object.len()];
        for object in object {
            let offset = mesh.get(object.geometry).map(|mesh| mesh.geometry.prim_offset as u32).unwrap_or(0);
            dscene.object_prim_offset.get_mut(object.index).into_iter().for_each(|slot| *slot = offset);
        }
    }
}
