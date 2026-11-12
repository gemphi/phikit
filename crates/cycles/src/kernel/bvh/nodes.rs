// SPDX-FileCopyrightText: 2009-2010 NVIDIA Corporation
// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/bvh/nodes.h` (`bvh_aligned_node_
//! intersect`, lines 25-80): a ray against the two children's boxes of an
//! inner node, with the visibility of each child tested.
use crate::kernel::globals::KernelGlobals;

fn float(word: i32) -> f32 {
    f32::from_bits(word as u32)
}

/// Which children the ray crosses (bit 0 the first, bit 1 the second) and
/// the distance to each.
pub fn bvh_aligned_node_intersect(kg: &KernelGlobals, p: [f32; 3], idir: [f32; 3], tmin: f32, tmax: f32, node_addr: usize, visibility: u32) -> (u32, [f32; 2]) {
    let node = &kg.scene.bvh_node;
    let cnodes = node[node_addr];
    let boxes = [node[node_addr + 1], node[node_addr + 2], node[node_addr + 3]];
    let mut near_far = [(0.0f32, 0.0f32); 2];
    for child in 0..2 {
        let (mut near, mut far) = (tmin, tmax);
        for axis in 0..3 {
            let lo = (float(boxes[axis][child]) - p[axis]) * idir[axis];
            let hi = (float(boxes[axis][child + 2]) - p[axis]) * idir[axis];
            near = near.max(lo.min(hi));
            far = far.min(lo.max(hi));
        }
        near_far[child] = (near, far);
    }
    let crossed = |child: usize| near_far[child].1 >= near_far[child].0 && (cnodes[child] as u32) & visibility != 0;
    (u32::from(crossed(0)) | (u32::from(crossed(1)) << 1), [near_far[0].0, near_far[1].0])
}
