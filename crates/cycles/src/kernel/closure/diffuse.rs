// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/closure/bsdf_diffuse.h` (lines 15-70):
//! the Lambertian BSDF. `DiffuseBsdf` holds only the base of a
//! `ShaderClosure`, its normal.
use super::alloc::bsdf_alloc;
use crate::kernel::constant;
use crate::kernel::sample::sample_cos_hemisphere;
use crate::kernel::types::{LABEL_DIFFUSE, LABEL_REFLECT, SD_BSDF, SD_BSDF_HAS_EVAL, ShaderClosure, ShaderData};
use crate::util::vector::dot;

/// `bsdf_diffuse_setup`: a diffuse closure about `n` with `weight`.
pub fn bsdf_diffuse_setup(sd: &mut ShaderData, n: [f32; 3], weight: [f32; 3]) {
    let kind = constant("CLOSURE_BSDF_DIFFUSE_ID").unwrap_or(0) as i32;
    if let Some(bsdf) = bsdf_alloc(sd, weight) {
        bsdf.n = n;
        bsdf.kind = kind;
        sd.flag |= SD_BSDF | SD_BSDF_HAS_EVAL;
    }
}

/// `bsdf_diffuse_eval`: the value and the pdf in direction `wo`, both
/// `max(N . wo, 0) / pi`.
pub fn bsdf_diffuse_eval(sc: &ShaderClosure, wo: [f32; 3]) -> ([f32; 3], f32) {
    let cos_no = dot(sc.n, wo).max(0.0) * std::f32::consts::FRAC_1_PI;
    ([cos_no; 3], cos_no)
}

/// `bsdf_diffuse_sample`: a cosine-weighted direction about the closure's
/// normal; no value and pdf when it is under the surface (`Ng`). Returns
/// the value, the direction, the pdf and the label.
pub fn bsdf_diffuse_sample(sc: &ShaderClosure, ng: [f32; 3], rand: [f32; 2]) -> ([f32; 3], [f32; 3], f32, u32) {
    let (wo, pdf) = sample_cos_hemisphere(sc.n, rand);
    let label = LABEL_REFLECT | LABEL_DIFFUSE;
    match dot(ng, wo) > 0.0 {
        true => ([pdf; 3], wo, pdf, label),
        false => ([0.0; 3], wo, 0.0, label),
    }
}
