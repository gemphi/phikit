// SPDX-License-Identifier: Apache-2.0
use super::*;
use crate::kernel::{constant, flag};
use crate::util::{hash_to_float, motion_decompose, murmur_hash3};

/// One square placed by an object that `setup` changes, written to a device.
fn written(setup: impl FnOnce(&mut Object), need_motion: MotionType) -> Rig {
    let mut rig = Rig::new(vec![square()]);
    rig.need_motion = need_motion;
    setup(&mut rig.object[0]);
    rig.update_object();
    rig
}

fn still(setup: impl FnOnce(&mut Object)) -> Rig {
    written(setup, MotionType::None)
}

#[test]
fn an_object_s_transform_and_its_inverse_reach_the_kernel() {
    let place = Transform::translate(1.0, 2.0, 3.0) * Transform::scale(2.0, 2.0, 2.0);
    let rig = still(|object| {
        object.set_transform(place);
    });
    let object = &rig.dscene.object;
    assert_eq!((object.float_list(0, "tfm"), object.float_list(0, "itfm")), (place.word().to_vec(), place.inverse().word().to_vec()));
    assert_eq!((object.float(0, "volume_density"), object.int(0, "numverts"), object.int(0, "numprims"), object.int(0, "primitive_type")), (1.0, 4, 2, 1));
}

#[test]
fn an_object_s_settings_reach_the_kernel() {
    let rig = still(|object| {
        object.node.set("color", Value::Float3([0.25, 0.5, 0.75]));
        object.node.set("alpha", Value::Float(0.5));
        object.node.set("pass_id", Value::Int(7));
        object.node.set("random_id", Value::Uint(u32::MAX));
        object.node.set("ao_distance", Value::Float(2.0));
        object.node.set("dupli_uv", Value::Float2([0.1, 0.2]));
        object.node.set("shadow_terminator_shading_offset", Value::Float(1.0));
    });
    let object = &rig.dscene.object;
    assert_eq!((object.float_list(0, "color"), object.float(0, "alpha"), object.float(0, "pass_id")), (vec![0.25, 0.5, 0.75], 0.5, 7.0));
    assert_eq!((object.float(0, "random_number"), object.float(0, "ao_distance"), object.float_list(0, "dupli_uv")), (1.0, 2.0, vec![0.1, 0.2]));
    // The shading offset is stored ready to multiply by; the geometry offset as it is.
    assert_eq!((object.float(0, "shadow_terminator_shading_offset"), object.float(0, "shadow_terminator_geometry_offset")), (2.0, 0.1));
    assert_eq!((object.uint(0, "visibility"), object.int(0, "lightgroup")), (127, constant("LIGHTGROUP_NONE").expect("in data/constant.tsv") as i32));
    let not_found = constant("ATTR_STD_NOT_FOUND").expect("in data/constant.tsv") as i32;
    assert_eq!((object.int(0, "position_offset"), object.int(0, "normal_offset")), (not_found, not_found));
}

#[test]
fn an_object_is_named_for_cryptomatte_by_the_hash_of_its_name() {
    let rig = still(|object| {
        object.node.name = "Cube".to_string();
        object.node.set("asset_name", Value::String("Set".to_string()));
    });
    let object = &rig.dscene.object;
    assert_eq!(object.float(0, "cryptomatte_object"), hash_to_float(murmur_hash3(b"Cube", 0)));
    assert_eq!(object.float(0, "cryptomatte_asset"), hash_to_float(murmur_hash3(b"Set", 0)));
}

#[test]
fn a_light_set_past_the_last_is_no_set() {
    let set = |number: u32| still(|object| done(object.node.set("receiver_light_set", Value::Uint(number)))).dscene.object.uint(0, "receiver_light_set");
    assert_eq!([0, 5, 63, 64].map(set), [0, 5, 63, 0]);
    let rig = still(|object| done(object.node.set("shadow_set_membership", Value::Uint64(0b101))));
    assert_eq!((rig.dscene.object.uint64(0, "shadow_set_membership"), rig.dscene.object.uint64(0, "light_set_membership")), (0b101, u64::MAX));
}

#[test]
fn a_particle_is_numbered_after_the_systems_before_its_own() {
    // The rig's systems have 3 and 5 particles; particle 0 is a dummy.
    let rig = still(|object| {
        object.particle_system = Some(1);
        object.node.set("particle_index", Value::Int(2));
    });
    assert_eq!(rig.dscene.object.int(0, "particle_index"), 1 + 3 + 2);
    assert_eq!(still(|_| {}).dscene.object.int(0, "particle_index"), 0);
}

#[test]
fn an_object_is_in_the_light_group_of_its_name() {
    let mut rig = Rig::new(vec![square()]);
    rig.lightgroup = vec!["key".to_string(), "fill".to_string()];
    rig.object[0].node.set("lightgroup", Value::String("fill".to_string()));
    rig.update_object();
    assert_eq!(rig.dscene.object.int(0, "lightgroup"), 1);
}

#[test]
fn the_flags_say_what_the_object_is() {
    let mask = |setup: fn(&mut Object)| still(setup).dscene.object_flag[0];
    assert_eq!(mask(|_| {}), 0);
    assert_eq!(mask(|object| done(object.set_transform(Transform::scale(1.0, -1.0, 1.0)))), flag("SD_OBJECT_NEGATIVE_SCALE"));
    assert_eq!(mask(|object| done(object.node.set("use_holdout", Value::Boolean(true)))), flag("SD_OBJECT_HOLDOUT_MASK"));
    assert_eq!(mask(|object| done(object.node.set("is_caustics_caster", Value::Boolean(true)))), flag("SD_OBJECT_CAUSTICS_CASTER"));
    assert_eq!(mask(|object| done(object.node.set("is_caustics_receiver", Value::Boolean(true)))), flag("SD_OBJECT_CAUSTICS_RECEIVER"));
}

#[test]
fn a_mesh_whose_vertices_move_or_that_has_corner_normals_says_so() {
    let mut rig = Rig::new(vec![moving_square(), square()]);
    rig.mesh[1].add_attribute(std("CORNER_NORMAL"));
    rig.update_object();
    assert_eq!(rig.dscene.object_flag, [flag("SD_OBJECT_HAS_VERTEX_MOTION"), flag("SD_OBJECT_HAS_CORNER_NORMALS")]);
    assert_eq!((rig.dscene.object.int(0, "primitive_type"), rig.dscene.object.uint(0, "num_geom_steps")), (65, 3));
    // A blurred mesh is motion for the BVH, whether or not an object moves.
    assert_eq!(rig.dscene.data.int("bvh", "have_motion"), 1);
}

#[test]
fn motion_vectors_of_a_still_object_are_the_identity() {
    let rig = written(|object| done(object.set_transform(moved(3.0))), MotionType::Pass);
    assert_eq!(rig.dscene.object_motion_pass, [Transform::IDENTITY; 2]);
    assert_eq!(rig.dscene.data.int("bvh", "have_motion"), 0);
}

#[test]
fn motion_vectors_of_a_moving_object_are_from_where_it_is_to_where_it_was_and_will_be() {
    let rig = written(|object| done(object.node.set("motion", Value::TransformArray(vec![moved(-1.0), moved(0.0), moved(2.0)]))), MotionType::Pass);
    assert_eq!(rig.dscene.object_motion_pass, [moved(-1.0), moved(2.0)]);
    assert_eq!(rig.dscene.object.uint(0, "num_tfm_steps"), 3);
}

#[test]
fn a_mesh_with_moving_vertices_keeps_its_motion_transforms_in_the_world() {
    let mut rig = Rig::new(vec![moving_square()]);
    rig.need_motion = MotionType::Pass;
    rig.object[0].set_transform(moved(3.0));
    rig.update_object();
    assert_eq!(rig.dscene.object_motion_pass, [moved(3.0); 2]);
}

#[test]
fn motion_blur_decomposes_each_moving_object_s_steps_one_object_after_another() {
    let mut rig = Rig::new(vec![square(), square(), square()]);
    rig.need_motion = MotionType::Blur;
    let (first, last) = (vec![moved(0.0), moved(1.0)], vec![moved(0.0), moved(2.0), moved(4.0)]);
    rig.object[0].node.set("motion", Value::TransformArray(first.clone()));
    // The second object's steps are all where it is: no motion.
    rig.object[1].node.set("motion", Value::TransformArray(vec![Transform::IDENTITY; 3]));
    rig.object[2].node.set("motion", Value::TransformArray(last.clone()));
    rig.update_object();
    let dscene = &rig.dscene;
    assert_eq!([0, 1, 2].map(|index| dscene.object.uint(index, "motion_offset")), [0, 0, 2]);
    assert_eq!([0, 1, 2].map(|index| dscene.object.uint(index, "num_tfm_steps")), [2, 0, 3]);
    assert_eq!(dscene.object_motion, [motion_decompose(&first), motion_decompose(&last)].concat());
    let motion = flag("SD_OBJECT_MOTION");
    assert_eq!(dscene.object_flag, [motion, 0, motion]);
    assert_eq!(dscene.data.int("bvh", "have_motion"), 1);
    assert!(dscene.object_motion_pass.is_empty());
}
