// SPDX-FileCopyrightText: 2009-2010 NVIDIA Corporation
// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use super::*;
use crate::util::Transform;

fn gather(object: &[Object], mesh: &[Mesh], param: BvhParam) -> (BvhRange, Vec<BvhReference>) {
    BvhBuild::new(object, mesh, &mut PackedBvh::default(), param).add_reference()
}

#[test]
fn a_mesh_gives_a_reference_for_each_triangle() {
    let (range, reference) = gather(&[Object::new(0)], &[square()], BvhParam::default());
    assert_eq!((range.start, range.size, reference.len()), (0, 2, 2));
    assert_eq!((range.bound.min, range.bound.max), ([0.0; 3], [1.0, 1.0, 0.0]));
    // Both triangles' boxes are the whole square: one doubled centre.
    assert_eq!((range.cent_bound.min, range.cent_bound.max), ([1.0, 1.0, 0.0], [1.0, 1.0, 0.0]));
    assert_eq!(reference.iter().map(|reference| (reference.prim_index, reference.prim_object, reference.prim_type)).collect::<Vec<_>>(), [(0, 0, 1), (1, 0, 1)]);
    assert!(reference.iter().all(|reference| (reference.time_from, reference.time_to) == (0.0, 1.0)));
}

#[test]
fn a_triangle_with_a_corner_that_is_not_finite_is_left_out() {
    let mut mesh = square();
    mesh.geometry.position_mut()[3] = [f32::INFINITY, 1.0, 0.0];
    let (range, reference) = gather(&[Object::new(0)], &[mesh], BvhParam::default());
    assert_eq!((range.size, reference[0].prim_index), (1, 0));
    assert!(range.bound.valid());
}

#[test]
fn a_moving_triangle_has_one_box_around_all_its_steps() {
    let (range, reference) = gather(&[Object::new(0)], &[moving_square()], BvhParam::default());
    let bit = PrimitiveBit::new();
    assert_eq!((reference.len(), reference[0].prim_type), (2, bit.triangle | bit.motion));
    assert_eq!((range.bound.min[2], range.bound.max[2], reference[1].bound.min[2], reference[1].bound.max[2]), (-1.0, 1.0, -1.0, 1.0));
}

#[test]
fn with_time_steps_it_has_a_box_for_each_span_unless_primitives_are_cut() {
    let stepped = BvhParam { motion_triangle_step_count: 1, use_spatial_split: false, ..BvhParam::default() };
    let (_, reference) = gather(&[Object::new(0)], &[moving_square()], stepped);
    let span = |at: usize| (reference[at].prim_index, reference[at].time_from, reference[at].time_to, reference[at].bound.min[2], reference[at].bound.max[2]);
    assert_eq!(reference.len(), 4);
    assert_eq!((span(0), span(1)), ((0, 0.0, 0.5, -1.0, 0.0), (0, 0.5, 1.0, 0.0, 1.0)));
    assert_eq!(span(3), (1, 0.5, 1.0, 0.0, 1.0));
    let (_, reference) = gather(&[Object::new(0)], &[moving_square()], BvhParam { use_spatial_split: true, ..stepped });
    assert_eq!(reference.len(), 2);
}

#[test]
fn at_the_top_an_instance_is_one_reference_and_applied_geometry_its_triangles() {
    let mut mesh = [square(), square()];
    mesh.iter_mut().for_each(Mesh::compute_bound);
    mesh[1].geometry.transform_applied = true;
    // The last object's box is never worked out: no ray can reach it.
    let mut object = [Object::new(0), Object::new(1), Object::new(0)];
    assert!(object[0].set_transform(Transform::translate(5.0, 0.0, 0.0)));
    object[0].compute_bound(&mesh[0].geometry, false);
    object[1].compute_bound(&mesh[1].geometry, false);
    let (range, reference) = gather(&object, &mesh, BvhParam { top_level: true, ..BvhParam::default() });
    assert_eq!(reference.iter().map(|reference| (reference.prim_index, reference.prim_object, reference.prim_type)).collect::<Vec<_>>(), [(-1, 0, 0), (0, 1, 1), (1, 1, 1)]);
    assert_eq!((reference[0].bound.min[0], reference[0].bound.max[0], range.bound.max[0]), (5.0, 6.0, 6.0));
    // Below the top, every object's triangles are taken, reachable or not.
    assert_eq!(gather(&object, &mesh, BvhParam::default()).1.len(), 6);
}

#[test]
fn nothing_to_reference_leaves_a_point_at_the_origin() {
    let (range, reference) = gather(&[Object::new(0)], &[Mesh::new()], BvhParam::default());
    assert!(reference.is_empty());
    assert_eq!((range.size, range.bound.min, range.bound.max), (0, [0.0; 3], [0.0; 3]));
}
