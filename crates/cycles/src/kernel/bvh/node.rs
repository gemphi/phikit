// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/bvh/nodes.h`
//! (`bvh_aligned_node_intersect`, lines 24-71): a ray against the two
//! boxes of an inner node. The unaligned node, which hair has, is not
//! ported: the build makes none.

use crate::kernel::KernelGlobal;

/// Cycles' `max` and `min` of two floats: the second when either is not a
/// number, which keeps a box the ray runs along from being lost.
fn most(a: f32, b: f32) -> f32 {
    match a > b {
        true => a,
        false => b,
    }
}

fn least(a: f32, b: f32) -> f32 {
    match a < b {
        true => a,
        false => b,
    }
}

/// `bvh_aligned_node_intersect`: which children of the node at
/// `node_addr` the ray enters between `tmin` and `tmax`, as bit 1 for the
/// first and bit 2 for the second, and how far along the ray each starts.
/// A child no ray of this `visibility` may see counts as missed.
pub fn bvh_aligned_node_intersect(kg: &KernelGlobal, p: [f32; 3], idir: [f32; 3], tmin: f32, tmax: f32, node_addr: i32, visibility: u32) -> (u32, [f32; 2]) {
    let node = &kg.scene.bvh_node[node_addr as usize..node_addr as usize + 4];
    let float = |row: usize, column: usize| f32::from_bits(node[row][column] as u32);
    // Rows 1 to 3 are x, y and z: each the low side of both children, then
    // the high side of both.
    let child = |index: usize| {
        let (mut near, mut far) = (tmin, tmax);
        for axis in 0..3 {
            let low = (float(axis + 1, index) - p[axis]) * idir[axis];
            let high = (float(axis + 1, index + 2) - p[axis]) * idir[axis];
            near = most(near, least(low, high));
            far = least(far, most(low, high));
        }
        (near, far)
    };
    let ((near0, far0), (near1, far1)) = (child(0), child(1));
    let seen = |index: usize| node[0][index] as u32 & visibility != 0;
    let mask = u32::from(far0 >= near0 && seen(0)) | (u32::from(far1 >= near1 && seen(1)) << 1);
    (mask, [near0, near1])
}

#[cfg(test)]
mod test;
