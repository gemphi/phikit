// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use super::*;
use crate::bvh::BvhType;
use crate::kernel::flag;

fn has(rig: &Rig, object: usize, name: &str) -> bool {
    rig.dscene.object_flag[object] & flag(name) != 0
}

/// Three squares side by side, two units apart; the middle one has a
/// shader with a volume.
fn beside_a_volume() -> Rig {
    let mut rig = Rig::new(vec![square(), square(), square()]);
    rig.shader.push(Shader { has_volume: true, ..Shader::default() });
    rig.mesh[1].geometry.set_used_shader(vec![1]);
    for (object, x) in rig.object.iter_mut().zip([0.0, 0.5, 5.0]) {
        object.set_transform(moved(x));
    }
    rig
}

#[test]
fn an_object_that_touches_a_volume_s_box_says_so() {
    let mut rig = beside_a_volume();
    rig.update();
    assert_eq!([0, 1, 2].map(|object| has(&rig, object, "SD_OBJECT_HAS_VOLUME")), [false, true, false]);
    // The volume does not count as touching itself.
    assert_eq!([0, 1, 2].map(|object| has(&rig, object, "SD_OBJECT_INTERSECTS_VOLUME")), [true, false, false]);
    assert_eq!(rig.object.iter().map(|object| object.intersects_volume).collect::<Vec<_>>(), [true, false, false]);
    assert!(!has(&rig, 1, "SD_OBJECT_HAS_VOLUME_ATTRIBUTES"));
}

#[test]
fn before_the_boxes_are_final_every_object_may_touch_a_volume() {
    let mut rig = beside_a_volume();
    rig.geometry.device_update_preprocess(&mut rig.mesh, &rig.shader);
    rig.update_object();
    rig.object_manager.device_update_flag(&mut rig.dscene, &mut rig.object, &rig.mesh, false);
    assert_eq!([0, 1, 2].map(|object| has(&rig, object, "SD_OBJECT_INTERSECTS_VOLUME")), [true; 3]);
    // The flags stay due, to be worked out again with the final boxes.
    assert!(rig.object_manager.need_update() && rig.object_manager.need_flag_update);
}

#[test]
fn a_shadow_catcher_and_corner_normals_are_flagged() {
    let mut rig = Rig::new(vec![square(), square()]);
    rig.object[0].node.set("is_shadow_catcher", Value::Boolean(true));
    rig.mesh[1].add_attribute(std("CORNER_NORMAL"));
    rig.update();
    assert_eq!([0, 1].map(|object| has(&rig, object, "SD_OBJECT_SHADOW_CATCHER")), [true, false]);
    assert_eq!([0, 1].map(|object| has(&rig, object, "SD_OBJECT_HAS_CORNER_NORMALS")), [false, true]);
}

#[test]
fn a_flag_that_no_longer_holds_is_cleared() {
    let mut rig = Rig::new(vec![square()]);
    rig.mesh[0].add_attribute(std("CORNER_NORMAL"));
    rig.update();
    assert!(has(&rig, 0, "SD_OBJECT_HAS_CORNER_NORMALS"));
    rig.mesh[0].geometry.attribute.remove_standard(std("CORNER_NORMAL"));
    rig.object_manager.need_flag_update = true;
    rig.object_manager.device_update_flag(&mut rig.dscene, &mut rig.object, &rig.mesh, true);
    assert!(!has(&rig, 0, "SD_OBJECT_HAS_CORNER_NORMALS"));
}

#[test]
fn a_transform_is_not_applied_to_a_mesh_that_must_stay_an_instance() {
    let applied = |setup: fn(&mut Rig)| {
        let mut rig = Rig::new(vec![square()]);
        rig.param.bvh_type = BvhType::Static;
        rig.object[0].set_transform(moved(1.0));
        setup(&mut rig);
        rig.update();
        rig.mesh[0].geometry.transform_applied
    };
    assert!(applied(|_| {}));
    // Subsurface scattering is traced within the one mesh.
    assert!(!applied(|rig| rig.mesh[0].geometry.has_surface_bssrdf = true));
    assert!(!applied(|rig| {
        rig.shader[0].has_true_displacement = true;
        rig.mesh[0].geometry.set_used_shader(vec![0]);
    }));
    assert!(!applied(|rig| done(rig.mesh[0].geometry.node.set_enum("subdivision_type", "linear"))));
    // An object that moves keeps its transform when the scene is blurred.
    assert!(!applied(|rig| {
        rig.need_motion = MotionType::Blur;
        rig.object[0].node.set("motion", Value::TransformArray(vec![moved(0.0), moved(1.0), moved(2.0)]));
    }));
}

#[test]
fn motion_steps_are_moved_with_the_mesh_except_for_motion_vectors() {
    let height = |need_motion: MotionType| {
        let mut rig = Rig::new(vec![moving_square()]);
        (rig.param.bvh_type, rig.need_motion) = (BvhType::Static, need_motion);
        rig.object[0].set_transform(Transform::translate(0.0, 0.0, 10.0));
        rig.update();
        let position = rig.mesh[0].geometry.position_attribute().expect("a mesh has positions");
        [0, 1, 2].map(|step| position.data(step).float3()[0][2])
    };
    assert_eq!(height(MotionType::None), [10.0, 9.0, 11.0]);
    assert_eq!(height(MotionType::Pass), [10.0, -1.0, 1.0]);
}

#[test]
fn each_object_s_first_primitive_is_its_mesh_s() {
    let mut rig = Rig::new(vec![square(), square()]);
    rig.object.push(Object::new(0));
    rig.update();
    ObjectManager::device_update_prim_offset(&mut rig.dscene, &rig.object, &rig.mesh);
    assert_eq!(rig.dscene.object_prim_offset, [0, 2, 0]);
}
