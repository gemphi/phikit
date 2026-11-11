// SPDX-License-Identifier: Apache-2.0
use super::*;

/// Run `run` on a builder over two objects, with room for `room`
/// primitives in the packed arrays, and give those arrays back.
fn build_in<R>(room: usize, param: BvhParam, run: impl FnOnce(&mut BvhBuild<'_>) -> R) -> (R, PackedBvh) {
    let mut pack = PackedBvh { prim_type: vec![0; room], prim_index: vec![0; room], prim_object: vec![0; room], ..PackedBvh::default() };
    let (object, mesh) = ([Object::new(0), Object::new(0)], [square()]);
    let out = run(&mut BvhBuild::new(&object, &mesh, &mut pack, param));
    (out, pack)
}

/// Every second reference of a row made a moving triangle.
fn mixed(count: usize) -> (BvhRange, Vec<BvhReference>) {
    let x: Vec<f32> = (0..count).map(|at| at as f32 * 2.0).collect();
    let (range, mut reference) = row(&x);
    reference.iter_mut().skip(1).step_by(2).for_each(|reference| reference.prim_type |= PrimitiveBit::new().motion);
    (range, reference)
}

#[test]
fn a_range_fits_a_leaf_while_no_kind_of_primitive_has_too_many() {
    let fits = |reference: &(BvhRange, Vec<BvhReference>), param: BvhParam| build_in(0, param, |build| build.range_within_max_leaf_size(&reference.0, &reference.1)).0;
    assert!(fits(&row(&[0.0; 8]), BvhParam::default()));
    assert!(!fits(&row(&[0.0; 9]), BvhParam::default()));
    // Four that stand still and four that move.
    assert!(fits(&mixed(8), BvhParam::default()));
    assert!(!fits(&mixed(8), BvhParam { max_motion_triangle_leaf_size: 3, ..BvhParam::default() }));
}

#[test]
fn a_leaf_takes_its_range_s_place_in_the_packed_arrays() {
    let (_, reference) = row(&[0.0, 2.0, 4.0, 6.0, 8.0, 10.0]);
    let range = BvhRange { start: 2, ..range_of(&reference[2..5]) };
    let (leaf, pack) = build_in(6, BvhParam { use_spatial_split: false, ..BvhParam::default() }, |build| build.create_leaf_node(&range, &reference));
    assert_eq!(leaf.kind, BvhNodeKind::Leaf { lo: 2, hi: 5 });
    assert_eq!((leaf.bound, leaf.visibility, leaf.time_from, leaf.time_to), (range.bound, 127, 0.0, 1.0));
    assert_eq!((pack.prim_index, pack.prim_type), (vec![0, 0, 2, 3, 4, 0], vec![0, 0, 1, 1, 1, 0]));
}

#[test]
fn primitives_of_two_types_get_a_leaf_each_under_one_node() {
    let (range, reference) = mixed(4);
    let (node, pack) = build_in(4, BvhParam { use_spatial_split: false, ..BvhParam::default() }, |build| build.create_leaf_node(&range, &reference));
    assert_eq!(node.bound, range.bound);
    assert_eq!(node.child().iter().map(|leaf| leaf.kind.clone()).collect::<Vec<_>>(), [BvhNodeKind::Leaf { lo: 0, hi: 2 }, BvhNodeKind::Leaf { lo: 2, hi: 4 }]);
    // Those that stand still first, then those that move.
    assert_eq!((pack.prim_index, pack.prim_type), (vec![0, 2, 1, 3], vec![1, 1, 65, 65]));
    assert_eq!((node.child()[0].bound.max[0], node.child()[1].bound.min[0]), (5.0, 2.0));
}

#[test]
fn when_primitives_may_be_cut_leaves_take_the_arrays_up_as_they_are_made() {
    let (_, reference) = row(&[0.0, 2.0, 4.0, 6.0, 8.0]);
    let (first, second) = (BvhRange { start: 3, ..range_of(&reference[3..]) }, range_of(&reference[..3]));
    let ((first, second, free), pack) = build_in(0, BvhParam::default(), |build| (build.create_leaf_node(&first, &reference), build.create_leaf_node(&second, &reference), build.spatial_free_index));
    assert_eq!((first.kind, second.kind, free), (BvhNodeKind::Leaf { lo: 0, hi: 2 }, BvhNodeKind::Leaf { lo: 2, hi: 5 }, 5));
    assert_eq!(pack.prim_index, [3, 4, 0, 1, 2]);
}

#[test]
fn each_instanced_object_gets_a_leaf_of_its_own() {
    let (range, mut reference) = row(&[0.0, 2.0, 4.0]);
    for (at, reference) in reference.iter_mut().enumerate() {
        (reference.prim_index, reference.prim_object, reference.prim_type) = (-1, at.min(1) as i32, 0);
    }
    let (node, pack) = build_in(3, BvhParam { use_spatial_split: false, top_level: true, ..BvhParam::default() }, |build| build.create_leaf_node(&range, &reference));
    // One, then a node of the other two.
    assert_eq!((node.subtree_size(BvhStat::LeafCount), node.subtree_size(BvhStat::Depth), node.bound), (3, 3, range.bound));
    assert_eq!(node.child()[0].kind, BvhNodeKind::Leaf { lo: 0, hi: 1 });
    assert_eq!(node.child()[1].child().iter().map(BvhNode::triangle_count).collect::<Vec<_>>(), [1, 1]);
    assert_eq!((pack.prim_index, pack.prim_object), (vec![-1; 3], vec![0, 1, 1]));
}

#[test]
fn a_range_of_nothing_is_an_empty_leaf() {
    let (leaf, _) = build_in(0, BvhParam { use_spatial_split: false, ..BvhParam::default() }, |build| build.create_leaf_node(&BvhRange::default(), &[]));
    assert_eq!((leaf.kind.clone(), leaf.visibility, leaf.bound.valid()), (BvhNodeKind::Leaf { lo: 0, hi: 0 }, 0, false));
}

#[test]
fn a_leaf_holds_for_the_spans_of_its_references() {
    let (range, mut reference) = row(&[0.0, 2.0]);
    (reference[0].time_from, reference[0].time_to) = (0.25, 0.5);
    (reference[1].time_from, reference[1].time_to) = (0.5, 0.75);
    let (leaf, pack) = build_in(2, BvhParam { use_spatial_split: false, ..BvhParam::default() }, |build| {
        build.need_prim_time = true;
        build.pack.prim_time.resize(2, [0.0; 2]);
        build.create_leaf_node(&range, &reference)
    });
    assert_eq!((leaf.time_from, leaf.time_to), (0.25, 0.75));
    assert_eq!(pack.prim_time, [[0.25, 0.5], [0.5, 0.75]]);
}
