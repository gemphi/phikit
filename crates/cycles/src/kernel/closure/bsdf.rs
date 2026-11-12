// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/closure/bsdf.h` (`bsdf_sample`,
//! lines 165-330; `CLOSURE_IS_BSDF`, `kernel/types.h`). Only the diffuse
//! closure is ported: any other type has no sample (a zero pdf).
use super::diffuse::bsdf_diffuse_sample;
use crate::kernel::constant;
use crate::kernel::{ShaderClosure, ShaderData};

/// `CLOSURE_IS_BSDF` (`kernel/types.h:535`): not none, and up to the transparent BSDF.
pub fn closure_is_bsdf(kind: i32) -> bool {
    let last = constant("CLOSURE_BSDF_TRANSPARENT_ID").unwrap_or(0) as i32;
    kind != constant("CLOSURE_NONE_ID").unwrap_or(0) as i32 && kind <= last
}

/// What `bsdf_sample` gives: the value, the direction, the pdf, the label,
/// and the roughness and eta that go with them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BsdfSample {
    pub eval: [f32; 3],
    pub wo: [f32; 3],
    pub pdf: f32,
    pub label: u32,
    pub eta: f32,
}

/// `bsdf_sample`: the direction a closure sends a path in. The pdf is zero
/// for a closure that is not ported.
pub fn bsdf_sample(sd: &ShaderData, sc: &ShaderClosure, rand: [f32; 3]) -> BsdfSample {
    let diffuse = constant("CLOSURE_BSDF_DIFFUSE_ID").unwrap_or(-1) as i32;
    if sc.kind == diffuse {
        let (eval, wo, pdf, label) = bsdf_diffuse_sample(sc, sd.ng, [rand[0], rand[1]]);
        return BsdfSample { eval, wo, pdf, label, eta: 1.0 };
    }
    BsdfSample { eval: [0.0; 3], wo: [0.0; 3], pdf: 0.0, label: 0, eta: 1.0 }
}
