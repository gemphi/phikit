// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/types.h` (`SHADER_CLOSURE_BASE` and
//! `ShaderClosure`, lines 868-886; `ShaderData`, lines 1019-1099): a shading point and the
//! closures its shader left in it, the members a diffuse or emissive
//! surface and the background use.
use super::bit::{MAX_CLOSURE, NONE};
/// `ShaderClosure`: a weight, the chance it is sampled and a closure
/// type; `n` is the normal a BSDF uses (`DiffuseBsdf` adds nothing else).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ShaderClosure {
    pub weight: [f32; 3],
    pub sample_weight: f32,
    pub kind: i32,
    pub n: [f32; 3],
}

/// `ShaderData`, the members a diffuse or emissive surface and the
/// background use.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShaderData {
    pub p: [f32; 3],
    pub n: [f32; 3],
    pub ng: [f32; 3],
    /// `wi`: towards where the ray came from.
    pub wi: [f32; 3],
    pub shader: i32,
    pub flag: u32,
    pub object_flag: u32,
    pub object: i32,
    pub prim: i32,
    pub kind: i32,
    pub u: f32,
    pub v: f32,
    pub ray_length: f32,
    pub time: f32,
    pub num_closure: usize,
    /// `num_closure_left`: room for closures; none when the shader is run only for its emission.
    pub num_closure_left: usize,
    pub closure: [ShaderClosure; MAX_CLOSURE],
    pub closure_emission_background: [f32; 3],
}

impl Default for ShaderData {
    fn default() -> Self {
        Self {
            p: [0.0; 3],
            n: [0.0; 3],
            ng: [0.0; 3],
            wi: [0.0; 3],
            shader: NONE,
            flag: 0,
            object_flag: 0,
            object: NONE,
            prim: NONE,
            kind: 0,
            u: 0.0,
            v: 0.0,
            ray_length: 0.0,
            time: 0.0,
            num_closure: 0,
            num_closure_left: MAX_CLOSURE,
            closure: [ShaderClosure::default(); MAX_CLOSURE],
            closure_emission_background: [0.0; 3],
        }
    }
}
