// SPDX-FileCopyrightText: 2009-2010 NVIDIA Corporation
// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Tests of `src/bvh/bvh2`, one file for each file there. This file
//! tests `mod.rs`.

mod instance;
mod refit;

use super::*;

/// Walk the tree of `mesh` alone and give the triangle each place
/// reached stands for.
pub(crate) fn reached(bvh: &Bvh2, mesh: &Mesh) -> Vec<i32> {
    // A root of -1 is the first leaf, with every bit turned.
    let (place, _) = walk(&bvh.pack, bvh.pack.root_index, &|place| triangle_box(mesh, bvh.pack.prim_index[place] as usize));
    place.into_iter().map(|place| bvh.pack.prim_index[place]).collect()
}

fn unsplit() -> BvhParam {
    BvhParam { use_spatial_split: false, ..BvhParam::default() }
}

#[test]
fn a_walk_of_the_packed_tree_reaches_every_triangle_once() {
    let mesh = strip(40);
    let bvh = tree_of(&mesh, unsplit());
    let mut triangle = reached(&bvh, &mesh);
    triangle.sort_unstable();
    assert_eq!(triangle, (0..80).collect::<Vec<_>>());
    assert_eq!(bvh.pack.root_index, 0);
}

#[test]
fn an_inner_node_is_four_words_and_a_leaf_one() {
    let bvh = tree_of(&strip(40), unsplit());
    let leaf_count = bvh.pack.leaf_node.len();
    // A binary tree has one inner node fewer than it has leaves.
    assert_eq!(bvh.pack.node.len(), (leaf_count - 1) * 4);
    assert!(leaf_count >= 10);
    // Each leaf: its primitives, the rays that see it, their type.
    assert!(bvh.pack.leaf_node.iter().all(|leaf| leaf[0] < leaf[1] && leaf[1] - leaf[0] <= 8 && leaf[2] == 127 && leaf[3] == 1));
    assert_eq!(bvh.pack.leaf_node.iter().map(|leaf| leaf[1] - leaf[0]).sum::<i32>(), 80);
}

#[test]
fn the_root_keeps_its_two_children_s_boxes_and_where_they_are() {
    let mesh = strip(40);
    let bvh = tree_of(&mesh, unsplit());
    let root = &bvh.pack.node[..4];
    assert_eq!((root[0][0], root[0][1]), (127, 127));
    // The two halves of the strip, which both have nodes of their own.
    assert!(root[0][2] > 0 && root[0][3] > 0 && root[0][2] % 4 == 0 && root[0][3] % 4 == 0);
    let around = kept_box(root, 0).merge(&kept_box(root, 1));
    assert_eq!((around.min, around.max), ([0.0; 3], [79.0, 1.0, 0.0]));
    assert!(kept_box(root, 0).max[0] < kept_box(root, 1).min[0] || kept_box(root, 1).max[0] < kept_box(root, 0).min[0]);
}

#[test]
fn a_tree_of_one_leaf_has_no_node_to_start_at() {
    let bvh = tree_of(&square(), BvhParam::default());
    assert_eq!((bvh.pack.root_index, bvh.pack.node.len()), (-1, 0));
    assert_eq!(bvh.pack.leaf_node, [[0, 2, 127, 1]]);
    assert_eq!((&bvh.pack.prim_index, &bvh.pack.prim_visibility), (&vec![0, 1], &vec![127, 127]));
    let empty = tree_of(&Mesh::new(), BvhParam::default());
    assert_eq!((empty.pack.root_index, &empty.pack.leaf_node), (-1, &vec![[0; 4]]));
}

#[test]
fn a_triangle_cut_in_two_is_found_wherever_a_ray_meets_it() {
    let mesh = strip_under_a_sliver();
    let bvh = tree_of(&mesh, BvhParam::default());
    // A leaf's box holds only its part of a cut triangle, so the boxes
    // are not checked against whole triangles here.
    let (place, _) = walk(&bvh.pack, bvh.pack.root_index, &|_| BoundBox::EMPTY);
    let mut triangle: Vec<i32> = place.iter().map(|place| bvh.pack.prim_index[*place]).collect();
    assert!(triangle.len() > 33 && triangle.len() == bvh.pack.prim_index.len());
    triangle.sort_unstable();
    triangle.dedup();
    assert_eq!(triangle, (0..33).collect::<Vec<_>>());
    // Along the long triangle's lower edge and through its middle, the
    // boxes that hold a point lead to a leaf that has it.
    for step in 0..=62 {
        let x = step as f32 * 0.5;
        assert!(finds(&bvh.pack, 0, [x, 0.0, 0.0], 32) && finds(&bvh.pack, 0, [x, x / 62.0, 0.0], 32), "at {x}");
    }
    // And so for the middle of every small triangle.
    for index in 0..32 {
        let corner = mesh.triangle(index).corner(mesh.geometry.position());
        let middle = [0, 1, 2].map(|axis| (corner[0][axis] + corner[1][axis] + corner[2][axis]) / 3.0);
        assert!(finds(&bvh.pack, 0, middle, index as i32), "triangle {index}");
    }
}

#[test]
fn a_moving_mesh_s_boxes_hold_every_motion_step() {
    let mesh = moving_square();
    let bvh = tree_of(&mesh, BvhParam::default());
    assert_eq!(reached(&bvh, &mesh).len(), 2);
    assert_eq!(bvh.pack.leaf_node[0][3], 65);
    let stepped = tree_of(&mesh, BvhParam { motion_triangle_step_count: 2, ..BvhParam::default() });
    assert_eq!((stepped.pack.prim_index.len(), stepped.pack.prim_time.len()), (8, 8));
}

#[test]
fn a_primitive_is_seen_by_the_rays_that_see_its_object() {
    use crate::graph::Value;
    let mut object = [Object::new(0)];
    object[0].node.set("visibility", Value::Uint(5));
    object[0].node.set("is_shadow_catcher", Value::Boolean(true));
    let mut bvh = Bvh2::new(BvhParam::default());
    bvh.build(&object, &[square()]);
    // A shadow catcher's rays are repeated 16 bits up.
    assert_eq!(bvh.pack.prim_visibility, [5 | (5 << 16); 2]);
    assert_eq!(bvh.pack.leaf_node[0][2], 5 | (5 << 16));
}
