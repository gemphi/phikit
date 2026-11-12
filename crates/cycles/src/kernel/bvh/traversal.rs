// SPDX-FileCopyrightText: 2009-2010 NVIDIA Corporation
// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/bvh/traversal.h` (`BVH_FUNCTION_FULL_
//! NAME(BVH)`, lines 20-250): the closest hit of a ray with the scene's
//! tree. The ray enters an instance through its object's inverse
//! transform (`bvh_instance_push`, `geom/object.h:462`) and leaves it
//! again by restoring the world ray (`bvh_instance_pop`).
use super::nodes::bvh_aligned_node_intersect;
use super::util::intersection_skip_self_shadow;
use crate::kernel::geom::{bvh_clamp_direction, bvh_inverse_direction, triangle_intersect};
use crate::kernel::globals::KernelGlobals;
use crate::kernel::types::{Intersection, NONE, Ray};

/// `ENTRYPOINT_SENTINEL`: the bottom of the stack, the end of a walk.
const ENTRYPOINT_SENTINEL: i32 = 0x7654_3210;
/// `BVH_STACK_SIZE`: 64 object, 64 mesh and 64 for object node splitting.
const BVH_STACK_SIZE: usize = 192;
/// `PRIMITIVE_ALL`.
const PRIMITIVE_ALL: i32 = 127;

/// `bvh_intersect`: the closest hit of `ray` among the primitives that
/// `visibility` sees, in `isect`. False, and `isect.prim == NONE`, for none.
pub fn bvh_intersect(kg: &KernelGlobals, ray: &Ray, isect: &mut Intersection, visibility: u32) -> bool {
    let scene = &*kg.scene;
    let mut traversal_stack = [0i32; BVH_STACK_SIZE];
    traversal_stack[0] = ENTRYPOINT_SENTINEL;
    let mut stack_ptr: i32 = 0;
    let mut node_addr = kg.bvh_root;
    let mut p = ray.p;
    let mut dir = bvh_clamp_direction(ray.d);
    let mut idir = bvh_inverse_direction(dir);
    let tmin = ray.tmin;
    let mut object = NONE;
    isect.t = ray.tmax;
    isect.u = 0.0;
    isect.v = 0.0;
    isect.prim = NONE;
    isect.object = NONE;
    loop {
        loop {
            // Traverse the inner nodes.
            while node_addr >= 0 && node_addr != ENTRYPOINT_SENTINEL {
                let (mask, dist) = bvh_aligned_node_intersect(kg, p, idir, tmin, isect.t, node_addr as usize, visibility);
                let cnodes = scene.bvh_node[node_addr as usize];
                node_addr = cnodes[2];
                let mut node_addr_child1 = cnodes[3];
                if mask == 3 {
                    // Both children were crossed: push the farther one.
                    if dist[1] < dist[0] {
                        std::mem::swap(&mut node_addr, &mut node_addr_child1);
                    }
                    stack_ptr += 1;
                    traversal_stack[stack_ptr as usize] = node_addr_child1;
                } else if mask == 2 {
                    node_addr = node_addr_child1;
                } else if mask == 0 {
                    node_addr = traversal_stack[stack_ptr as usize];
                    stack_ptr -= 1;
                }
            }
            // A leaf: its primitives or an instance.
            if node_addr < 0 {
                let leaf = scene.bvh_leaf_node[(-node_addr - 1) as usize];
                let prim_addr = leaf[0];
                if prim_addr >= 0 {
                    let prim_addr2 = leaf[1];
                    let kind = leaf[3];
                    node_addr = traversal_stack[stack_ptr as usize];
                    stack_ptr -= 1;
                    for prim_addr in prim_addr as usize..prim_addr2 as usize {
                        let prim_object = if object == NONE { scene.prim_object[prim_addr] } else { object };
                        let prim = scene.prim_index[prim_addr];
                        if intersection_skip_self_shadow(&ray.kind, prim_object, prim) {
                            continue;
                        }
                        if kind & PRIMITIVE_ALL == 1 {
                            let hit = triangle_intersect(kg, isect, p, dir, tmin, visibility, prim_object, prim, prim_addr);
                            // A shadow ray ends at the first hit.
                            if hit && visibility & crate::kernel::types::PATH_RAY_VISIBILITY_SHADOW_OPAQUE != 0 {
                                return true;
                            }
                        }
                    }
                } else {
                    // An instance: the ray goes into the object's space.
                    object = scene.prim_object[(-prim_addr - 1) as usize];
                    let itfm = &kg.object[object as usize].itfm;
                    p = itfm.point(ray.p);
                    dir = bvh_clamp_direction(itfm.direction(ray.d));
                    idir = bvh_inverse_direction(dir);
                    stack_ptr += 1;
                    traversal_stack[stack_ptr as usize] = ENTRYPOINT_SENTINEL;
                    node_addr = scene.object_node[object as usize];
                }
            }
            if node_addr == ENTRYPOINT_SENTINEL {
                break;
            }
        }
        if stack_ptr >= 0 {
            // Leave the instance: the world ray again.
            p = ray.p;
            dir = bvh_clamp_direction(ray.d);
            idir = bvh_inverse_direction(dir);
            object = NONE;
            node_addr = traversal_stack[stack_ptr as usize];
            stack_ptr -= 1;
        }
        if node_addr == ENTRYPOINT_SENTINEL {
            break;
        }
    }
    isect.prim != NONE
}
