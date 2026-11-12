// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/integrator/state_template.h` (the
//! `path`, `ray` and `isect` structs) and `shadow_state_template.h` (the
//! queue of a shadow path), with `state.h` (`IntegratorStateCPU`).
//!
//! The members are those of the templates that every path has
//! (`KERNEL_FEATURE_PATH_TRACING`), in the templates' order, each a whole
//! number of 32-bit words as the record layouts of `kernel/record.rs` are:
//! a `uint16_t` is a word, a `packed_float3` three. So the struct is `Pod`
//! and a path state can live in a block of a device, and
//! [`StateLayout`](super::layout::StateLayout), read from `data/state.tsv`
//! (`tool/state.py`), says where each member is; `tests/kernel/integrator/
//! state.rs` checks the struct against it. The members of a feature a gate
//! does not have (path guiding, light passes, MNEE, volumes, subsurface)
//! are left out.
use bytemuck::{Pod, Zeroable};

use crate::kernel::types::NONE;
use crate::kernel::{Intersection, Ray, RaySelfPrimitive};

/// `IntegratorStateCPU::path`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct PathState {
    pub render_pixel_index: u32,
    pub sample: u32,
    pub bounce: u32,
    pub transparent_bounce: u32,
    pub diffuse_bounce: u32,
    pub glossy_bounce: u32,
    pub transmission_bounce: u32,
    pub volume_bounce: u32,
    pub volume_bounds_bounce: u32,
    /// The `DeviceKernel` queued for the path; zero is none.
    pub queued_kernel: u32,
    pub rng_pixel: u32,
    pub rng_offset: u32,
    pub visibility: u32,
    pub flag: u32,
    pub mnee: u32,
    pub optical_depth: f32,
    pub mis_ray_pdf: f32,
    pub mis_origin_n: [f32; 3],
    pub min_ray_pdf: f32,
    pub continuation_probability: f32,
    pub throughput: [f32; 3],
    pub shader_sort_key: u32,
}

/// `IntegratorStateCPU::ray`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct RayState {
    pub p: [f32; 3],
    pub dp: f32,
    pub d: [f32; 3],
    pub dd: f32,
    pub tmin: f32,
    pub tmax: f32,
    pub time: f32,
}

/// `IntegratorStateCPU::isect`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct IsectState {
    pub t: f32,
    pub u: f32,
    pub v: f32,
    pub prim: i32,
    pub object: i32,
    pub kind: i32,
}

/// `IntegratorShadowStateCPU`, so far its queue: no shadow path is made
/// until there are lights (gate 3), and `shadow_path.queued_kernel` is what
/// the megakernel looks at first.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct ShadowPathState {
    pub queued_kernel: u32,
}

/// `IntegratorStateCPU`: the path, its ray and its last hit, and the
/// queues of the shadow path and of the ambient occlusion path.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct IntegratorState {
    pub path: PathState,
    pub ray: RayState,
    pub isect: IsectState,
    pub shadow: ShadowPathState,
    pub ao: ShadowPathState,
}

impl IntegratorState {
    /// A state whose queues are empty: no kernel is queued, the path is
    /// terminated (`path_state_init_queues`).
    pub fn new() -> IntegratorState {
        bytemuck::Zeroable::zeroed()
    }

    /// `integrator_state_read_ray`.
    pub fn read_ray(&self) -> Ray {
        let r = &self.ray;
        Ray { p: r.p, d: r.d, tmin: r.tmin, tmax: r.tmax, time: r.time, dp: r.dp, dd: r.dd, own: RaySelfPrimitive::default() }
    }

    /// `integrator_state_write_ray`.
    pub fn write_ray(&mut self, ray: &Ray) {
        self.ray = RayState { p: ray.p, dp: ray.dp, d: ray.d, dd: ray.dd, tmin: ray.tmin, tmax: ray.tmax, time: ray.time };
    }

    /// `integrator_state_read_isect`.
    pub fn read_isect(&self) -> Intersection {
        let i = &self.isect;
        Intersection { t: i.t, u: i.u, v: i.v, prim: i.prim, object: i.object, kind: i.kind }
    }

    /// `integrator_state_write_isect`.
    pub fn write_isect(&mut self, isect: &Intersection) {
        self.isect = IsectState { t: isect.t, u: isect.u, v: isect.v, prim: isect.prim, object: isect.object, kind: isect.kind };
    }

    /// The hit that starts a path: nothing (`path_state_init_integrator`).
    pub fn clear_isect(&mut self) {
        self.isect = IsectState { t: 0.0, u: 0.0, v: 0.0, prim: NONE, object: NONE, kind: 0 };
    }
}

impl Default for IntegratorState {
    fn default() -> Self {
        Self::new()
    }
}
