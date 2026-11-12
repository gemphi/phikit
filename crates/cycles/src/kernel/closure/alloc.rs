// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/closure/alloc.h` (lines 10-100):
//! the closures of a shading point are an array in the shader data; a
//! closure is given its weight, and a BSDF its sample weight, the chance
//! that it is the one a bounce samples.
use crate::kernel::constant;
use crate::kernel::bit::MAX_CLOSURE;
use crate::kernel::{ShaderClosure, ShaderData};

/// `CLOSURE_WEIGHT_CUTOFF` (`kernel/types.h`): a weight under it is no closure.
const CLOSURE_WEIGHT_CUTOFF: f32 = 1e-5;

/// `closure_alloc`: the next closure of the shader data, of `kind` and
/// `weight`; `None` when the array is full.
pub fn closure_alloc(sd: &mut ShaderData, kind: i32, weight: [f32; 3]) -> Option<&mut ShaderClosure> {
    if sd.num_closure_left == 0 || sd.num_closure >= MAX_CLOSURE {
        return None;
    }
    let place = sd.num_closure;
    sd.num_closure += 1;
    sd.num_closure_left -= 1;
    sd.closure[place] = ShaderClosure { weight, kind, ..ShaderClosure::default() };
    Some(&mut sd.closure[place])
}

/// `closure_sample_weight`: the weight with no negative part, and its
/// average as the sample weight; zero when it is under the cutoff or not
/// finite.
pub fn closure_sample_weight(weight: &mut [f32; 3]) -> f32 {
    *weight = weight.map(|channel| channel.max(0.0));
    let sample_weight = ((weight[0] + weight[1] + weight[2]) / 3.0).abs();
    // Written so that a weight that is not a number is under the cutoff.
    match sample_weight >= CLOSURE_WEIGHT_CUTOFF {
        true => sample_weight,
        false => 0.0,
    }
}

/// `bsdf_alloc`: a closure for a BSDF; `None` for a weight too small to
/// matter.
pub fn bsdf_alloc(sd: &mut ShaderData, mut weight: [f32; 3]) -> Option<&mut ShaderClosure> {
    let sample_weight = closure_sample_weight(&mut weight);
    if !(sample_weight > 0.0) {
        return None;
    }
    let closure = closure_alloc(sd, constant("CLOSURE_NONE_ID").unwrap_or(0) as i32, weight)?;
    closure.sample_weight = sample_weight;
    Some(closure)
}
