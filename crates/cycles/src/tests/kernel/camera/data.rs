// SPDX-License-Identifier: Apache-2.0
use super::*;
use crate::kernel::constant;

#[test]
fn a_camera_type_is_named_by_cycles_number() {
    let number = |name: &str| constant(name).expect("in data/constant.tsv") as i32;
    assert_eq!(CameraType::of(number("CAMERA_PERSPECTIVE")), Some(CameraType::Perspective));
    assert_eq!(CameraType::of(number("CAMERA_ORTHOGRAPHIC")), Some(CameraType::Orthographic));
    assert_eq!(CameraType::of(number("CAMERA_PANORAMA")), Some(CameraType::Panorama));
    assert_eq!(CameraType::of(number("CAMERA_CUSTOM")), Some(CameraType::Custom));
    assert_eq!(CameraType::of(99), None);
}

#[test]
fn the_kernel_s_camera_is_what_the_scene_s_camera_wrote() {
    let place = Transform::translate(1.0, 2.0, 3.0);
    let (camera, dscene) = still(|camera| {
        camera.node.set("matrix", Value::Transform(place));
        camera.node.set("aperturesize", Value::Float(0.25));
        camera.node.set("blades", Value::Int(5));
    });
    let cam = KernelCamera::of(&dscene.data);
    assert_eq!((cam.kind, cam.cameratoworld, cam.rastertocamera), (Some(CameraType::Perspective), place, camera.rastertocamera));
    assert_eq!((cam.aperturesize, cam.blades, cam.height), (0.25, 5.0, 512.0));
    assert_eq!((cam.nearclip, cam.inv_aperture_ratio), (camera.node.float("nearclip"), 1.0));
    // No motion blur: the shutter is marked shut and nothing moves.
    assert_eq!((cam.shuttertime, cam.motion_step_count, cam.have_perspective_motion), (-1.0, 0, false));
    assert_eq!((cam.dx, cam.dy), (camera.dx, camera.dy));
}

#[test]
fn with_motion_blur_it_knows_the_shutter_and_the_camera_s_steps() {
    let (camera, dscene) = scene(moving, MotionType::Blur);
    let cam = KernelCamera::of(&dscene.data);
    assert_eq!((cam.shuttertime, cam.motion_step_count), (camera.node.float("shuttertime"), 3));
    assert_eq!(cam.rolling_shutter_type, 0);
}
