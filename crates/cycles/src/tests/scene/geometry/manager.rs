// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use super::*;
use crate::util::Transform;

#[test]
fn a_new_manager_has_everything_to_update_and_an_update_leaves_nothing() {
    let mut rig = Rig::new(vec![square()]);
    assert!(rig.geometry.need_update() && rig.geometry.need_flag_update);
    rig.update();
    assert!(!rig.geometry.need_update() && !rig.geometry.need_flag_update);
    assert!(!rig.mesh[0].geometry.node.is_modified() && !rig.mesh[0].geometry.position_is_modified());
}

#[test]
fn a_tag_is_passed_on_to_the_objects_unless_it_came_from_them() {
    let mut rig = Rig::new(vec![square()]);
    rig.update();
    assert_eq!(rig.geometry.tag_update(GeometryManager::flag("GEOMETRY_MODIFIED")), ObjectManager::flag("GEOMETRY_MANAGER"));
    assert!(rig.geometry.need_update());
    assert_eq!(rig.geometry.tag_update(GeometryManager::flag("OBJECT_MANAGER")), 0);
}

#[test]
fn each_mesh_starts_where_the_one_before_ends() {
    let mut mesh = vec![square(), square(), Mesh::new(), square()];
    mesh[1].resize_subd_face(2, 8);
    GeometryManager::geom_calc_offset(&mut mesh);
    assert_eq!(mesh.iter().map(|mesh| mesh.geometry.prim_offset).collect::<Vec<_>>(), [0, 2, 4, 4]);
    assert_eq!((mesh[2].face_offset, mesh[2].corner_offset), (2, 8));
    // A mesh that moved in the arrays must have its BVH looked at again.
    assert_eq!(mesh.iter().map(|mesh| mesh.geometry.need_update_bvh_for_offset).collect::<Vec<_>>(), [false, true, true, true]);
}

#[test]
fn an_update_gives_a_mesh_normals_and_a_box() {
    let mut rig = Rig::new(vec![square()]);
    rig.update();
    let mesh = &rig.mesh[0];
    assert!(mesh.geometry.attribute.find_standard(std("VERTEX_NORMAL")).is_some());
    assert_eq!((mesh.geometry.bound.min, mesh.geometry.bound.max), ([0.0; 3], [1.0, 1.0, 0.0]));
    // Nobody asked for generated coordinates.
    assert!(mesh.geometry.attribute.find_standard(std("GENERATED")).is_none());
}

#[test]
fn generated_coordinates_are_made_for_a_shader_that_asks() {
    let mut rig = Rig::new(vec![square()]);
    rig.shader[0].attribute.add_standard(std("GENERATED"));
    rig.mesh[0].geometry.set_used_shader(vec![0]);
    rig.update();
    assert!(rig.mesh[0].geometry.attribute.find_standard(std("GENERATED")).is_some());
    assert_eq!(rig.dscene.attribute_float3.len(), 4);
}

#[test]
fn an_object_s_box_is_its_mesh_s_in_the_world() {
    let mut rig = Rig::new(vec![square()]);
    rig.object[0].set_transform(Transform::translate(10.0, 0.0, 0.0));
    rig.update();
    assert_eq!((rig.object[0].bound.min, rig.object[0].bound.max), ([10.0, 0.0, 0.0], [11.0, 1.0, 0.0]));
    assert!(!rig.mesh[0].geometry.transform_applied);
}

#[test]
fn a_static_scene_puts_an_only_object_s_transform_into_its_mesh() {
    let mut rig = Rig::new(vec![square(), square()]);
    rig.param.bvh_type = BvhType::Static;
    // The second mesh is placed twice, so it stays an instance.
    rig.object.push(Object::new(1));
    for object in rig.object.iter_mut() {
        object.set_transform(Transform::translate(10.0, 0.0, 0.0));
    }
    rig.update();
    assert!(rig.mesh[0].geometry.transform_applied && !rig.mesh[1].geometry.transform_applied);
    assert_eq!((rig.mesh[0].geometry.position()[0], rig.mesh[1].geometry.position()[0]), ([10.0, 0.0, 0.0], [0.0; 3]));
    assert_eq!(rig.dscene.tri_vert[0], [10.0, 0.0, 0.0]);
    // The box is the same either way.
    assert_eq!((rig.object[0].bound, rig.object[1].bound), (rig.object[2].bound, rig.object[2].bound));
    let applied = crate::kernel::flag("SD_OBJECT_TRANSFORM_APPLIED");
    assert_eq!(rig.dscene.object_flag.iter().map(|mask| mask & applied != 0).collect::<Vec<_>>(), [true, false, false]);
}

#[test]
fn a_mesh_has_of_its_shaders_what_they_have() {
    let mut rig = Rig::new(vec![square(), square()]);
    rig.shader.push(Shader { has_volume: true, has_surface_bssrdf: true, ..Shader::default() });
    rig.mesh[1].geometry.set_used_shader(vec![0, 1]);
    rig.geometry.device_update_preprocess(&mut rig.mesh, &rig.shader);
    assert_eq!(rig.mesh.iter().map(|mesh| (mesh.geometry.has_volume, mesh.geometry.has_surface_bssrdf)).collect::<Vec<_>>(), [(false, false), (true, true)]);
    // The volume is looked for again each time; subsurface stays.
    rig.mesh[1].geometry.set_used_shader(vec![0]);
    rig.geometry.need_flag_update = true;
    rig.geometry.device_update_preprocess(&mut rig.mesh, &rig.shader);
    assert_eq!((rig.mesh[1].geometry.has_volume, rig.mesh[1].geometry.has_surface_bssrdf), (false, true));
}

#[test]
fn a_shader_whose_displacement_changed_tags_its_mesh_s_positions() {
    let mut rig = Rig::new(vec![square()]);
    rig.mesh[0].geometry.set_used_shader(vec![0]);
    rig.update();
    rig.shader[0].need_update_displacement = true;
    rig.geometry.need_flag_update = true;
    rig.geometry.device_update_preprocess(&mut rig.mesh, &rig.shader);
    assert!(rig.mesh[0].geometry.position_is_modified());
}

#[test]
fn in_a_viewport_a_mesh_that_moved_remembers_where_it_was() {
    let mut rig = Rig::new(vec![moving_square(), square()]);
    rig.update();
    assert!(rig.geometry.update_interactive_motion(&mut rig.mesh));
    assert!(rig.geometry.need_update());
    let position = rig.mesh[0].geometry.position_attribute().expect("a mesh has positions");
    assert_eq!((position.data(1), position.modified), (position.data(0), true));
    // Now the step is where the mesh is: nothing more to do.
    assert!(!rig.geometry.update_interactive_motion(&mut rig.mesh));
}

#[test]
fn freeing_empties_the_geometry_s_arrays() {
    let mut rig = Rig::new(vec![square()]);
    rig.update();
    assert!(!rig.dscene.tri_vert.is_empty() && !rig.dscene.attribute_map.is_empty());
    GeometryManager::device_free(&mut rig.dscene, true);
    assert!(rig.dscene.tri_vert.is_empty() && rig.dscene.tri_shader.is_empty() && rig.dscene.attribute_map.is_empty() && rig.dscene.attribute_normal.is_empty());
}
