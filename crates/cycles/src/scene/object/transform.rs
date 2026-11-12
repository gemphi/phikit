// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/object.cpp`
//! (`ObjectManager::device_update_transforms`,
//! `device_update_object_transform`, `UpdateObjectTransformState`): each
//! object as a `KernelObject`, with its flags and its motion.
//!
//! Cycles fills the objects in on several threads; here one after
//! another. A volume's density and velocity, hair and point clouds wait
//! for those kinds of geometry.

use super::{Object, ObjectInput, ObjectManager};
use crate::kernel::{constant, flag};
use crate::scene::{Attribute, AttributeStandard, DeviceScene, Mesh, MotionType};
use crate::util::{Transform, hash_to_float, motion_decompose, murmur_hash3};

/// `UpdateObjectTransformState`: what the objects have between them.
#[derive(Debug, Default)]
struct State {
    /// Each particle system's first particle; particle 0 is a dummy.
    particle_offset: Vec<usize>,
    /// Each object's first step in `object_motion`.
    motion_offset: Vec<usize>,
    have_motion: bool,
}

/// The flag named so when `on`, else no bits.
fn when(on: bool, name: &str) -> u32 {
    match on {
        true => flag(name),
        false => 0,
    }
}

/// The motion flag of a mesh: its vertices move, or, when they do not, it
/// has corner normals.
fn mesh_flag(mesh: &Mesh) -> u32 {
    let position = AttributeStandard::of("POSITION");
    let moving = mesh.geometry.attribute.find_standard(position).is_some_and(Attribute::has_motion);
    let subd = mesh.is_subdivided() && mesh.subd_attribute.find_standard(position).is_some_and(Attribute::has_motion);
    let corner = mesh.geometry.attribute.find_standard(AttributeStandard::of("CORNER_NORMAL")).is_some();
    match (moving || subd, corner) {
        (true, _) => flag("SD_OBJECT_HAS_VERTEX_MOTION"),
        (false, true) => flag("SD_OBJECT_HAS_CORNER_NORMALS"),
        (false, false) => 0,
    }
}

impl ObjectManager {
    /// `device_update_transforms`.
    pub fn device_update_transform(dscene: &mut DeviceScene, object: &mut [Object], mesh: &[Mesh], input: &ObjectInput) {
        let mut state = State::default();
        dscene.object.alloc(object.len());
        dscene.object_flag = vec![0; object.len()];
        let pass_size = constant("OBJECT_MOTION_PASS_SIZE").unwrap_or(0) as usize;
        match input.need_motion {
            MotionType::Pass | MotionType::PassInteractive => dscene.object_motion_pass = vec![Transform::EMPTY; pass_size * object.len()],
            MotionType::Blur => {
                // Every object's steps follow the one before's.
                let mut offset = 0;
                for object in object.iter_mut() {
                    state.motion_offset.push(offset);
                    object.update_motion();
                    offset += object.motion().len();
                }
                dscene.object_motion = vec![Default::default(); offset];
            }
            MotionType::None => {}
        }
        let mut particle = 1;
        for count in input.particle_count {
            state.particle_offset.push(particle);
            particle += count;
        }
        for object in object.iter_mut() {
            match mesh.get(object.geometry) {
                Some(mesh) => Self::device_update_object_transform(dscene, &mut state, object, mesh, input),
                None => debug_assert!(false, "object {} has no geometry", object.index),
            }
        }
        dscene.data.set_int("bvh", "have_motion", i32::from(state.have_motion));
        // Meshes only so far: no hair, no points, no volumes.
        ["have_curves", "have_points", "have_volumes"].iter().for_each(|name| dscene.data.set_int("bvh", name, 0));
    }

    /// `device_update_object_transform`.
    fn device_update_object_transform(dscene: &mut DeviceScene, state: &mut State, object: &mut Object, mesh: &Mesh, input: &ObjectInput) {
        let (index, node) = (object.index, &object.node);
        let transform = object.transform();
        let inverse = transform.inverse();
        let particle = object.particle_system.and_then(|system| state.particle_offset.get(system)).map(|offset| node.int("particle_index") + *offset as i32);
        let link_max = constant("LIGHT_LINK_SET_MAX").unwrap_or(0) as u32;
        let link_set = |name: &str| Some(node.uint(name)).filter(|set| *set < link_max).unwrap_or(0);
        let not_found = constant("ATTR_STD_NOT_FOUND").unwrap_or(0) as i32;

        let out = &mut dscene.object;
        out.set_transform(index, "tfm", &transform);
        out.set_transform(index, "itfm", &inverse);
        out.set_float(index, "volume_density", 1.0);
        out.set_float_list(index, "color", &node.float3("color"));
        out.set_float(index, "alpha", node.float("alpha"));
        out.set_float(index, "pass_id", node.int("pass_id") as f32);
        out.set_float(index, "random_number", node.uint("random_id") as f32 * (1.0 / u32::MAX as f32));
        out.set_int(index, "particle_index", particle.unwrap_or(0));
        out.set_uint(index, "motion_offset", 0);
        out.set_int(index, "position_offset", not_found);
        out.set_int(index, "normal_offset", not_found);
        out.set_float(index, "ao_distance", node.float("ao_distance"));
        out.set_uint(index, "receiver_light_set", link_set("receiver_light_set"));
        out.set_uint64(index, "light_set_membership", node.uint64("light_set_membership"));
        out.set_uint(index, "blocker_shadow_set", link_set("blocker_shadow_set"));
        out.set_uint64(index, "shadow_set_membership", node.uint64("shadow_set_membership"));
        out.set_float_list(index, "dupli_generated", &node.float3("dupli_generated"));
        out.set_float_list(index, "dupli_uv", &node.float2("dupli_uv"));
        out.set_uint(index, "num_geom_steps", mesh.geometry.motion_step_count());
        out.set_int(index, "numverts", mesh.vert_count() as i32);
        out.set_int(index, "numprims", mesh.triangle_count() as i32);
        out.set_uint(index, "attribute_map_offset", 0);
        out.set_float(index, "cryptomatte_object", hash_to_float(murmur_hash3(node.name.as_bytes(), 0)));
        out.set_float(index, "cryptomatte_asset", hash_to_float(murmur_hash3(node.string("asset_name").as_bytes(), 0)));
        out.set_float(index, "shadow_terminator_shading_offset", 1.0 / (1.0 - 0.5 * node.float("shadow_terminator_shading_offset")));
        out.set_float(index, "shadow_terminator_geometry_offset", node.float("shadow_terminator_geometry_offset"));
        out.set_uint(index, "visibility", object.visibility_for_tracing());
        out.set_int(index, "primitive_type", mesh.primitive_type());
        let lightgroup = input.lightgroup.iter().position(|name| name == node.string("lightgroup"));
        out.set_int(index, "lightgroup", lightgroup.map(|place| place as i32).unwrap_or(constant("LIGHTGROUP_NONE").unwrap_or(-1) as i32));

        state.have_motion |= mesh.geometry.use_motion_blur();
        let mut mask = when(transform.negative_scale(), "SD_OBJECT_NEGATIVE_SCALE") | mesh_flag(mesh);
        mask |= when(node.boolean("is_caustics_caster"), "SD_OBJECT_CAUSTICS_CASTER") | when(node.boolean("is_caustics_receiver"), "SD_OBJECT_CAUSTICS_RECEIVER");
        mask |= when(node.boolean("use_holdout"), "SD_OBJECT_HOLDOUT_MASK");

        match input.need_motion {
            MotionType::Pass | MotionType::PassInteractive => {
                object.update_motion();
                let (first, last) = (object.motion().first().copied(), object.motion().last().copied());
                let (mut before, mut after) = match object.use_motion() {
                    true => (first.unwrap_or(transform), last.unwrap_or(transform)),
                    false => (transform, transform),
                };
                // A mesh whose vertices move brings them in its own space;
                // otherwise the shading point is moved, in the world.
                if mask & flag("SD_OBJECT_HAS_VERTEX_MOTION") == 0 {
                    (before, after) = (before * inverse, after * inverse);
                }
                let place = index * constant("OBJECT_MOTION_PASS_SIZE").unwrap_or(0) as usize;
                dscene.object_motion_pass[place] = before;
                dscene.object_motion_pass[place + 1] = after;
            }
            MotionType::Blur if object.use_motion() => {
                let offset = state.motion_offset.get(index).copied().unwrap_or(0);
                dscene.object.set_uint(index, "motion_offset", offset as u32);
                let decomp = motion_decompose(object.motion());
                dscene.object_motion[offset..offset + decomp.len()].copy_from_slice(&decomp);
                mask |= flag("SD_OBJECT_MOTION");
                state.have_motion = true;
            }
            _ => {}
        }
        dscene.object.set_uint(index, "num_tfm_steps", object.motion().len() as u32);
        dscene.object_flag[index] = mask;
    }
}
