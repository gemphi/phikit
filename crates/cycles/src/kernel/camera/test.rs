// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Tests of this module. It also holds what the others send a ray with.


pub(crate) use crate::kernel::test::near;
pub(crate) use crate::graph::Value;
pub(crate) use crate::kernel::camera::*;
pub(crate) use crate::kernel::{KernelGlobal, Ray, RaySelfPrimitive, flag};
pub(crate) use crate::scene::{Camera, DeviceScene, LookupTableSet, MotionType};
pub(crate) use crate::kernel::test::rig::Rig;
pub(crate) use crate::util::Transform;

/// A camera changed by `setup`, written to a new device scene.
pub(crate) fn scene(setup: impl FnOnce(&mut Camera), need_motion: MotionType) -> (Camera, DeviceScene) {
    let (mut camera, mut dscene, mut lookup) = (Camera::new(), DeviceScene::new(), LookupTableSet::new());
    setup(&mut camera);
    camera.device_update(&mut dscene, &mut lookup, need_motion);
    (camera, dscene)
}

pub(crate) fn still(setup: impl FnOnce(&mut Camera)) -> (Camera, DeviceScene) {
    scene(setup, MotionType::None)
}

/// The ray through the top left corner of `pixel`, for the random numbers
/// `time` and `lens`, and what the camera lets through.
pub(crate) fn ray_of(dscene: &DeviceScene, pixel: [i32; 2], time: f32, lens: [f32; 2]) -> (Ray, [f32; 3]) {
    let kg = KernelGlobal::of(dscene);
    let mut ray = Ray::default();
    let through = camera_sample(&kg, pixel[0], pixel[1], [0.5; 2], time, lens, &mut ray);
    (ray, through)
}

/// Where a ray is at the depth `z` in front of a camera at the origin.
pub(crate) fn at_depth(ray: &Ray, z: f32) -> [f32; 3] {
    let t = (z - ray.p[2]) / ray.d[2];
    [0, 1, 2].map(|axis| ray.p[axis] + ray.d[axis] * t)
}

/// A camera whose places over the shutter are apart.
pub(crate) fn moving(camera: &mut Camera) {
    camera.node.set("motion", Value::TransformArray(vec![Transform::translate(-1.0, 0.0, 0.0), Transform::IDENTITY, Transform::translate(1.0, 2.0, 0.0)]));
}

#[test]
fn the_middle_of_the_image_looks_down_the_camera_s_axis() {
    let (camera, dscene) = still(|_| {});
    let (ray, through) = ray_of(&dscene, [512, 256], 0.3, [0.0; 2]);
    let (near_clip, far_clip) = (camera.node.float("nearclip"), camera.node.float("farclip"));
    assert_eq!(through, [1.0; 3]);
    assert!(near(&ray.d, &[0.0, 0.0, 1.0]) && near(&ray.p, &[0.0, 0.0, near_clip]), "{ray:?}");
    // Without motion blur every ray is sent in the middle of the shutter.
    assert_eq!((ray.tmin, ray.time, ray.own), (0.0, 0.5, RaySelfPrimitive::default()));
    assert!((ray.tmax / (far_clip - near_clip) - 1.0).abs() < 1e-4, "{}", ray.tmax);
}

#[test]
fn a_camera_that_was_moved_sends_its_rays_from_where_it_is() {
    let (_, dscene) = still(|camera| {
        camera.node.set("matrix", Value::Transform(Transform::translate(1.0, 2.0, 3.0)));
    });
    let (ray, _) = ray_of(&dscene, [512, 256], 0.0, [0.0; 2]);
    assert!(near(&ray.p, &[1.0, 2.0, 3.0]) && near(&ray.d, &[0.0, 0.0, 1.0]), "{ray:?}");
    let kg = KernelGlobal::of(&dscene);
    assert_eq!(camera_position(&kg), [1.0, 2.0, 3.0]);
    assert!(near(&[camera_distance(&kg, [1.0, 2.0, 7.0]), camera_distance(&kg, [4.0, 6.0, 3.0])], &[4.0, 5.0]));
}

#[test]
fn the_lens_is_a_disk_or_a_polygon_of_blades_squeezed_for_an_anamorphic_one() {
    let (_, dscene) = still(|camera| {
        camera.node.set("aperture_ratio", Value::Float(2.0));
    });
    let round = KernelGlobal::of(&dscene).cam;
    assert!(near(&camera_sample_aperture(&round, [1.0, 0.5]), &[0.5, 0.0]));
    assert!(near(&camera_sample_aperture(&round, [0.5, 1.0]), &[0.0, 1.0]));
    let (_, dscene) = still(|camera| {
        camera.node.set("blades", Value::Int(4));
    });
    let square = KernelGlobal::of(&dscene).cam;
    let corner = std::f32::consts::FRAC_1_SQRT_2;
    assert!(near(&camera_sample_aperture(&square, [0.0, 0.0]), &[corner, corner]));
    assert!(near(&camera_sample_aperture(&square, [0.25, 0.0]), &[-corner, corner]));
}

#[test]
fn with_motion_blur_the_shutter_s_curve_says_when_a_ray_is_sent() {
    let (_, dscene) = scene(|_| {}, MotionType::Blur);
    // A shutter that is open evenly sends rays evenly over its time.
    let time = [0.0, 0.25, 0.75, 1.0].map(|random| ray_of(&dscene, [512, 256], random, [0.0; 2]).0.time);
    assert!(time.iter().zip([0.0, 0.25, 0.75, 1.0]).all(|(time, random)| (time - random).abs() < 0.01), "{time:?}");
}

#[test]
fn a_rolling_shutter_sends_each_row_at_its_own_moment() {
    let rolling = |duration: f32| {
        let setup = |camera: &mut Camera| {
            camera.node.set_enum("rolling_shutter_type", "top");
            camera.node.set("rolling_shutter_duration", Value::Float(duration));
        };
        let (_, dscene) = scene(setup, MotionType::Blur);
        [0.25, 0.75].map(|random| ray_of(&dscene, [512, 128], random, [0.0; 2]).0.time)
    };
    // Only rolling: the row alone decides, a quarter down from the top.
    assert!(near(&rolling(0.0), &[0.75, 0.75]));
    // Half rolling, half blur.
    let half = rolling(0.5);
    assert!((half[0] - 0.5).abs() < 0.01 && (half[1] - 0.75).abs() < 0.01, "{half:?}");
}

#[test]
fn a_camera_that_moves_sends_each_ray_from_where_it_is_at_that_moment() {
    let (_, dscene) = scene(moving, MotionType::Blur);
    let origin = |random: f32| ray_of(&dscene, [512, 256], random, [0.0; 2]).0.p;
    let close = |left: [f32; 3], right: [f32; 3]| left.iter().zip(right).all(|(left, right)| (left - right).abs() < 0.03);
    assert!(close(origin(0.0), [-1.0, 0.0, 0.0]) && close(origin(0.5), [0.0; 3]) && close(origin(1.0), [1.0, 2.0, 0.0]));
    // For motion vectors the camera's places are not written, and it stays.
    let (_, dscene) = scene(moving, MotionType::Pass);
    assert!(near(&ray_of(&dscene, [512, 256], 0.0, [0.0; 2]).0.p, &[0.0; 3]));
}

#[test]
fn the_pixel_filter_s_table_places_a_ray_within_its_pixel() {
    let (_, mut dscene) = still(|_| {});
    let size = flag("FILTER_TABLE_SIZE") as usize;
    dscene.lookup_table = (0..size).map(|index| index as f32 / (size - 1) as f32).collect();
    dscene.data.set_int("tables", "filter_table_offset", 0);
    dscene.data.set_float2("integrator", "pixel_jitter", [f32::MAX; 2]);
    let kg = KernelGlobal::of(&dscene);
    let mut ray = Ray::default();
    // The table's last entry is one pixel: from 511, 255 to the middle.
    camera_sample(&kg, 511, 255, [1.0; 2], 0.0, [0.0; 2], &mut ray);
    assert!(near(&ray.d, &[0.0, 0.0, 1.0]), "{ray:?}");
    camera_sample(&kg, 512, 256, [0.0; 2], 0.0, [0.0; 2], &mut ray);
    assert!(near(&ray.d, &[0.0, 0.0, 1.0]), "{ray:?}");
}

#[test]
fn a_fixed_pixel_jitter_moves_every_ray_by_it() {
    let (_, mut dscene) = still(|_| {});
    dscene.data.set_float2("integrator", "pixel_jitter", [-0.5, -0.5]);
    let (ray, _) = ray_of(&dscene, [511, 255], 0.0, [0.0; 2]);
    let (middle, _) = ray_of(&dscene, [512, 256], 0.0, [0.0; 2]);
    assert!(ray.d[0] != middle.d[0] && ray.d[1] != middle.d[1]);
    dscene.data.set_float2("integrator", "pixel_jitter", [-1.0, -1.0]);
    assert!(near(&ray_of(&dscene, [511, 255], 0.0, [0.0; 2]).0.d, &[0.0, 0.0, 1.0]));
}

#[test]
fn a_camera_that_is_not_ported_sends_no_ray() {
    let (_, dscene) = still(|camera| {
        camera.node.set_enum("camera_type", "panorama");
    });
    assert_eq!(ray_of(&dscene, [512, 256], 0.0, [0.0; 2]).1, [0.0; 3]);
    let (_, dscene) = still(|camera| {
        camera.node.set_enum("camera_type", "custom");
    });
    assert_eq!(ray_of(&dscene, [512, 256], 0.0, [0.0; 2]).1, [0.0; 3]);
}
/// The ray through the middle of pixel `(x, y)` of the joined camera of
/// `kernel/test/rig.rs`, for sample 0: the middle of the filter.
pub(crate) fn through(rig: &mut Rig, x: i32, y: i32) -> Ray {
    let kg = rig.kernel();
    let mut ray = Ray::default();
    let throughput = camera_sample(&kg, x, y, [0.5, 0.5], 0.0, [0.5, 0.5], &mut ray);
    assert_eq!(throughput, [1.0; 3]);
    ray
}
#[test]
fn the_box_filter_moves_a_joined_camera_s_ray_within_a_pixel() {
    let mut rig = Rig::new(64, 32, [0.1; 3]);
    let kg = rig.kernel();
    let (mut low, mut high) = (Ray::default(), Ray::default());
    camera_sample(&kg, 30, 15, [0.01, 0.5], 0.0, [0.5; 2], &mut low);
    camera_sample(&kg, 30, 15, [0.99, 0.5], 0.0, [0.5; 2], &mut high);
    // The default box filter of a pixel: the ray moves by about a pixel.
    let step = (high.d[0] - low.d[0]) * 64.0 / (2.0 * 0.36);
    assert!(step > 0.9 && step < 1.1, "{step} pixels");
}