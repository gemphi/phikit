// SPDX-FileCopyrightText: 2009-2010 NVIDIA Corporation
// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Tests of this module.


pub(crate) use crate::bvh::test::*;
pub(crate) use crate::graph::Value;

/// Build the tree of `mesh`, placed by one object: its root, the packed
/// arrays, and the settings the builder ended with.
pub(crate) fn run(mesh: &Mesh, param: BvhParam) -> (BvhNode, PackedBvh, BvhParam) {
    let (mut pack, object) = (PackedBvh::default(), [Object::new(0)]);
    let mut build = BvhBuild::new(&object, std::slice::from_ref(mesh), &mut pack, param);
    let root = build.run();
    let param = build.param;
    (root, pack, param)
}

pub(crate) fn leaf_list(node: &BvhNode) -> Vec<&BvhNode> {
    match node.is_leaf() {
        true => vec![node],
        false => node.child().iter().flat_map(leaf_list).collect(),
    }
}

pub(crate) fn sorted(mut index: Vec<i32>) -> Vec<i32> {
    index.sort_unstable();
    index.dedup();
    index
}

pub(crate) fn unsplit() -> BvhParam {
    BvhParam { use_spatial_split: false, ..BvhParam::default() }
}

#[test]
fn a_type_s_place_counts_each_shape_with_and_without_motion() {
    let bit = PrimitiveBit::new();
    assert_eq!((bit.index(bit.triangle), bit.index(bit.triangle | bit.motion), bit.index(bit.point)), (0, 1, 6));
    // A curve's segment, packed above the type, is not looked at.
    assert_eq!(bit.index(2 | (5 << 7)), 2);
}

#[test]
fn every_triangle_of_a_mesh_is_in_one_leaf_of_no_more_than_eight() {
    let mesh = strip(40);
    let (root, pack, _) = run(&mesh, unsplit());
    assert_eq!((root.subtree_size(BvhStat::TriangleCount), pack.prim_index.len()), (80, 80));
    assert_eq!(sorted(pack.prim_index.clone()), (0..80).collect::<Vec<_>>());
    assert!(leaf_list(&root).iter().all(|leaf| (1..=8).contains(&leaf.triangle_count())));
    assert!(pack.prim_object.iter().all(|object| *object == 0) && pack.prim_type.iter().all(|kind| *kind == PrimitiveBit::new().triangle));
    assert!(pack.prim_time.is_empty());
    assert_eq!((root.bound.min, root.bound.max, root.visibility), ([0.0; 3], [79.0, 1.0, 0.0], 127));
    assert!(root.subtree_size(BvhStat::Depth) <= BvhLimit::get().max_depth + 1);
}

#[test]
fn a_leaf_s_box_holds_its_triangles_and_a_node_s_its_children() {
    let mesh = strip(40);
    let (root, pack, _) = run(&mesh, unsplit());
    fn check(node: &BvhNode, mesh: &Mesh, pack: &PackedBvh) {
        match node.kind {
            BvhNodeKind::Leaf { lo, hi } => assert!((lo..hi).all(|place| holds(&node.bound, &triangle_box(mesh, pack.prim_index[place] as usize)))),
            BvhNodeKind::Inner { .. } => node.child().iter().for_each(|child| {
                assert!(holds(&node.bound, &child.bound));
                check(child, mesh, pack);
            }),
        }
    }
    check(&root, &mesh, &pack);
}

#[test]
fn a_triangle_that_spoils_every_cut_is_cut_in_two() {
    let mesh = strip_under_a_sliver();
    let (root, pack, _) = run(&mesh, unsplit());
    assert_eq!((pack.prim_index.len(), root.subtree_size(BvhStat::TriangleCount)), (33, 33));

    let (root, pack, param) = run(&mesh, BvhParam::default());
    assert!(param.use_spatial_split);
    // The long triangle, the last, is in several leaves; none is missed.
    assert!(pack.prim_index.iter().filter(|index| **index == 32).count() > 1, "{:?}", pack.prim_index);
    assert_eq!(sorted(pack.prim_index.clone()), (0..33).collect::<Vec<_>>());
    assert_eq!(root.subtree_size(BvhStat::TriangleCount), pack.prim_index.len());
    // A cut tree costs a ray less.
    let whole = run(&mesh, unsplit()).0.subtree_sah_cost(&param, 1.0);
    assert!(root.subtree_sah_cost(&param, 1.0) < whole);
}

#[test]
fn a_mesh_that_moves_is_never_cut() {
    let (_, pack, param) = run(&moving_square(), BvhParam::default());
    assert!(!param.use_spatial_split);
    let bit = PrimitiveBit::new();
    assert_eq!(pack.prim_type, [bit.triangle | bit.motion; 2]);
}

#[test]
fn with_time_steps_each_leaf_holds_for_a_span_of_the_shutter() {
    let (root, pack, _) = run(&moving_square(), BvhParam { motion_triangle_step_count: 1, ..BvhParam::default() });
    assert_eq!(pack.prim_time.len(), 4);
    assert!(pack.prim_time.iter().all(|span| *span == [0.0, 0.5] || *span == [0.5, 1.0]));
    assert_eq!((root.time_from, root.time_to), (0.0, 1.0));
}

#[test]
fn the_top_level_has_an_inner_node_even_over_one_object() {
    let mut mesh = [square()];
    mesh[0].compute_bound();
    let mut object = [Object::new(0)];
    object[0].node.set("visibility", Value::Uint(3));
    object[0].compute_bound(&mesh[0].geometry, false);
    let mut pack = PackedBvh::default();
    let root = BvhBuild::new(&object, &mesh, &mut pack, BvhParam { top_level: true, ..BvhParam::default() }).run();
    assert!(!root.is_leaf());
    assert_eq!(root.child().iter().map(BvhNode::triangle_count).collect::<Vec<_>>(), [0, 1]);
    // The object is one primitive, with no triangle of its own here.
    assert_eq!((pack.prim_index, pack.prim_object, pack.prim_type, root.visibility), (vec![-1], vec![0], vec![0], 3));
}

#[test]
fn one_triangle_or_none_is_a_tree_of_one_leaf() {
    let (root, pack, _) = run(&mesh_of(&[[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]], &[[0, 1, 2]]), BvhParam::default());
    assert!(root.is_leaf());
    assert_eq!((root.triangle_count(), pack.prim_index), (1, vec![0]));
    let (root, pack, _) = run(&Mesh::new(), BvhParam::default());
    assert!(root.is_leaf() && root.triangle_count() == 0 && pack.prim_index.is_empty());
}
