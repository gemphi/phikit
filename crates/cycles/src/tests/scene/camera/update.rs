// SPDX-License-Identifier: Apache-2.0
use super::*;
use crate::util::ProjectionTransform;

fn updated(setup: impl FnOnce(&mut Camera), need_motion: MotionType) -> Camera {
    let mut camera = Camera::new();
    setup(&mut camera);
    camera.update(need_motion);
    camera
}

fn plain() -> Camera {
    updated(|_| {}, MotionType::None)
}

/// The tangent of half the default field of view, 45 degrees.
fn half_view() -> f32 {
    (std::f32::consts::FRAC_PI_8).tan()
}

#[test]
fn the_middle_of_the_raster_looks_down_the_camera_s_axis() {
    let point = plain().rastertocamera.point([512.0, 256.0, 0.0]);
    assert!(point[0].abs() < 1e-9 && point[1].abs() < 1e-9, "{point:?}");
    // Raster depth 0 is the near clip plane.
    assert!((point[2] - 1e-5).abs() < 1e-9, "{point:?}");
}

#[test]
fn the_corner_of_the_raster_is_at_the_edge_of_the_view() {
    let point = plain().rastertocamera.point([1024.0, 512.0, 0.0]);
    // The viewplane is 2 by 1 units; the view opens by its tangent.
    assert!((point[0] / point[2] - 2.0 * half_view()).abs() < 1e-3, "{point:?}");
    assert!((point[1] / point[2] - half_view()).abs() < 1e-3, "{point:?}");
}

#[test]
fn the_world_and_the_raster_map_onto_each_other() {
    // Clip planes close to the scene: with Cycles' own, a hundred thousand
    // units apart, a float keeps too little of the depth to come back by.
    let camera = updated(
        |camera| {
            camera.node.set("matrix", Value::Transform(Transform::translate(1.0, 2.0, 3.0)));
            camera.node.set("nearclip", Value::Float(0.1));
            camera.node.set("farclip", Value::Float(100.0));
        },
        MotionType::None,
    );
    let world = [1.5, 2.25, 8.0];
    let raster = camera.worldtoraster.point(world);
    assert!(raster[0] > 512.0 && raster[1] > 256.0, "{raster:?}");
    let back = camera.rastertoworld.point(raster);
    assert!(back.iter().zip(world).all(|(back, world)| (back - world).abs() < 1e-2), "{back:?}");
    assert_eq!(camera.worldtocamera.point([1.0, 2.0, 3.0]), [0.0; 3]);
    assert_eq!(camera.cameratoworld, Transform::translate(1.0, 2.0, 3.0));
}

#[test]
fn a_pixel_is_as_wide_as_it_is_high() {
    let camera = plain();
    assert!(camera.dx[0] > 0.0 && camera.dy[1] > 0.0);
    assert_eq!((camera.dx[1], camera.dy[0]), (0.0, 0.0));
    assert!((camera.dx[0] / camera.dy[1] - 1.0).abs() < 1e-3);
    assert_eq!(camera.dx, camera.full_dx);
}

#[test]
fn an_orthographic_camera_s_pixel_is_a_step_across_the_viewplane() {
    let camera = updated(|camera| done(camera.node.set_enum("camera_type", "orthograph")), MotionType::None);
    // 4 units over 1024 pixels.
    assert!(near(&camera.dx, &[4.0 / 1024.0, 0.0, 0.0]) && near(&camera.dy, &[0.0, 2.0 / 512.0, 0.0]));
}

#[test]
fn a_panorama_has_no_pixel_step_and_no_projection() {
    let camera = updated(|camera| done(camera.node.set_enum("camera_type", "panorama")), MotionType::None);
    assert_eq!((camera.dx, camera.dy), ([0.0; 3], [0.0; 3]));
    assert_eq!(camera.worldtoscreen, ProjectionTransform::IDENTITY);
}

#[test]
fn the_frustum_s_planes_face_outward_and_mirror_each_other() {
    let camera = plain();
    let (right, left, top, bottom) = (camera.frustum_right_normal, camera.frustum_left_normal, camera.frustum_top_normal, camera.frustum_bottom_normal);
    assert!(right[0] > 0.0 && left[0] < 0.0 && top[1] > 0.0 && bottom[1] < 0.0);
    assert!((right[0] + left[0]).abs() < 1e-4 && (top[1] + bottom[1]).abs() < 1e-4);
    assert!((right.iter().map(|part| part * part).sum::<f32>() - 1.0).abs() < 1e-5);
}

#[test]
fn a_half_size_render_sees_the_same_view_through_wider_pixels() {
    let camera = updated(|camera| done(camera.set_screen_size(512, 256)), MotionType::None);
    assert!((camera.dx[0] / camera.full_dx[0] - 2.0).abs() < 1e-3);
    let point = camera.rastertocamera.point([256.0, 128.0, 0.0]);
    assert!(point[0].abs() < 1e-9 && point[1].abs() < 1e-9);
}

#[test]
fn an_update_clears_the_changes_and_asks_for_the_device() {
    let mut camera = plain();
    assert!(!camera.node.is_modified() && camera.need_device_update && camera.need_flag_update);
    camera.need_device_update = false;
    camera.update(MotionType::None);
    assert!(!camera.need_device_update);
    camera.update(MotionType::Pass);
    assert!(camera.need_device_update);
}

#[test]
fn motion_vectors_of_a_still_camera_use_the_world_to_raster_matrix() {
    let (camera, dscene, _) = written(|_| {}, MotionType::Pass);
    assert_eq!(dscene.data.float_list("cam", "perspective_pre"), camera.worldtoraster.word());
    assert_eq!(dscene.data.float_list("cam", "motion_pass_post"), camera.worldtocamera.word());
}

#[test]
fn motion_vectors_of_a_moving_camera_use_where_it_was_and_will_be() {
    let before = Transform::translate(-1.0, 0.0, 0.0);
    let after = Transform::translate(1.0, 0.0, 0.0);
    let (camera, dscene, _) = written(|camera| done(camera.node.set("motion", Value::TransformArray(vec![before, Transform::IDENTITY, after]))), MotionType::Pass);
    assert_eq!(dscene.data.float_list("cam", "motion_pass_pre"), before.inverse().word());
    assert_eq!(dscene.data.float_list("cam", "motion_pass_post"), after.inverse().word());
    assert_ne!(dscene.data.float_list("cam", "perspective_pre"), camera.worldtoraster.word());
}

#[test]
fn a_field_of_view_that_changes_over_the_shutter_is_blurred_when_asked_for() {
    let zoom = |camera: &mut Camera| {
        camera.node.set("use_perspective_motion", Value::Boolean(true));
        camera.node.set("fov_pre", Value::Float(0.5));
    };
    let (_, blurred, _) = written(zoom, MotionType::Blur);
    assert_eq!(blurred.data.int("cam", "have_perspective_motion"), 1);
    assert_ne!(blurred.data.float_list("cam", "perspective_pre"), blurred.data.float_list("cam", "perspective_post"));
    assert_eq!(written(zoom, MotionType::None).1.data.int("cam", "have_perspective_motion"), 0);
}
