// SPDX-FileCopyrightText: 2009-2010 NVIDIA Corporation
// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/bvh/bvh.h` (`scene_intersect`, line
//! 224): the closest hit in the scene. Cycles chooses among the walks of
//! motion and hair; here there is the one.
use super::traversal::bvh_intersect;
use super::util::intersection_ray_valid;
use crate::kernel::globals::KernelGlobals;
use crate::kernel::types::{Intersection, Ray};

/// `scene_intersect`.
pub fn scene_intersect(kg: &KernelGlobals, ray: &Ray, visibility: u32, isect: &mut Intersection) -> bool {
    if !intersection_ray_valid(ray) {
        return false;
    }
    bvh_intersect(kg, ray, isect, visibility)
}
