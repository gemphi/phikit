// SPDX-License-Identifier: Apache-2.0
use super::*;
use crate::util::{motion_array_interpolate, motion_decompose};

fn before() -> Transform {
    Transform::translate(-1.0, 0.0, 0.0)
}

fn after() -> Transform {
    Transform::translate(1.0, 2.0, 0.0)
}

/// A camera that is at the origin in the middle of the shutter.
fn moving(camera: &mut Camera) {
    camera.node.set("motion", Value::TransformArray(vec![before(), Transform::IDENTITY, after()]));
}

#[test]
fn motion_vectors_are_a_pass_and_blur_is_not() {
    assert!(MotionType::Pass.is_pass() && MotionType::PassInteractive.is_pass());
    assert!(!MotionType::Blur.is_pass() && !MotionType::None.is_pass());
}

#[test]
fn motion_steps_run_from_the_shutter_s_open_to_its_close() {
    let mut camera = Camera::new();
    assert!(!camera.use_motion());
    assert_eq!((camera.motion_time(0), camera.motion_step(0.0)), (0.0, None));
    camera.node.set("motion", Value::TransformArray(vec![Transform::IDENTITY; 3]));
    assert!(camera.use_motion());
    assert_eq!([0, 1, 2].map(|step| camera.motion_time(step)), [-1.0, 0.0, 1.0]);
    assert_eq!((camera.motion_step(1.0), camera.motion_step(0.5)), (Some(2), None));
}

#[test]
fn for_blur_a_moving_camera_s_places_go_to_the_device() {
    let (_, dscene, _) = written(moving, MotionType::Blur);
    assert_eq!((dscene.camera_motion.len(), dscene.data.int("cam", "num_motion_steps")), (3, 3));
    assert_eq!(dscene.camera_motion, motion_decompose(&[before(), Transform::IDENTITY, after()]));
    // A kernel finds the camera at any moment of the shutter from them.
    let at = |time: f32| motion_array_interpolate(&dscene.camera_motion, time).word();
    assert!(near(&at(0.0), &before().word()) && near(&at(0.5), &Transform::IDENTITY.word()) && near(&at(1.0), &after().word()));
    assert!(near(&at(0.75), &Transform::translate(0.5, 1.0, 0.0).word()));
}

#[test]
fn a_camera_that_stands_still_has_none() {
    let still = |camera: &mut Camera| done(camera.node.set("motion", Value::TransformArray(vec![Transform::IDENTITY; 3])));
    let (_, dscene, _) = written(still, MotionType::Blur);
    assert!(dscene.camera_motion.is_empty());
    assert_eq!(dscene.data.int("cam", "num_motion_steps"), 0);
}

#[test]
fn without_blur_its_places_are_not_decomposed() {
    for need_motion in [MotionType::None, MotionType::Pass, MotionType::PassInteractive] {
        let (_, dscene, _) = written(moving, need_motion);
        assert!(dscene.camera_motion.is_empty(), "{need_motion:?}");
        assert_eq!(dscene.data.int("cam", "num_motion_steps"), 0);
    }
}

#[test]
fn a_camera_that_stops_moving_empties_the_array() {
    let (mut camera, mut dscene, mut lookup) = written(moving, MotionType::Blur);
    camera.node.set("motion", Value::TransformArray(Vec::new()));
    camera.device_update(&mut dscene, &mut lookup, MotionType::Blur);
    assert!(dscene.camera_motion.is_empty());
    assert_eq!(dscene.data.int("cam", "num_motion_steps"), 0);
}

#[test]
fn freeing_gives_the_array_up() {
    let (mut camera, mut dscene, mut lookup) = written(moving, MotionType::Blur);
    camera.device_free(&mut dscene, &mut lookup);
    assert!(dscene.camera_motion.is_empty());
    assert_eq!(lookup.table_count(), 0);
}

#[test]
fn in_a_viewport_where_the_camera_is_becomes_where_it_was() {
    let mut camera = Camera::new();
    let here = Transform::translate(0.0, 0.0, 4.0);
    camera.node.set("matrix", Value::Transform(here));
    camera.node.set("fov", Value::Float(0.5));
    camera.node.set("fov_pre", Value::Float(0.25));
    moving(&mut camera);
    camera.node.clear_modified();
    camera.update_interactive_motion();
    assert_eq!(camera.node.transform_array("motion"), [here, Transform::IDENTITY, after()]);
    assert_eq!(camera.node.float("fov_pre"), 0.5);
    // It moved since the frame before: the next update must see that.
    assert!(camera.node.is_modified());
    // Once it rests, nothing changes any more.
    camera.node.clear_modified();
    camera.update_interactive_motion();
    assert!(!camera.node.is_modified());
}

#[test]
fn a_camera_with_no_motion_steps_keeps_none() {
    let mut camera = Camera::new();
    camera.node.set("fov", Value::Float(0.5));
    camera.update_interactive_motion();
    assert!(camera.node.transform_array("motion").is_empty());
    assert_eq!(camera.node.float("fov_pre"), 0.5);
}
