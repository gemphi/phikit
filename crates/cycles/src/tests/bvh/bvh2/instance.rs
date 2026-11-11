// SPDX-License-Identifier: Apache-2.0
use super::*;
use crate::tests::scene::geometry::Rig;
use crate::util::Transform;

/// A strip of three squares placed twice, five units either side of a
/// square placed once, whose transform a static scene puts into it.
fn scene() -> Rig {
    let mut rig = Rig::new(vec![strip(3), square()]);
    rig.param.bvh_type = BvhType::Static;
    rig.object.push(Object::new(0));
    assert!(rig.object[0].set_transform(Transform::translate(0.0, -5.0, 0.0)));
    assert!(rig.object[2].set_transform(Transform::translate(0.0, 5.0, 0.0)));
    rig.update();
    rig
}

#[test]
fn each_object_is_told_where_its_geometry_s_tree_starts() {
    let rig = scene();
    let node = &rig.dscene.object_node;
    // Both placings of the strip share one tree, after the top level's
    // nodes; the square has none of its own.
    assert_eq!((node.len(), node[0], node[1]), (3, node[2], 0));
    assert!(node[0] >= 4 && node[0] % 4 == 0);
    assert!(rig.mesh[0].geometry.bvh.is_some() && rig.mesh[1].geometry.bvh.is_none());
}

#[test]
fn the_top_level_holds_instances_and_the_triangles_of_applied_geometry() {
    let rig = scene();
    let pack = rig.packed();
    let bound_of = |place: usize| match pack.prim_index[place] {
        -1 => rig.object[pack.prim_object[place] as usize].bound,
        index => triangle_box(&rig.mesh[1], index as usize - rig.mesh[1].geometry.prim_offset),
    };
    let (place, around) = walk(&pack, pack.root_index, &bound_of);
    let mut found: Vec<(i32, i32)> = place.iter().map(|place| (pack.prim_index[*place], pack.prim_object[*place])).collect();
    found.sort_unstable();
    // The square's triangles come after the strip's six, among them all.
    assert_eq!(found, [(-1, 0), (-1, 2), (6, 1), (7, 1)]);
    assert_eq!((around.min, around.max), ([0.0, -5.0, 0.0], [5.0, 6.0, 0.0]));
    // An instance is seen as an object; a triangle as its object is.
    assert!(place.iter().all(|place| pack.prim_visibility[*place] == [0, 127][usize::from(pack.prim_index[*place] != -1)]));
}

#[test]
fn a_geometry_s_tree_follows_with_its_places_moved() {
    let rig = scene();
    let pack = rig.packed();
    let (place, around) = walk(&pack, pack.object_node[0], &|place| triangle_box(&rig.mesh[0], pack.prim_index[place] as usize));
    let mut triangle: Vec<i32> = place.iter().map(|place| pack.prim_index[*place]).collect();
    triangle.sort_unstable();
    assert_eq!(triangle, [0, 1, 2, 3, 4, 5]);
    // After the top level's four, in the strip's own space, seen by all.
    assert!(place.iter().all(|place| *place >= 4 && pack.prim_visibility[*place] == 127 | (127 << 16)));
    assert_eq!((around.min, around.max, pack.prim_index.len()), ([0.0; 3], [5.0, 1.0, 0.0], 10));
}

#[test]
fn a_tree_of_one_leaf_is_found_among_the_leaves() {
    let mut rig = Rig::new(vec![square()]);
    rig.update();
    let pack = rig.packed();
    // The top level has an empty leaf and the object's; the square's
    // leaf is the third, and its two triangles follow the object.
    assert_eq!((pack.object_node.clone(), pack.leaf_node.len(), pack.node.len()), (vec![-3], 3, 4));
    assert_eq!(pack.leaf_node[2], [1, 3, 127 | (127 << 16), 1]);
    assert_eq!(pack.prim_index, [-1, 0, 1]);
}

#[test]
fn time_spans_follow_too_when_the_trees_keep_them() {
    let mut rig = Rig::new(vec![moving_square()]);
    rig.param.bvh_time_step_count = 1;
    rig.update();
    let pack = rig.packed();
    assert_eq!((pack.prim_index.len(), pack.prim_time.len()), (5, 5));
    assert!(pack.prim_time[1..].iter().all(|span| *span == [0.0, 0.5] || *span == [0.5, 1.0]));
}
