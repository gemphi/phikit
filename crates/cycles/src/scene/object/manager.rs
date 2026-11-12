// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/object.h` and `object.cpp`
//! (`ObjectManager`: its update flags, `tag_update`, `need_update`,
//! `update_interactive_motion`, `device_update`, `device_free`): the
//! objects of a scene, written to the device arrays `objects`,
//! `object_flag`, `object_motion_pass` and `object_motion`.
//!
//! Cycles tags each device array as modified or to reallocate and copies
//! only those. Here an update writes the arrays whole. The world's own
//! object (a background light) is not looked for: it comes with the
//! lights.

use super::Object;
use crate::graph::Value;
use crate::kernel::flag;
use crate::scene::{DeviceScene, GeometryManager, Mesh, MotionType};

/// What the objects read of the rest of the scene.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ObjectInput<'a> {
    /// `Scene::need_motion`.
    pub need_motion: MotionType,
    /// How many particles each of the scene's particle systems has.
    pub particle_count: &'a [usize],
    /// `Scene::lightgroups`: the light groups, each numbered by its place.
    pub lightgroup: &'a [String],
}

/// `ObjectManager`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectManager {
    update_flag: u32,
    /// The object flags must be worked out again.
    pub need_flag_update: bool,
}

impl Default for ObjectManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ObjectManager {
    /// A manager with everything to update.
    pub fn new() -> ObjectManager {
        ObjectManager { update_flag: Self::flag("UPDATE_ALL"), need_flag_update: true }
    }

    /// One of the manager's update flags, such as `TRANSFORM_MODIFIED`.
    pub fn flag(name: &str) -> u32 {
        flag(&format!("ObjectManager::{name}"))
    }

    /// `need_update`.
    pub fn need_update(&self) -> bool {
        self.update_flag != Self::flag("UPDATE_NONE")
    }

    pub(super) fn clear_update(&mut self) {
        self.update_flag = Self::flag("UPDATE_NONE");
    }

    /// `tag_update`: something about the objects changed. The answer is
    /// the flags the geometry manager is to be tagged with, zero when it
    /// was the one that tagged this. (Cycles also tags the lights, and the
    /// integrator for its shadow catcher settings.)
    pub fn tag_update(&mut self, update: u32) -> u32 {
        self.update_flag |= update;
        let on = |ours: &str, theirs: &[&str]| match update & Self::flag(ours) != 0 {
            true => theirs.iter().fold(0, |mask, name| mask | GeometryManager::flag(name)),
            false => 0,
        };
        // Objects added or taken away may be instances: no geometry
        // changed, yet the BVH must be built again.
        let changed = on("OBJECT_ADDED", &["GEOMETRY_ADDED", "GEOMETRY_REMOVED"]) | on("OBJECT_REMOVED", &["GEOMETRY_ADDED", "GEOMETRY_REMOVED"]);
        let moved = on("TRANSFORM_MODIFIED", &["TRANSFORM_MODIFIED"]) | on("VISIBILITY_MODIFIED", &["VISIBILITY_MODIFIED"]);
        match update & Self::flag("GEOMETRY_MANAGER") == 0 {
            true => GeometryManager::flag("OBJECT_MANAGER") | changed | moved,
            false => 0,
        }
    }

    /// `update_interactive_motion`: in a viewport an object's transform
    /// of the frame before is kept as its first motion step, for the
    /// motion vectors. `true` when an object that moved was found, and
    /// the transforms are tagged.
    pub fn update_interactive_motion(&mut self, object: &mut [Object]) -> bool {
        let mut update = false;
        for object in object.iter_mut() {
            let moving = object.use_motion();
            let mut motion = object.motion().to_vec();
            // One step alone is not motion: it only remembers the transform.
            motion.resize(motion.len().max(1), object.transform());
            motion[0] = object.transform();
            object.node.set("motion", Value::TransformArray(motion));
            update |= moving && object.node.socket_is_modified("motion");
        }
        match update {
            true => self.update_flag |= Self::flag("TRANSFORM_MODIFIED"),
            false => {}
        }
        update
    }

    /// `device_update`: number the objects and write their transforms.
    pub fn device_update(&mut self, dscene: &mut DeviceScene, object: &mut [Object], mesh: &[Mesh], input: &ObjectInput) {
        if !self.need_update() {
            return;
        }
        Self::device_free(dscene);
        object.iter_mut().enumerate().for_each(|(index, object)| object.index = index);
        if object.is_empty() {
            return;
        }
        Self::device_update_transform(dscene, object, mesh, input);
        object.iter_mut().for_each(|object| object.node.clear_modified());
    }

    /// `device_free`.
    pub fn device_free(dscene: &mut DeviceScene) {
        dscene.object.alloc(0);
        dscene.object_motion_pass.clear();
        dscene.object_motion.clear();
        dscene.object_flag.clear();
        dscene.object_prim_offset.clear();
    }
}

#[cfg(test)]
mod test;
