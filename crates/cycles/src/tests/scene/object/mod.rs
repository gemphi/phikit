// SPDX-License-Identifier: Apache-2.0
//! Tests of `src/scene/object`, one file for each file there. This file
//! tests `mod.rs`. The small scene they update a device with is the one of
//! the geometry's tests.

mod flag;
mod manager;
mod manifest;
mod offset;
mod transform;

use crate::graph::Value;
use crate::scene::*;
pub(crate) use crate::tests::scene::geometry::{Rig, moving_square, square, std};
use crate::util::{BoundBox, Transform};

/// A setter's answer, whether the value changed, is of no interest here.
pub(crate) fn done(_changed: bool) {}

fn moved(x: f32) -> Transform {
    Transform::translate(x, 0.0, 0.0)
}

/// An object whose motion steps are these.
fn in_motion(step: &[Transform]) -> Object {
    let mut object = Object::new(0);
    object.node.set("motion", Value::TransformArray(step.to_vec()));
    object
}

/// The square's geometry with its box worked out.
fn boxed() -> Mesh {
    let mut mesh = square();
    mesh.compute_bound();
    mesh
}

#[test]
fn a_new_object_is_where_its_geometry_is_and_seen_by_every_ray() {
    let object = Object::new(3);
    assert_eq!((object.geometry, object.transform(), object.node.uint("visibility")), (3, Transform::IDENTITY, 127));
    assert!(!object.bound.valid() && !object.use_motion() && object.particle_system.is_none());
    assert!(!object.has_light_linking() && !object.has_shadow_linking());
}

#[test]
fn motion_steps_run_from_the_shutter_s_open_to_its_close() {
    let object = in_motion(&[moved(0.0), moved(1.0), moved(2.0)]);
    assert!(object.use_motion());
    assert_eq!([0, 1, 2].map(|step| object.motion_time(step)), [-1.0, 0.0, 1.0]);
    assert_eq!((object.motion_step(1.0), object.motion_step(0.5)), (Some(2), None));
    // One step alone is not motion.
    assert!(!in_motion(&[moved(1.0)]).use_motion());
}

#[test]
fn an_object_that_does_not_in_fact_move_has_no_motion() {
    let mut object = in_motion(&[Transform::IDENTITY; 3]);
    object.update_motion();
    assert!(object.motion().is_empty());
    let mut object = in_motion(&[moved(-1.0), Transform::IDENTITY, moved(1.0)]);
    object.update_motion();
    assert_eq!(object.motion().len(), 3);
}

#[test]
fn a_missing_motion_step_takes_the_object_s_own_transform() {
    let mut object = in_motion(&[Transform::EMPTY, Transform::IDENTITY, moved(1.0)]);
    object.update_motion();
    assert_eq!(object.motion(), [Transform::IDENTITY, Transform::IDENTITY, moved(1.0)]);
    // A last step that was never set is taken off once the object moves.
    let mut object = in_motion(&[moved(-1.0), Transform::IDENTITY, Transform::EMPTY]);
    object.update_motion();
    assert_eq!(object.motion(), [moved(-1.0), Transform::IDENTITY]);
}

#[test]
fn an_object_that_hides_on_missing_motion_is_given_the_empty_transform() {
    let mut object = in_motion(&[moved(-1.0), Transform::IDENTITY, Transform::EMPTY]);
    object.node.set("hide_on_missing_motion", Value::Boolean(true));
    object.update_motion();
    assert_eq!((object.transform(), object.motion().len()), (Transform::EMPTY, 0));
}

#[test]
fn one_motion_step_can_be_set() {
    let mut object = in_motion(&[Transform::IDENTITY; 2]);
    assert!(object.set_motion_transform(moved(4.0), 1));
    assert_eq!(object.motion()[1], moved(4.0));
    assert!(!object.set_motion_transform(moved(4.0), 5));
}

#[test]
fn the_box_is_the_geometry_s_in_the_world() {
    let mesh = boxed();
    let mut object = Object::new(0);
    object.set_transform(Transform::translate(0.0, 0.0, 5.0) * Transform::scale(2.0, 2.0, 2.0));
    object.compute_bound(&mesh.geometry, false);
    assert_eq!((object.bound.min, object.bound.max), ([0.0, 0.0, 5.0], [2.0, 2.0, 5.0]));
}

#[test]
fn a_blurred_object_s_box_is_around_everywhere_it_passes() {
    let mesh = boxed();
    let mut object = in_motion(&[moved(0.0), moved(4.0), moved(8.0)]);
    object.compute_bound(&mesh.geometry, true);
    assert!(object.bound.min[0].abs() < 1e-4 && object.bound.max[0] > 8.9 && object.bound.max[0] < 9.0 + 1e-4, "{:?}", object.bound);
    // Without motion blur its own transform places it.
    object.compute_bound(&mesh.geometry, false);
    assert_eq!(object.bound.max[0], 1.0);
}

#[test]
fn a_transform_that_was_applied_is_not_applied_again_to_the_box() {
    let mut mesh = boxed();
    let mut object = Object::new(0);
    object.set_transform(moved(10.0));
    object.compute_bound(&mesh.geometry, false);
    object.apply_transform(&mut mesh, false);
    assert_eq!(mesh.geometry.position()[0], [10.0, 0.0, 0.0]);
    assert_eq!((mesh.geometry.bound.min[0], mesh.geometry.bound.max[0]), (10.0, 11.0));
    // Once the mesh is marked, as the manager marks it, the box is the mesh's own.
    mesh.geometry.transform_applied = true;
    object.compute_bound(&mesh.geometry, false);
    assert_eq!((object.bound.min[0], object.bound.max[0]), (10.0, 11.0));
    // The transform is still the object's: readers check `transform_applied`.
    assert_eq!(object.transform(), moved(10.0));
}

#[test]
fn a_mirroring_transform_tells_the_mesh() {
    let mut mesh = boxed();
    let mut object = Object::new(0);
    object.set_transform(Transform::scale(-1.0, 1.0, 1.0));
    object.apply_transform(&mut mesh, false);
    assert!(mesh.geometry.transform_negative_scaled);
    // The identity is nothing to apply.
    let mut plain = boxed();
    plain.geometry.node.clear_modified();
    Object::new(0).apply_transform(&mut plain, false);
    assert!(!plain.geometry.node.is_modified());
}

#[test]
fn an_object_with_no_size_is_not_worth_tracing() {
    let mesh = boxed();
    let mut object = Object::new(0);
    assert!(!object.is_traceable(&mesh.geometry));
    object.compute_bound(&mesh.geometry, false);
    assert!(object.is_traceable(&mesh.geometry));
    object.bound = BoundBox { min: [1.0; 3], max: [1.0; 3] };
    assert!(!object.is_traceable(&mesh.geometry));
}

#[test]
fn a_shadow_catcher_s_visibility_is_repeated_16_bits_up() {
    let mut object = Object::new(0);
    object.node.set("visibility", Value::Uint(0b101));
    assert_eq!(object.visibility_for_tracing(), 0b101);
    object.node.set("is_shadow_catcher", Value::Boolean(true));
    assert_eq!(object.visibility_for_tracing(), (0b101 << 16) | 0b101);
}

#[test]
fn a_mesh_that_emits_and_is_seen_can_be_a_light() {
    let mut mesh = boxed();
    mesh.geometry.set_used_shader(vec![0]);
    let shader = |emission: bool| [Shader { emission, ..Shader::default() }];
    let mut object = Object::new(0);
    object.compute_bound(&mesh.geometry, false);
    assert!(object.usable_as_light(&mesh.geometry, &shader(true)) && !object.usable_as_light(&mesh.geometry, &shader(false)));
    // Seen by the camera alone, it lights nothing.
    object.node.set("visibility", Value::Uint(crate::kernel::flag("PATH_RAY_VISIBILITY_CAMERA")));
    assert!(!object.usable_as_light(&mesh.geometry, &shader(true)));
}

#[test]
fn light_linking_is_a_set_to_take_light_from_or_a_set_left_out() {
    let mut object = Object::new(0);
    assert_eq!(object.node.uint64("light_set_membership"), u64::MAX);
    object.node.set("receiver_light_set", Value::Uint(2));
    assert!(object.has_light_linking() && !object.has_shadow_linking());
    object.node.set("shadow_set_membership", Value::Uint64(0b11));
    assert!(object.has_shadow_linking());
}

#[test]
fn what_changed_is_told_as_the_manager_s_flags() {
    let mut object = Object::new(0);
    object.node.clear_modified();
    assert_eq!(object.update_flag(), 0);
    object.set_transform(moved(1.0));
    assert_eq!(object.update_flag(), ObjectManager::flag("OBJECT_MODIFIED") | ObjectManager::flag("TRANSFORM_MODIFIED"));
    object.node.clear_modified();
    object.node.set("use_holdout", Value::Boolean(true));
    object.node.set("is_shadow_catcher", Value::Boolean(true));
    assert_eq!(object.update_flag(), ["OBJECT_MODIFIED", "HOLDOUT_MODIFIED", "VISIBILITY_MODIFIED"].iter().fold(0, |mask, name| mask | ObjectManager::flag(name)));
}
