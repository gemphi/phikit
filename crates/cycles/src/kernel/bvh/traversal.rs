// SPDX-FileCopyrightText: 2009-2010 NVIDIA Corporation
// SPDX-FileCopyrightText: 2009-2012 Intel Corporation
// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/bvh/traversal.h` (lines 24-261), which
//! Cycles adapted from code of NVIDIA Corporation: a ray walked through
//! the scene's tree to its nearest hit.
//!
//! Cycles compiles that file once for each mix of features. This is the
//! one with none of them: triangles that do not move. Not ported: motion
//! triangles, curves, points, shadow linking, and the walks of the other
//! files (`shadow_all.h`, `local.h`, `volume.h`).

use super::{bvh_aligned_node_intersect, intersection_skip_self_shadow};
use crate::kernel::geom::triangle::triangle_intersect;
use crate::kernel::geom::{bvh_clamp_direction, bvh_instance_pop, bvh_instance_push, bvh_inverse_direction};
use crate::kernel::{Intersection, KernelGlobal, Ray};

/// The walk's place in the tree, and the ray as that place sees it.
struct Walk {
    /// The nodes still to visit, the mark of an end under each tree's.
    /// Cycles keeps 192 in an array; this grows as it must.
    stack: Vec<i32>,
    /// The instance the walk is inside, or none.
    object: i32,
    p: [f32; 3],
    dir: [f32; 3],
    idir: [f32; 3],
}

impl Walk {
    /// The node to visit next: the last one put aside, or the mark.
    fn pop(&mut self, sentinel: i32) -> i32 {
        self.stack.pop().unwrap_or(sentinel)
    }
}

/// The inner nodes from `node_addr` down to a leaf or the end of the
/// tree: of two children the ray enters, the nearer is visited first and
/// the other put aside.
fn descend(kg: &KernelGlobal, walk: &mut Walk, mut node_addr: i32, tmin: f32, tmax: f32, visibility: u32) -> i32 {
    let sentinel = kg.bvh.sentinel;
    while node_addr >= 0 && node_addr != sentinel {
        let (mask, dist) = bvh_aligned_node_intersect(kg, walk.p, walk.idir, tmin, tmax, node_addr, visibility);
        let [_, _, first, second] = kg.scene.bvh_node[node_addr as usize];
        node_addr = match (mask, dist[1] < dist[0]) {
            (3, true) => {
                walk.stack.push(first);
                second
            }
            (3, false) => {
                walk.stack.push(second);
                first
            }
            (2, _) => second,
            (1, _) => first,
            _ => walk.pop(sentinel),
        };
    }
    node_addr
}

/// The primitives of a leaf, from `start` to `end` in the tree's arrays,
/// each against the ray. True when a shadow ray needs to look no further.
fn leaf(kg: &KernelGlobal, walk: &Walk, ray: &Ray, isect: &mut Intersection, visibility: u32, range: (i32, i32), kind: u32) -> bool {
    let (scene, bvh) = (kg.scene, &kg.bvh);
    for prim_addr in range.0..range.1 {
        let prim_object = match walk.object == bvh.object_none {
            true => scene.prim_object[prim_addr as usize],
            false => walk.object,
        };
        let prim = scene.prim_index[prim_addr as usize];
        if intersection_skip_self_shadow(&ray.own, prim_object, prim) {
            continue;
        }
        let hit = match kind & bvh.primitive_all == bvh.primitive_triangle {
            true => triangle_intersect(kg, isect, walk.p, walk.dir, ray.tmin, isect.t, visibility, prim_object, prim, prim_addr),
            false => false,
        };
        if hit && visibility & bvh.shadow_opaque != 0 {
            return true;
        }
    }
    false
}

/// `scene_intersect` for Cycles' own tree, `bvh_intersect`: the nearest
/// thing `ray` hits that a ray of this `visibility` may see, written into
/// `isect`. False when it hits nothing.
pub fn bvh_intersect(kg: &KernelGlobal, ray: &Ray, isect: &mut Intersection, visibility: u32) -> bool {
    let (scene, bvh) = (kg.scene, &kg.bvh);
    let sentinel = bvh.sentinel;
    let dir = bvh_clamp_direction(ray.d);
    let mut walk = Walk { stack: vec![sentinel], object: bvh.object_none, p: ray.p, dir, idir: bvh_inverse_direction(dir) };
    let mut node_addr = bvh.root;
    (isect.t, isect.u, isect.v, isect.prim, isect.object) = (ray.tmax, 0.0, 0.0, bvh.prim_none, bvh.object_none);

    loop {
        // One tree: the scene's, or inside an instance its geometry's.
        loop {
            node_addr = descend(kg, &mut walk, node_addr, ray.tmin, isect.t, visibility);
            if node_addr < 0 {
                let [start, end, _, kind] = scene.bvh_leaf_node[(-node_addr - 1) as usize];
                match start >= 0 {
                    true => {
                        node_addr = walk.pop(sentinel);
                        if leaf(kg, &walk, ray, isect, visibility, (start, end), kind as u32) {
                            return true;
                        }
                    }
                    // An instance: go on in its geometry's tree, with the
                    // ray as the object sees it.
                    false => {
                        walk.object = scene.prim_object[(-start - 1) as usize];
                        (walk.p, walk.dir, walk.idir) = bvh_instance_push(kg, walk.object, ray);
                        walk.stack.push(sentinel);
                        node_addr = scene.object_node[walk.object as usize];
                    }
                }
            }
            if node_addr == sentinel {
                break;
            }
        }
        // The end of an instance's tree: back to the scene's.
        if !walk.stack.is_empty() {
            (walk.p, walk.dir, walk.idir) = bvh_instance_pop(ray);
            walk.object = bvh.object_none;
            node_addr = walk.pop(sentinel);
        }
        if node_addr == sentinel {
            break;
        }
    }
    isect.prim != bvh.prim_none
}
