// SPDX-FileCopyrightText: 2009-2010 NVIDIA Corporation
// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::bvh::*;
use crate::util::BoundBox;

fn cube(x: f32) -> BoundBox {
    BoundBox { min: [x, 0.0, 0.0], max: [x + 1.0, 1.0, 1.0] }
}

/// Two leaves side by side, of two primitives and of one.
fn pair() -> BvhNode {
    BvhNode::inner(cube(0.0).merge(&cube(1.0)), BvhNode::leaf(cube(0.0), 0b01, 0, 2), BvhNode::leaf(cube(1.0), 0b10, 2, 3))
}

#[test]
fn a_leaf_has_primitives_and_no_children() {
    let leaf = BvhNode::leaf(cube(0.0), 5, 4, 9);
    assert!(leaf.is_leaf() && leaf.child().is_empty());
    assert_eq!((leaf.triangle_count(), leaf.visibility, leaf.time_from, leaf.time_to), (5, 5, 0.0, 1.0));
}

#[test]
fn an_inner_node_is_seen_by_the_rays_that_see_either_child() {
    let node = pair();
    assert!(!node.is_leaf());
    assert_eq!((node.child().len(), node.triangle_count(), node.visibility), (2, 0, 0b11));
}

#[test]
fn a_subtree_is_counted_by_what_is_asked() {
    let node = BvhNode::inner(cube(0.0).merge(&cube(2.0)), pair(), BvhNode::leaf(cube(2.0), 1, 3, 7));
    let size = |stat: BvhStat| node.subtree_size(stat);
    assert_eq!((size(BvhStat::NodeCount), size(BvhStat::InnerCount), size(BvhStat::LeafCount)), (5, 2, 3));
    assert_eq!((size(BvhStat::TriangleCount), size(BvhStat::ChildNodeCount), size(BvhStat::Depth)), (7, 4, 3));
    assert_eq!(BvhNode::leaf(cube(0.0), 1, 0, 0).subtree_size(BvhStat::Depth), 1);
}

#[test]
fn a_ray_pays_for_a_child_as_often_as_the_child_s_area_is_of_the_node_s() {
    // The node's area is 10 and each cube's 6: the node of two, then
    // six tenths of two primitives and of one.
    let cost = pair().subtree_sah_cost(&BvhParam::default(), 1.0);
    assert!((cost - 3.8).abs() < 1e-6, "{cost}");
}

#[test]
fn a_node_with_no_visibility_takes_its_children_s() {
    let mut node = pair();
    node.visibility = 0;
    assert_eq!((node.update_visibility(), node.visibility), (0b11, 0b11));
    // One that has some keeps it.
    node.visibility = 0b100;
    assert_eq!(node.update_visibility(), 0b100);
}

#[test]
fn an_inner_node_holds_for_as_long_as_its_children_do() {
    let mut first = BvhNode::leaf(cube(0.0), 1, 0, 1);
    let mut second = BvhNode::leaf(cube(1.0), 1, 1, 2);
    (first.time_from, first.time_to) = (0.25, 0.5);
    (second.time_from, second.time_to) = (0.5, 0.75);
    let mut node = BvhNode::inner(cube(0.0).merge(&cube(1.0)), first, second);
    assert_eq!((node.time_from, node.time_to), (0.0, 1.0));
    node.update_time();
    assert_eq!((node.time_from, node.time_to), (0.25, 0.75));
}
