// SPDX-License-Identifier: Apache-2.0
use crate::kernel::geom::triangle::*;
use crate::kernel::{Intersection, KernelGlobal, flag};
use crate::tests::kernel::bvh::{all, rig_of, square};
use crate::tests::kernel::near;

#[test]
fn a_hit_is_written_with_the_triangle_and_its_object() {
    let rig = rig_of(vec![square(), square()]);
    let kg = KernelGlobal::of(&rig.dscene);
    // The second mesh's second triangle is the scene's fourth, and its
    // vertices come after the first mesh's four.
    let (object, prim) = (1, 3);
    let prim_addr = rig.dscene.prim_index.iter().position(|index| *index == prim).expect("the tree holds every triangle") as i32;
    let mut isect = Intersection::default();
    assert!(triangle_intersect(&kg, &mut isect, [0.25, 0.75, -2.0], [0.0, 0.0, 1.0], 0.0, 10.0, all(), object, prim, prim_addr));
    assert_eq!((isect.prim, isect.object, isect.kind), (prim, object, flag("PRIMITIVE_TRIANGLE") as i32));
    assert!(near(&[isect.t, isect.u, isect.v], &[2.0, 0.25, 0.5]), "{isect:?}");
}

#[test]
fn a_miss_or_a_triangle_the_ray_may_not_see_leaves_the_hit_as_it_was() {
    let rig = rig_of(vec![square()]);
    let kg = KernelGlobal::of(&rig.dscene);
    let before = Intersection { t: 7.0, prim: 9, ..Intersection::default() };
    let mut isect = before;
    let prim_addr = rig.dscene.prim_index.iter().position(|index| *index == 0).expect("the tree holds every triangle") as i32;
    // Beside the triangle, beyond the ray's end, and for no kind of ray.
    assert!(!triangle_intersect(&kg, &mut isect, [0.25, 0.75, -2.0], [0.0, 0.0, 1.0], 0.0, 10.0, all(), 0, 0, prim_addr));
    assert!(!triangle_intersect(&kg, &mut isect, [0.75, 0.25, -2.0], [0.0, 0.0, 1.0], 0.0, 1.0, all(), 0, 0, prim_addr));
    assert!(!triangle_intersect(&kg, &mut isect, [0.75, 0.25, -2.0], [0.0, 0.0, 1.0], 0.0, 10.0, 0, 0, 0, prim_addr));
    assert_eq!(isect, before);
    assert!(triangle_intersect(&kg, &mut isect, [0.75, 0.25, -2.0], [0.0, 0.0, 1.0], 0.0, 10.0, all(), 0, 0, prim_addr));
}
