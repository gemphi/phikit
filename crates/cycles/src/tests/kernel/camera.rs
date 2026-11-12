// SPDX-License-Identifier: Apache-2.0
//! Tests of `kernel/camera/camera.rs` against the closed form of a pinhole
//! camera: 50 mm lens on a 36 mm sensor fitted to the width, at (0, 0, 5)
//! looking down -z.
use super::rig::Rig;
use crate::kernel::camera::*;
use crate::kernel::types::Ray;
use crate::util::vector::{dot, len, normalize};

/// The direction of the ray through the middle of pixel `(x, y)` of a
/// `width` by `height` image (row 0 at the bottom, as Cycles counts):
/// `(sx, sy, -1)` in the world, with `sx` and `sy` the screen coordinates
/// of the pixel times the tangent of half the field of view.
fn direction(x: f32, y: f32, width: f32, height: f32) -> [f32; 3] {
    // Horizontal fit: the sensor width is the image width.
    let half_width = 0.5 * 36.0 / 50.0;
    let half_height = half_width * height / width;
    let sx = ((x + 0.5) / width * 2.0 - 1.0) * half_width;
    let sy = ((y + 0.5) / height * 2.0 - 1.0) * half_height;
    normalize([sx, sy, -1.0])
}

fn ray_of(rig: &mut Rig, x: i32, y: i32) -> Ray {
    let kg = rig.kernel();
    let mut ray = Ray::default();
    // Sample 0: the middle of the filter, the middle of the pixel.
    let throughput = camera_sample(&kg, x, y, [0.5, 0.5], 0.0, [0.5, 0.5], &mut ray);
    assert_eq!(throughput, [1.0; 3]);
    ray
}

#[test]
fn a_perspective_ray_goes_through_the_middle_of_its_pixel() {
    let mut rig = Rig::new(64, 32, [0.1; 3]);
    for (x, y) in [(0, 0), (63, 0), (0, 31), (63, 31), (31, 15), (10, 20), (50, 3)] {
        let ray = ray_of(&mut rig, x, y);
        let expect = direction(x as f32, y as f32, 64.0, 32.0);
        for axis in 0..3 {
            assert!((ray.d[axis] - expect[axis]).abs() < 1e-5, "pixel {x},{y}: {:?} against {expect:?}", ray.d);
        }
    }
}

#[test]
fn a_perspective_ray_starts_on_the_near_plane_and_ends_at_the_far_one() {
    let mut rig = Rig::new(64, 32, [0.1; 3]);
    let ray = ray_of(&mut rig, 31, 15);
    // Looking down -z from z = 5: the near plane is 0.1 along the ray,
    // and the ray is as long as the clip range (100 - 0.1) over the cosine.
    let z_inv = 1.0 / direction(31.0, 15.0, 64.0, 32.0)[2].abs();
    assert!((len(ray.p) - 5.0).abs() < 0.2 && (ray.p[2] - (5.0 - 0.1 * z_inv * ray.d[2].abs())).abs() < 1e-4, "{:?}", ray.p);
    assert!((ray.tmax - 99.9 * z_inv).abs() < 1e-2 && ray.tmin == 0.0);
    assert!((len(ray.d) - 1.0).abs() < 1e-6 && ray.time == 0.5);
}

#[test]
fn the_ray_through_a_corner_of_the_image_leans_by_half_the_sensor() {
    let mut rig = Rig::new(100, 100, [0.1; 3]);
    // The left edge of the image at its middle is at the sensor's edge:
    // tan(angle) = 18 / 50.
    let ray = ray_of(&mut rig, 0, 50);
    let tan = (ray.d[0] / ray.d[2]).abs();
    assert!((tan - 0.36 * 0.99).abs() < 0.01, "{tan}");
    assert!(ray.d[0] < 0.0 && dot(ray.d, [0.0, 0.0, -1.0]) > 0.9);
}

#[test]
fn an_orthographic_camera_sends_parallel_rays_from_a_grid() {
    let mut rig = Rig::new(60, 60, [0.1; 3]).orthographic();
    // `ortho_scale` of 6 over 60 pixels: 0.1 apart.
    let a = ray_of(&mut rig, 10, 30);
    let b = ray_of(&mut rig, 11, 30);
    let c = ray_of(&mut rig, 10, 31);
    assert_eq!(a.d, [0.0, 0.0, -1.0]);
    assert_eq!((a.d, b.d), (c.d, a.d));
    assert!((b.p[0] - a.p[0] - 0.1).abs() < 1e-5 && (c.p[1] - a.p[1] - 0.1).abs() < 1e-5);
    // The pixel's middle is half a pixel in: x = -3 + 10.5 * 0.1.
    assert!((a.p[0] - (-3.0 + 10.5 * 0.1)).abs() < 1e-3, "{:?}", a.p);
    // Starts at the near clip plane, 0.1 in front of the camera at z = 5.
    assert!((a.p[2] - 4.9).abs() < 1e-5 && (a.tmax - 99.9).abs() < 1e-3);
}

#[test]
fn the_filter_moves_the_ray_within_a_pixel() {
    let mut rig = Rig::new(64, 32, [0.1; 3]);
    let kg = rig.kernel();
    let (mut low, mut high) = (Ray::default(), Ray::default());
    camera_sample(&kg, 30, 15, [0.01, 0.5], 0.0, [0.5; 2], &mut low);
    camera_sample(&kg, 30, 15, [0.99, 0.5], 0.0, [0.5; 2], &mut high);
    // The default box filter of a pixel: the ray moves by about a pixel.
    let step = (high.d[0] - low.d[0]) * 64.0 / (2.0 * 0.36);
    assert!(step > 0.9 && step < 1.1, "{step} pixels");
}
