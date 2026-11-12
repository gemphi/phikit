// SPDX-License-Identifier: Apache-2.0
use super::*;

/// How far off the axis a ray goes, across and up, for each unit forward.
fn slope(ray: &Ray) -> [f32; 2] {
    [(ray.d[0] / ray.d[2]).abs(), (ray.d[1] / ray.d[2]).abs()]
}

#[test]
fn the_corners_of_the_image_are_the_corners_of_the_field_of_view() {
    let (camera, dscene) = still(|_| {});
    let (first, _) = ray_of(&dscene, [0, 0], 0.0, [0.0; 2]);
    let (last, _) = ray_of(&dscene, [1024, 512], 0.0, [0.0; 2]);
    // The viewplane of an image twice as wide as high reaches 2 across and
    // 1 up, at the distance the field of view sets.
    let half = (camera.node.float("fov") * 0.5).tan();
    assert!(near(&slope(&first), &[2.0 * half, half]), "{first:?}");
    assert!(near(&first.d, &[-last.d[0], -last.d[1], last.d[2]]));
    assert!(near(&[crate::util::vector::len(first.d)], &[1.0]));
}

#[test]
fn a_ray_starts_on_the_near_plane_and_ends_on_the_far_one() {
    let (_, dscene) = still(|camera| {
        camera.node.set("nearclip", Value::Float(0.5));
        camera.node.set("farclip", Value::Float(10.0));
    });
    // Off the axis too: the planes are flat, so that ray is longer.
    for pixel in [[512, 256], [0, 0], [900, 100]] {
        let (ray, _) = ray_of(&dscene, pixel, 0.0, [0.0; 2]);
        assert!(near(&[ray.p[2], ray.p[2] + ray.d[2] * ray.tmax], &[0.5, 10.0]), "{pixel:?}: {ray:?}");
    }
    let (middle, _) = ray_of(&dscene, [512, 256], 0.0, [0.0; 2]);
    let (corner, _) = ray_of(&dscene, [0, 0], 0.0, [0.0; 2]);
    assert!(corner.tmax > middle.tmax);
}

#[test]
fn neighbouring_rays_spread_by_a_pixel_and_start_together() {
    let (camera, dscene) = still(|_| {});
    let (ray, _) = ray_of(&dscene, [512, 256], 0.0, [0.0; 2]);
    // One pixel is 4 / 1024 of the viewplane across and 2 / 512 up.
    let half = (camera.node.float("fov") * 0.5).tan();
    assert!((ray.dd / (half / 256.0) - 1.0).abs() < 1e-3, "{}", ray.dd);
    // They start at one point; the near plane is all that parts them.
    assert!((ray.dp - camera.node.float("nearclip") * ray.dd).abs() < 1e-9);
}

#[test]
fn with_an_open_lens_the_rays_of_a_pixel_meet_where_it_is_sharp() {
    let sharp = |camera: &mut Camera| {
        camera.node.set("focaldistance", Value::Float(4.0));
    };
    let (_, pinhole) = still(sharp);
    let (_, dscene) = still(|camera| {
        sharp(camera);
        camera.node.set("aperturesize", Value::Float(0.1));
    });
    let pixel = [100, 400];
    let focus = at_depth(&ray_of(&pinhole, pixel, 0.0, [0.0; 2]).0, 4.0);
    let (first, _) = ray_of(&dscene, pixel, 0.0, [0.1, 0.9]);
    let (second, _) = ray_of(&dscene, pixel, 0.0, [0.8, 0.3]);
    assert!(near(&at_depth(&first, 4.0), &focus) && near(&at_depth(&second, 4.0), &focus), "{first:?} {second:?}");
    // They start apart, on the lens, and no further out than its radius.
    assert!(first.p != second.p);
    assert!([first, second].iter().all(|ray| (ray.p[0] * ray.p[0] + ray.p[1] * ray.p[1]).sqrt() <= 0.1 + 1e-4));
    // The spread is the pixel's, whatever the lens does.
    assert_eq!(first.dd, ray_of(&pinhole, pixel, 0.0, [0.0; 2]).0.dd);
}

#[test]
fn a_field_of_view_that_changes_is_mixed_over_the_shutter() {
    let (camera, dscene) = scene(
        |camera| {
            let fov = camera.node.float("fov");
            camera.node.set("use_perspective_motion", Value::Boolean(true));
            camera.node.set("fov_pre", Value::Float(fov * 0.5));
            camera.node.set("fov_post", Value::Float(fov));
        },
        MotionType::Blur,
    );
    assert!(KernelGlobal::of(&dscene).cam.have_perspective_motion);
    let fov = camera.node.float("fov");
    let across = |random: f32| slope(&ray_of(&dscene, [0, 0], random, [0.0; 2]).0)[0];
    // Narrow when the shutter opens, the camera's own from the middle on.
    let (open, middle, close) = (across(0.0), across(0.5), across(1.0));
    assert!((open - 2.0 * (fov * 0.25).tan()).abs() < 0.01, "{open}");
    assert!((middle - 2.0 * (fov * 0.5).tan()).abs() < 0.01 && (close - middle).abs() < 0.01, "{middle} {close}");
    assert!(across(0.25) > open && across(0.25) < middle);
}

#[test]
fn a_stereo_perspective_is_not_ported_and_sends_no_ray() {
    let (_, mut dscene) = still(|_| {});
    dscene.data.set_float("cam", "interocular_offset", 0.03);
    assert_eq!(ray_of(&dscene, [512, 256], 0.0, [0.0; 2]).1, [0.0; 3]);
}
/// The direction of the ray through the middle of pixel `(x, y)` of a
/// `width` by `height` image (row 0 at the bottom, as Cycles counts) from
/// the joined camera of `tests/kernel/rig.rs`: 50 mm on a 36 mm sensor
/// fitted to the width, at (0, 0, 5) looking down -z.
fn pinhole(x: f32, y: f32, width: f32, height: f32) -> [f32; 3] {
    let half_width = 0.5 * 36.0 / 50.0;
    let half_height = half_width * height / width;
    let sx = ((x + 0.5) / width * 2.0 - 1.0) * half_width;
    let sy = ((y + 0.5) / height * 2.0 - 1.0) * half_height;
    crate::util::vector::normalize([sx, sy, -1.0])
}
#[test]
fn a_joined_camera_s_ray_goes_through_the_middle_of_its_pixel() {
    let mut rig = Rig::new(64, 32, [0.1; 3]);
    for (x, y) in [(0, 0), (63, 0), (0, 31), (63, 31), (31, 15), (10, 20), (50, 3)] {
        let ray = through(&mut rig, x, y);
        let expect = pinhole(x as f32, y as f32, 64.0, 32.0);
        for axis in 0..3 {
            assert!((ray.d[axis] - expect[axis]).abs() < 1e-5, "pixel {x},{y}: {:?} against {expect:?}", ray.d);
        }
    }
}
#[test]
fn a_joined_camera_s_ray_starts_on_the_near_plane_and_ends_at_the_far_one() {
    let mut rig = Rig::new(64, 32, [0.1; 3]);
    let ray = through(&mut rig, 31, 15);
    // Looking down -z from z = 5: the near plane is 0.1 along the ray,
    // and the ray is as long as the clip range (100 - 0.1) over the cosine.
    let z_inv = 1.0 / pinhole(31.0, 15.0, 64.0, 32.0)[2].abs();
    assert!((crate::util::vector::len(ray.p) - 5.0).abs() < 0.2 && (ray.p[2] - (5.0 - 0.1 * z_inv * ray.d[2].abs())).abs() < 1e-4, "{:?}", ray.p);
    assert!((ray.tmax - 99.9 * z_inv).abs() < 1e-2 && ray.tmin == 0.0);
    assert!((crate::util::vector::len(ray.d) - 1.0).abs() < 1e-6 && ray.time == 0.5);
}
#[test]
fn the_ray_through_the_edge_of_the_image_leans_by_half_the_sensor() {
    let mut rig = Rig::new(100, 100, [0.1; 3]);
    // The left edge of the image at its middle is at the sensor's edge:
    // tan(angle) = 18 / 50.
    let ray = through(&mut rig, 0, 50);
    let tan = (ray.d[0] / ray.d[2]).abs();
    assert!((tan - 0.36 * 0.99).abs() < 0.01, "{tan}");
    assert!(ray.d[0] < 0.0 && crate::util::vector::dot(ray.d, [0.0, 0.0, -1.0]) > 0.9);
}