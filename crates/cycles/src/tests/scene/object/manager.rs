// SPDX-License-Identifier: Apache-2.0
use super::*;

#[test]
fn a_new_manager_has_everything_to_update() {
    let mut rig = Rig::new(vec![square()]);
    assert!(rig.object_manager.need_update() && rig.object_manager.need_flag_update);
    rig.update();
    assert!(!rig.object_manager.need_update() && !rig.object_manager.need_flag_update);
}

#[test]
fn an_update_numbers_the_objects_and_takes_them_as_unchanged() {
    let mut rig = Rig::new(vec![square(), square()]);
    rig.object.push(Object::new(0));
    rig.update_object();
    assert_eq!(rig.object.iter().map(|object| object.index).collect::<Vec<_>>(), [0, 1, 2]);
    assert!(rig.object.iter().all(|object| !object.node.is_modified()));
    assert_eq!((rig.dscene.object.len(), rig.dscene.object_flag.len()), (3, 3));
}

#[test]
fn nothing_is_written_when_nothing_is_due() {
    let mut rig = Rig::new(vec![square()]);
    rig.update();
    rig.dscene.object.set_float(0, "alpha", 0.25);
    rig.update_object();
    assert_eq!(rig.dscene.object.float(0, "alpha"), 0.25);
    rig.object_manager.tag_update(ObjectManager::flag("OBJECT_MODIFIED"));
    rig.update_object();
    assert_eq!(rig.dscene.object.float(0, "alpha"), 0.0);
}

#[test]
fn a_scene_with_no_objects_has_empty_arrays() {
    let mut rig = Rig::new(Vec::new());
    rig.update();
    assert!(rig.dscene.object.is_empty() && rig.dscene.object_flag.is_empty());
}

#[test]
fn a_tag_is_passed_on_to_the_geometry_unless_it_came_from_it() {
    let mut manager = ObjectManager::new();
    let geometry = |name: &str| GeometryManager::flag(name);
    assert_eq!(manager.tag_update(ObjectManager::flag("OBJECT_MODIFIED")), geometry("OBJECT_MANAGER"));
    assert_eq!(manager.tag_update(ObjectManager::flag("TRANSFORM_MODIFIED")), geometry("OBJECT_MANAGER") | geometry("TRANSFORM_MODIFIED"));
    assert_eq!(manager.tag_update(ObjectManager::flag("VISIBILITY_MODIFIED")), geometry("OBJECT_MANAGER") | geometry("VISIBILITY_MODIFIED"));
    // An object added may be an instance: the BVH must be built again.
    assert_eq!(manager.tag_update(ObjectManager::flag("OBJECT_ADDED")), geometry("OBJECT_MANAGER") | geometry("GEOMETRY_ADDED") | geometry("GEOMETRY_REMOVED"));
    assert_eq!(manager.tag_update(ObjectManager::flag("GEOMETRY_MANAGER") | ObjectManager::flag("TRANSFORM_MODIFIED")), 0);
}

#[test]
fn in_a_viewport_an_object_remembers_the_transform_it_had() {
    let mut rig = Rig::new(vec![square(), square()]);
    rig.update();
    rig.object[0].set_transform(moved(1.0));
    // The first frame only remembers: one step is not motion.
    assert!(!rig.object_manager.update_interactive_motion(&mut rig.object));
    assert_eq!((rig.object[0].motion(), rig.object[1].motion()), (&[moved(1.0)][..], &[Transform::IDENTITY][..]));
    assert!(!rig.object_manager.need_update());
}

#[test]
fn in_a_viewport_an_object_in_motion_is_brought_up_to_date() {
    let mut rig = Rig::new(vec![square()]);
    rig.object[0].node.set("motion", Value::TransformArray(vec![moved(0.0), moved(2.0)]));
    rig.update();
    rig.object[0].set_transform(moved(1.0));
    assert!(rig.object_manager.update_interactive_motion(&mut rig.object));
    assert_eq!(rig.object[0].motion(), [moved(1.0), moved(2.0)]);
    assert!(rig.object_manager.need_update());
}

#[test]
fn freeing_empties_the_objects_arrays() {
    let mut rig = Rig::new(vec![square()]);
    rig.need_motion = MotionType::Pass;
    rig.update();
    assert!(!rig.dscene.object.is_empty() && !rig.dscene.object_motion_pass.is_empty());
    ObjectManager::device_free(&mut rig.dscene);
    assert!(rig.dscene.object.is_empty() && rig.dscene.object_motion_pass.is_empty() && rig.dscene.object_flag.is_empty());
}
