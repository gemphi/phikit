// SPDX-FileCopyrightText: 2009-2010 NVIDIA Corporation
// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use super::*;

#[test]
fn a_refitted_tree_keeps_its_shape_and_holds_the_moved_vertices() {
    let mut mesh = strip(40);
    let mut bvh = tree_of(&mesh, BvhParam { use_spatial_split: false, ..BvhParam::default() });
    let before = bvh.clone();
    // Lift every second vertex.
    mesh.geometry.position_mut().iter_mut().step_by(2).for_each(|point| point[2] += 3.0);
    bvh.refit(&[Object::new(0)], std::slice::from_ref(&mesh));

    assert_eq!((&bvh.pack.prim_index, bvh.pack.node.len(), &bvh.pack.leaf_node), (&before.pack.prim_index, before.pack.node.len(), &before.pack.leaf_node));
    // Where the children are is unchanged; their boxes are not.
    assert!(bvh.pack.node.chunks_exact(4).zip(before.pack.node.chunks_exact(4)).all(|(now, was)| now[0] == was[0]));
    assert_eq!(kept_box(&bvh.pack.node[..4], 0).merge(&kept_box(&bvh.pack.node[..4], 1)).max[2], 3.0);
    // The walk checks that every box holds what is under it now.
    assert_eq!(reached(&bvh, &mesh).len(), 80);
}

#[test]
fn a_tree_of_one_leaf_is_refitted_too() {
    let mut mesh = square();
    let mut bvh = tree_of(&mesh, BvhParam::default());
    mesh.geometry.position_mut()[0] = [-4.0, 0.0, 0.0];
    let mut object = [Object::new(0)];
    object[0].node.set("visibility", crate::graph::Value::Uint(3));
    bvh.refit(&object, std::slice::from_ref(&mesh));
    // The leaf has no box of its own; it is seen as its object now is.
    assert_eq!(bvh.pack.leaf_node, [[0, 2, 3, 1]]);
    assert_eq!(bvh.pack.prim_visibility, [3, 3]);
}

#[test]
fn a_moving_mesh_is_refitted_around_every_motion_step() {
    let mut mesh = strip(4);
    mesh.geometry.node.set("motion_steps", crate::graph::Value::Uint(3));
    mesh.geometry.node.set("use_motion_blur", crate::graph::Value::Boolean(true));
    let mut bvh = tree_of(&mesh, BvhParam { max_triangle_leaf_size: 2, max_motion_triangle_leaf_size: 2, ..BvhParam::default() });
    assert!(bvh.pack.root_index == 0);
    // Now it moves: a step two units up.
    let lifted: Vec<[f32; 3]> = mesh.geometry.position().iter().map(|point| [point[0], point[1], 2.0]).collect();
    let position = mesh.geometry.attribute.find_standard_mut(crate::scene::AttributeStandard::of("POSITION")).expect("a mesh has positions");
    position.add_motion(3);
    position.data_mut(2).float3_mut().copy_from_slice(&lifted);
    bvh.refit(&[Object::new(0)], std::slice::from_ref(&mesh));
    let around = kept_box(&bvh.pack.node[..4], 0).merge(&kept_box(&bvh.pack.node[..4], 1));
    assert_eq!((around.min[2], around.max[2]), (0.0, 2.0));
}
