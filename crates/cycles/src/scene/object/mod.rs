// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/object.h` and `object.cpp` (`Object`):
//! a geometry placed in the scene. Several objects may place one geometry.
//!
//! The sockets are the `object` type of `data/node.tsv`. The geometry and
//! the particle system are places in the scene's lists. Not ported: the
//! step size of a volume (`compute_volume_step_size`), the nudge a
//! volume's transform gets (`adjust_volume_tfm`), and an object's own
//! attribute values (`attributes`).

mod flag;
mod manager;
mod manifest;
mod offset;
mod transform;

pub use manager::{ObjectInput, ObjectManager};

use super::{Geometry, Mesh, Shader};
use crate::graph::{Node, Value};
use crate::kernel::{constant, flag as kernel_flag};
use crate::util::{BoundBox, Transform, motion_array_interpolate, motion_decompose};

/// `Object`.
#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    /// The settings, by socket name.
    pub node: Node,
    /// `geometry`: its place in the scene's list of geometry.
    pub geometry: usize,
    /// `particle_system`: its place in the scene's list, when the object
    /// is a particle.
    pub particle_system: Option<usize>,
    /// `bounds`: the box it fits in, in the world.
    pub bound: BoundBox,
    /// It touches the box of an object with a volume; set when the
    /// device's flags are updated.
    pub intersects_volume: bool,
    /// Its place in the device arrays; set when the device is updated.
    pub index: usize,
    /// Its own entry in the attribute map, or 0 for its geometry's.
    pub attr_map_offset: usize,
}

impl Object {
    /// An object that places the geometry at `geometry`, every socket at
    /// its default.
    pub fn new(geometry: usize) -> Object {
        let node = Node::new("object").expect("data/node.tsv has the object");
        Object { node, geometry, particle_system: None, bound: BoundBox::EMPTY, intersects_volume: false, index: 0, attr_map_offset: 0 }
    }

    /// `get_tfm`.
    pub fn transform(&self) -> Transform {
        self.node.transform("tfm")
    }

    /// `set_tfm`.
    pub fn set_transform(&mut self, transform: Transform) -> bool {
        self.node.set("tfm", Value::Transform(transform))
    }

    /// `get_motion`: its transform at each motion step.
    pub fn motion(&self) -> &[Transform] {
        self.node.transform_array("motion")
    }

    /// `set_motion_tfm`: its transform at one motion step.
    pub fn set_motion_transform(&mut self, transform: Transform, step: usize) -> bool {
        let mut motion = self.motion().to_vec();
        let changed = motion.get_mut(step).map(|slot| *slot = transform).is_some();
        changed && self.node.set("motion", Value::TransformArray(motion))
    }

    /// `use_motion`.
    pub fn use_motion(&self) -> bool {
        self.motion().len() > 1
    }

    /// `motion_time`: a step's time, from -1 to 1.
    pub fn motion_time(&self, step: usize) -> f32 {
        match self.use_motion() {
            true => 2.0 * step as f32 / (self.motion().len() - 1) as f32 - 1.0,
            false => 0.0,
        }
    }

    /// `motion_step`: the step at a time, when there is one exactly then.
    pub fn motion_step(&self, time: f32) -> Option<usize> {
        (0..self.motion().len()).filter(|_| self.use_motion()).find(|step| self.motion_time(*step) == time)
    }

    /// `update_motion`: make the motion steps whole, or drop them. A step
    /// with no transform takes the object's own; an object that does not
    /// in fact move has no motion; one that hides when a step is missing
    /// gets the empty transform.
    pub fn update_motion(&mut self) {
        let transform = self.transform();
        let mut motion = self.motion().to_vec();
        if motion.len() < 2 {
            return;
        }
        let mut moving = false;
        for index in 0..motion.len() {
            let (missing, last) = (motion[index] == Transform::EMPTY, index + 1 == motion.len());
            match (missing, self.node.boolean("hide_on_missing_motion"), moving && last) {
                (true, true, _) => {
                    self.set_transform(Transform::EMPTY);
                    self.node.set("motion", Value::TransformArray(Vec::new()));
                    return;
                }
                (true, false, true) => {
                    // A last step that was never set is taken off.
                    motion.pop();
                    self.node.set("motion", Value::TransformArray(motion));
                    return;
                }
                (true, false, false) => motion[index] = transform,
                (false, _, _) => {}
            }
            moving |= motion[index] != transform;
        }
        let kept = match moving {
            true => motion,
            false => Vec::new(),
        };
        self.node.set("motion", Value::TransformArray(kept));
    }

    /// `compute_bounds`: the box of its geometry, in the world. With
    /// motion blur, the box of every place it passes through, sampled at
    /// 128 times.
    pub fn compute_bound(&mut self, geometry: &Geometry, motion_blur: bool) {
        self.bound = match (motion_blur && self.use_motion(), geometry.transform_applied) {
            (true, _) => {
                let decomp = motion_decompose(self.motion());
                let mut bound = BoundBox::EMPTY;
                (0..128).for_each(|step| bound.grow_box(&geometry.bound.transformed(&motion_array_interpolate(&decomp, step as f32 / 128.0))));
                bound
            }
            (false, true) => geometry.bound,
            (false, false) => geometry.bound.transformed(&self.transform()),
        };
    }

    /// `apply_transform`: put its transform into its mesh. The transform
    /// is not reset: whoever reads it checks `transform_applied`.
    pub fn apply_transform(&mut self, mesh: &mut Mesh, apply_to_motion: bool) {
        let transform = self.transform();
        if transform == Transform::IDENTITY {
            return;
        }
        mesh.apply_transform(&transform, apply_to_motion);
        // The normals keep their side on a mirrored mesh: it is told, so
        // it turns them when it works them out again.
        mesh.geometry.transform_negative_scaled |= transform.negative_scale();
        if self.bound.valid() {
            mesh.compute_bound();
            self.compute_bound(&mesh.geometry, false);
        }
    }

    /// `is_traceable`: worth putting in the kernel's scene. Not a light,
    /// and not empty.
    pub fn is_traceable(&self, geometry: &Geometry) -> bool {
        !geometry.kind.is_light() && self.bound.valid() && self.bound.size() != [0.0; 3]
    }

    /// `visibility_for_tracing`: the rays that see it; a shadow catcher's
    /// are repeated 16 bits up, for the shadow catcher pass.
    pub fn visibility_for_tracing(&self) -> u32 {
        let visibility = self.node.uint("visibility");
        debug_assert!(visibility & !kernel_flag("PATH_RAY_VISIBILITY_ALL") == 0);
        match self.node.boolean("is_shadow_catcher") {
            true => (visibility << 16) | visibility,
            false => visibility,
        }
    }

    /// `usable_as_light`: a mesh that rays can reach, seen by a ray off a
    /// surface or out of a volume, with a shader that emits.
    pub fn usable_as_light(&self, geometry: &Geometry, shader: &[Shader]) -> bool {
        let seen = ["DIFFUSE", "GLOSSY", "TRANSMIT", "VOLUME_SCATTER"].iter().fold(0, |seen, name| seen | kernel_flag(&format!("PATH_RAY_VISIBILITY_{name}")));
        let lit = self.node.uint("visibility") & seen != 0;
        geometry.kind.is_mesh_like() && self.is_traceable(geometry) && lit && geometry.shader_list(shader).any(|shader| shader.emission)
    }

    /// `has_light_linking`: it takes light from a set of lights only, or
    /// is not in every set.
    pub fn has_light_linking(&self) -> bool {
        self.node.uint("receiver_light_set") != 0 || self.node.uint64("light_set_membership") != link_mask_all()
    }

    /// `has_shadow_linking`.
    pub fn has_shadow_linking(&self) -> bool {
        self.node.uint("blocker_shadow_set") != 0 || self.node.uint64("shadow_set_membership") != link_mask_all()
    }

    /// `tag_update`, the part that is the object's own: what changed, as
    /// the flags `ObjectManager::tag_update` takes.
    pub fn update_flag(&self) -> u32 {
        let changed = |name: &str| self.node.socket_is_modified(name);
        let flag = |on: bool, name: &str| match on {
            true => ObjectManager::flag(name),
            false => 0,
        };
        let modified = self.node.is_modified();
        let catcher = modified && changed("is_shadow_catcher");
        flag(modified, "OBJECT_MODIFIED") | flag(modified && changed("use_holdout"), "HOLDOUT_MODIFIED") | flag(changed("tfm") || changed("motion"), "TRANSFORM_MODIFIED") | flag(catcher || changed("visibility"), "VISIBILITY_MODIFIED")
    }
}

/// `LIGHT_LINK_MASK_ALL`: in every set.
fn link_mask_all() -> u64 {
    constant("LIGHT_LINK_MASK_ALL").unwrap_or(-1) as u64
}
