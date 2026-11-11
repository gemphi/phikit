// SPDX-License-Identifier: Apache-2.0
//! Tests of `src/scene/camera`, one file for each file there. This file
//! tests `mod.rs` and holds what the others write a camera with.

mod motion;
mod update;

use crate::graph::Value;
use crate::scene::*;
use crate::util::{BoundBox2D, Transform};

/// A camera changed by `setup`, written to a new device scene.
fn written(setup: impl FnOnce(&mut Camera), need_motion: MotionType) -> (Camera, DeviceScene, LookupTableSet) {
    let (mut camera, mut dscene, mut lookup) = (Camera::new(), DeviceScene::new(), LookupTableSet::new());
    setup(&mut camera);
    camera.device_update(&mut dscene, &mut lookup, need_motion);
    (camera, dscene, lookup)
}

/// A setter's answer, whether the value changed, is of no interest here.
fn done(_changed: bool) {}

fn still(setup: impl FnOnce(&mut Camera)) -> DeviceScene {
    written(setup, MotionType::None).1
}

fn float(dscene: &DeviceScene, name: &str) -> f32 {
    dscene.data.float("cam", name)
}

fn near(left: &[f32], right: &[f32]) -> bool {
    left.len() == right.len() && left.iter().zip(right).all(|(left, right)| (left - right).abs() < 1e-4)
}

#[test]
fn a_new_camera_is_a_perspective_on_1024_by_512_with_the_viewplane_to_fit() {
    let camera = Camera::new();
    assert_eq!((camera.width, camera.height, camera.kind()), (1024, 512, "perspective"));
    assert_eq!(camera.bound("viewplane"), BoundBox2D::of_radius([2.0, 1.0]));
    assert_eq!((camera.bound("border"), camera.bound("viewport_camera_border")), (BoundBox2D::default(), BoundBox2D::default()));
    assert_eq!(camera.node.float_array("shutter_curve").len(), 256);
}

#[test]
fn the_viewplane_is_one_unit_to_the_short_side() {
    let mut camera = Camera::new();
    camera.node.set("full_width", Value::Int(500));
    camera.node.set("full_height", Value::Int(1000));
    camera.compute_auto_viewplane();
    assert_eq!(camera.bound("viewplane"), BoundBox2D::of_radius([1.0, 2.0]));
    camera.node.set_enum("camera_type", "panorama");
    camera.compute_auto_viewplane();
    assert_eq!(camera.bound("viewplane"), BoundBox2D::default());
}

#[test]
fn a_new_screen_size_counts_as_a_change() {
    let mut camera = Camera::new();
    camera.node.clear_modified();
    assert!(!camera.set_screen_size(1024, 512));
    assert!(!camera.node.is_modified());
    assert!(camera.set_screen_size(512, 256));
    assert!(camera.node.is_modified());
    assert_eq!((camera.width, camera.height), (512, 256));
}

#[test]
fn the_lens_and_the_size_reach_the_kernel() {
    let dscene = still(|_| {});
    assert_eq!((dscene.data.int("cam", "type"), float(&dscene, "width"), float(&dscene, "height")), (0, 1024.0, 512.0));
    assert_eq!((float(&dscene, "focaldistance"), float(&dscene, "aperturesize"), float(&dscene, "blades")), (10.0, 0.0, 0.0));
    assert_eq!((float(&dscene, "sensorwidth"), float(&dscene, "inv_aperture_ratio"), float(&dscene, "differential_scale")), (0.036, 1.0, 1.0));
    assert_eq!((float(&dscene, "nearclip"), float(&dscene, "cliplength")), (1e-5, 1e5 - 1e-5));
    // No motion blur: the shutter time says so.
    assert_eq!(float(&dscene, "shuttertime"), -1.0);
    assert_eq!(dscene.data.float_list("cam", "cameratoworld"), Transform::IDENTITY.word());
}

#[test]
fn fewer_than_three_blades_is_a_round_aperture() {
    let blades = |count: u32| float(&still(|camera| done(camera.node.set("blades", Value::Uint(count)))), "blades");
    assert_eq!([2, 3, 6].map(blades), [0.0, 3.0, 6.0]);
}

#[test]
fn a_panorama_s_ranges_are_stored_ready_for_the_kernel() {
    let dscene = still(|_| {});
    let pi = std::f32::consts::PI;
    assert!(near(&dscene.data.float_list("cam", "equirectangular_range"), &[-2.0 * pi, pi, -pi, pi]));
    assert!(near(&dscene.data.float_list("cam", "central_cylindrical_range"), &[pi, -pi, -1.0, 1.0]));
    assert_eq!((float(&dscene, "pole_merge_angle_from"), float(&dscene, "pole_merge_angle_to")), (-1.0, -1.0));
}

#[test]
fn a_stereo_eye_is_half_the_distance_between_the_eyes_to_its_side() {
    let eye = |name: &'static str| float(&still(move |camera| done(camera.node.set_enum("stereo_eye", name))), "interocular_offset");
    assert_eq!([eye("none"), eye("left"), eye("right")], [0.0, -0.0325, 0.0325]);
}

#[test]
fn a_smaller_render_scales_the_differentials() {
    let dscene = still(|camera| done(camera.set_screen_size(512, 256)));
    assert_eq!((float(&dscene, "width"), float(&dscene, "differential_scale")), (512.0, 0.5));
}

#[test]
fn an_unchanged_camera_writes_nothing_until_the_motion_needed_changes() {
    let (mut camera, mut dscene, mut lookup) = written(|_| {}, MotionType::None);
    assert!(!camera.need_device_update);
    dscene.data.set_float("cam", "width", 7.0);
    camera.device_update(&mut dscene, &mut lookup, MotionType::None);
    assert_eq!(float(&dscene, "width"), 7.0);
    camera.device_update(&mut dscene, &mut lookup, MotionType::Blur);
    assert_eq!((float(&dscene, "width"), float(&dscene, "shuttertime")), (1024.0, 1.0));
}

#[test]
fn motion_blur_puts_the_shutter_s_curve_into_the_lookup_tables() {
    let (mut camera, dscene, mut lookup) = written(|_| {}, MotionType::Blur);
    assert_eq!((lookup.table_count(), dscene.lookup_table.len(), dscene.data.int("cam", "shutter_table_offset")), (1, 256, 0));
    // A shutter open all the time draws every moment alike.
    assert!(dscene.lookup_table.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(dscene.lookup_table.iter().all(|time| (0.0..1.0).contains(time)));
    camera.device_free(&mut DeviceScene::new(), &mut lookup);
    assert_eq!(lookup.table_count(), 0);
    assert_eq!(written(|_| {}, MotionType::None).2.table_count(), 0);
}

#[test]
fn a_shutter_closed_for_its_first_half_draws_from_the_second() {
    let curve: Vec<f32> = (0..256).map(|index| f32::from(u8::from(index >= 128))).collect();
    let (_, dscene, _) = written(|camera| done(camera.node.set("shutter_curve", Value::FloatArray(curve))), MotionType::Blur);
    assert!(dscene.lookup_table.iter().all(|time| *time > 0.49), "{:?}", &dscene.lookup_table[..4]);
}
