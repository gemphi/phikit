// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::kernel::bvh::test::*;
use crate::kernel::geom::{bvh_clamp_direction, bvh_inverse_direction};

/// The children the root of a strip's own tree lets a ray along z into,
/// and how far along the ray each starts. The scene's tree above it holds
/// the strip as one instance.
fn root(rig: &Rig, x: f32, visibility: u32) -> (u32, [f32; 2]) {
    let kg = KernelGlobal::of(&rig.dscene);
    let idir = bvh_inverse_direction(bvh_clamp_direction([0.0, 0.0, 1.0]));
    bvh_aligned_node_intersect(&kg, [x, 0.5, -3.0], idir, 0.0, 100.0, rig.dscene.object_node[0], visibility)
}

#[test]
fn a_ray_enters_the_child_whose_box_it_crosses() {
    // Sixteen squares in a row: the root's two children are its two ends.
    let rig = rig_of(vec![strip(16)]);
    let (low, high) = (root(&rig, 0.5, all()), root(&rig, 30.5, all()));
    assert!(low.0 == 1 || low.0 == 2, "one child at one end: {low:?}");
    assert!(high.0 == 1 || high.0 == 2, "one child at the other: {high:?}");
    assert_ne!(low.0, high.0);
    // The box is flat at z = 0, three units along the ray.
    let entered = low.1[(low.0 - 1) as usize];
    assert!((entered - 3.0).abs() < 1e-4, "{low:?}");
    // Beside the strip, neither.
    assert_eq!(root(&rig, -5.0, all()).0, 0);
}

#[test]
fn a_scene_of_one_object_has_a_root_with_an_empty_child() {
    // A tree is never a leaf alone: the one instance is the root's second
    // child, and the first is a box nothing can enter.
    let rig = rig_of(vec![strip(16)]);
    let kg = KernelGlobal::of(&rig.dscene);
    let idir = bvh_inverse_direction(bvh_clamp_direction([0.0, 0.0, 1.0]));
    let at = |x: f32| bvh_aligned_node_intersect(&kg, [x, 0.5, -3.0], idir, 0.0, 100.0, kg.bvh.root, all()).0;
    assert_eq!((at(0.5), at(30.5), at(-5.0)), (2, 2, 0));
}

#[test]
fn a_child_is_missed_when_the_ray_ends_before_it_or_starts_after_it() {
    let rig = rig_of(vec![strip(16)]);
    let kg = KernelGlobal::of(&rig.dscene);
    let idir = bvh_inverse_direction(bvh_clamp_direction([0.0, 0.0, 1.0]));
    let between = |tmin: f32, tmax: f32| bvh_aligned_node_intersect(&kg, [0.5, 0.5, -3.0], idir, tmin, tmax, rig.dscene.object_node[0], all()).0;
    assert!(between(0.0, 100.0) != 0);
    assert_eq!((between(0.0, 2.0), between(4.0, 100.0)), (0, 0));
}

#[test]
fn a_child_no_ray_of_this_kind_may_see_counts_as_missed() {
    let mut rig = Rig::new(vec![strip(16)]);
    let camera = flag("PATH_RAY_VISIBILITY_CAMERA");
    rig.object[0].node.set("visibility", crate::graph::Value::Uint(all() & !camera));
    rig.update();
    // The object's visibility is on the scene's tree, where the object is.
    let kg = KernelGlobal::of(&rig.dscene);
    let idir = bvh_inverse_direction(bvh_clamp_direction([0.0, 0.0, 1.0]));
    let seen = |visibility: u32| bvh_aligned_node_intersect(&kg, [0.5, 0.5, -3.0], idir, 0.0, 100.0, kg.bvh.root, visibility).0;
    assert_eq!((seen(camera), seen(all())), (0, 2));
}
