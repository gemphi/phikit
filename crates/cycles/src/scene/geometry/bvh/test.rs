// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::scene::geometry::test::*;
use crate::bvh::BvhParam;
use crate::bvh::test::{strip, triangle_box, walk};
use crate::util::Transform;

/// Mark a mesh's tree, to tell one that was kept from one made anew.
fn mark(rig: &mut Rig) {
    rig.mesh[0].geometry.bvh.as_mut().expect("the mesh has a tree").param.sah_node_cost = 9.0;
}

fn marked(rig: &Rig) -> bool {
    rig.mesh[0].geometry.bvh.as_ref().is_some_and(|bvh| bvh.param.sah_node_cost == 9.0)
}

/// Say the first mesh changed; with `rebuild`, in its triangles.
fn tag(rig: &mut Rig, rebuild: bool) {
    let flag = rig.mesh[0].geometry.tag_update(rebuild);
    rig.geometry.tag_update(flag);
}

#[test]
fn an_instanced_geometry_has_its_own_tree_and_so_has_any_under_a_gpu_layout() {
    let mut geometry = Geometry::new(GeometryKind::Mesh).expect("data/node.tsv has the mesh");
    assert!(geometry.need_build_bvh(BvhLayout::of("BVH2")));
    geometry.transform_applied = true;
    assert!(!geometry.need_build_bvh(BvhLayout::of("BVH2")) && !geometry.need_build_bvh(BvhLayout::of("EMBREE")));
    assert!(geometry.need_build_bvh(BvhLayout::of("OPTIX")) && geometry.need_build_bvh(BvhLayout::of("MULTI_HIPRT")));
}

#[test]
fn a_tag_says_whether_the_tree_must_be_built_again() {
    let mut geometry = Geometry::new(GeometryKind::Mesh).expect("data/node.tsv has the mesh");
    assert_eq!((geometry.tag_update(false), geometry.need_update_rebuild), (GeometryManager::flag("GEOMETRY_MODIFIED"), false));
    assert_eq!((geometry.tag_update(true), geometry.need_update_rebuild), (GeometryManager::flag("GEOMETRY_MODIFIED"), true));
    // It stays until a tree is built.
    assert!(geometry.tag_update(false) != 0 && geometry.need_update_rebuild);
}

#[test]
fn an_update_builds_each_mesh_s_tree_and_gives_the_scene_s_to_the_device() {
    let mut rig = Rig::new(vec![strip(3), square()]);
    rig.update();
    assert!(rig.mesh.iter().all(|mesh| mesh.geometry.bvh.is_some() && !mesh.geometry.need_update_rebuild && !mesh.geometry.need_update_bvh_for_offset));
    let data = &rig.dscene.data;
    assert_eq!((data.int("bvh", "root"), data.int("bvh", "bvh_layout"), data.int("bvh", "use_bvh_steps"), data.int("bvh", "curve_subdivisions")), (0, BvhLayout::of("BVH2").0, 0, 8));
    assert!(!rig.dscene.bvh_node.is_empty() && rig.dscene.object_node.len() == 2 && rig.dscene.prim_time.is_empty());
    assert_eq!(rig.dscene.prim_index.len(), rig.dscene.prim_visibility.len());
    // The scene's tree is kept, its arrays handed over.
    assert!(rig.bvh.as_ref().is_some_and(|bvh| bvh.param.top_level && bvh.pack.node.is_empty()));
}

#[test]
fn a_dynamic_tree_is_refitted_when_only_vertices_moved() {
    let mut rig = Rig::new(vec![strip(40)]);
    rig.update();
    mark(&mut rig);
    rig.mesh[0].geometry.position_mut()[0] = [-50.0, 0.0, 0.0];
    tag(&mut rig, false);
    rig.update();
    assert!(marked(&rig));
    // The walk checks every box; the scene's tree holds the new corner.
    let bvh = rig.mesh[0].geometry.bvh.as_ref().expect("kept");
    let (place, around) = walk(&bvh.pack, bvh.pack.root_index, &|place| triangle_box(&rig.mesh[0], bvh.pack.prim_index[place] as usize));
    assert_eq!((place.len(), around.min[0], rig.object[0].bound.min[0]), (80, -50.0, -50.0));
    let pack = rig.packed();
    assert_eq!(walk(&pack, pack.root_index, &|place| rig.object[pack.prim_object[place] as usize].bound).1.min[0], -50.0);
}

#[test]
fn it_is_built_again_when_its_triangles_changed_or_the_scene_is_static() {
    let mut rig = Rig::new(vec![strip(40)]);
    rig.update();
    mark(&mut rig);
    tag(&mut rig, true);
    rig.mesh[0].geometry.node.tag_modified();
    rig.update();
    assert!(!marked(&rig) && !rig.mesh[0].geometry.need_update_rebuild);

    // Placed twice, so a static scene leaves the mesh a tree of its own.
    let mut rig = Rig::new(vec![strip(40)]);
    rig.object.push(Object::new(0));
    rig.param.bvh_type = BvhType::Static;
    rig.update();
    mark(&mut rig);
    rig.mesh[0].geometry.position_mut()[0] = [-50.0, 0.0, 0.0];
    tag(&mut rig, false);
    rig.update();
    assert!(!marked(&rig));
}

#[test]
fn a_mesh_s_tree_is_built_with_the_scene_s_settings() {
    let mut rig = Rig::new(vec![strip(3)]);
    rig.param.use_bvh_spatial_split = true;
    rig.param.bvh_time_step_count = 2;
    rig.update();
    let param = rig.mesh[0].geometry.bvh.as_ref().expect("built").param;
    let expected = BvhParam { use_spatial_split: true, use_compact_structure: true, motion_triangle_step_count: 2, motion_curve_step_count: 2, motion_point_step_count: 2, curve_subdivision: 8, ..BvhParam::default() };
    assert_eq!(param, expected);
    assert_eq!(rig.bvh.as_ref().expect("built").param, BvhParam { top_level: true, use_compact_structure: false, ..expected });
    assert_eq!(rig.dscene.data.int("bvh", "use_bvh_steps"), 1);
}

#[test]
fn the_scene_s_tree_is_kept_until_something_in_it_changes() {
    let mut rig = Rig::new(vec![square(), square()]);
    rig.update();
    let built = rig.dscene.bvh_node.clone();
    rig.dscene.bvh_node[0] = [7; 4];
    // A shader's attributes changed: the arrays are written, the tree stays.
    rig.geometry.tag_update(GeometryManager::flag("SHADER_ATTRIBUTE_MODIFIED"));
    rig.update();
    assert_eq!((rig.dscene.bvh_node[0], rig.dscene.data.int("bvh", "bvh_layout")), ([7; 4], BvhLayout::of("BVH2").0));
    // An object moved: its box in the tree must move.
    assert!(rig.object[1].set_transform(Transform::translate(3.0, 0.0, 0.0)));
    let flag = rig.object_manager.tag_update(ObjectManager::flag("TRANSFORM_MODIFIED"));
    rig.geometry.tag_update(flag);
    rig.update();
    assert!(rig.dscene.bvh_node[0] != [7; 4] && rig.dscene.bvh_node != built);
    let pack = rig.packed();
    assert_eq!(walk(&pack, pack.root_index, &|place| rig.object[pack.prim_object[place] as usize].bound).1.max[0], 4.0);
}

#[test]
fn freeing_keeps_the_scene_s_tree_unless_forced() {
    let mut rig = Rig::new(vec![square()]);
    rig.update();
    GeometryManager::device_free(&mut rig.dscene, false);
    assert!(!rig.dscene.bvh_node.is_empty() && !rig.dscene.prim_index.is_empty() && rig.dscene.tri_vert.is_empty());
    // No tree to walk meanwhile.
    assert_eq!(rig.dscene.data.int("bvh", "bvh_layout"), BvhLayout::NONE.0);
    GeometryManager::device_free(&mut rig.dscene, true);
    assert!(rig.dscene.bvh_node.is_empty() && rig.dscene.bvh_leaf_node.is_empty() && rig.dscene.object_node.is_empty());
    assert!(rig.dscene.prim_index.is_empty() && rig.dscene.prim_type.is_empty() && rig.dscene.prim_visibility.is_empty() && rig.dscene.prim_object.is_empty());
}

#[test]
fn a_layout_that_is_not_ported_leaves_the_device_no_tree() {
    let mut rig = Rig::new(vec![square()]);
    rig.bvh_layout_mask = BvhLayout::of("OPTIX").0;
    rig.update();
    assert!(rig.bvh.is_none() && rig.mesh[0].geometry.bvh.is_none());
    assert!(rig.dscene.bvh_node.is_empty() && rig.dscene.bvh_leaf_node.is_empty() && rig.dscene.prim_index.is_empty());
    assert_eq!((rig.dscene.data.int("bvh", "root"), rig.dscene.data.int("bvh", "bvh_layout")), (-1, BvhLayout::of("OPTIX").0));
    // The triangles are written all the same.
    assert_eq!(rig.dscene.tri_vindex.len(), 2);
}
