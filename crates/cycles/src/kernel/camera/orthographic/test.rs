// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::kernel::camera::test::*;

fn flat(camera: &mut Camera) {
    camera.node.set_enum("camera_type", "orthograph");
}

#[test]
fn every_ray_goes_the_same_way_and_they_start_side_by_side() {
    let (camera, dscene) = still(flat);
    let (first, through) = ray_of(&dscene, [0, 0], 0.0, [0.0; 2]);
    let (last, _) = ray_of(&dscene, [1024, 512], 0.0, [0.0; 2]);
    assert_eq!(through, [1.0; 3]);
    assert!(near(&first.d, &[0.0, 0.0, 1.0]) && near(&last.d, &[0.0, 0.0, 1.0]));
    // The viewplane's corners: 2 across and 1 up, on the near plane.
    let near_clip = camera.node.float("nearclip");
    assert!(near(&[first.p[0].abs(), first.p[1].abs(), first.p[2]], &[2.0, 1.0, near_clip]), "{first:?}");
    assert!(near(&first.p, &[-last.p[0], -last.p[1], last.p[2]]));
}

#[test]
fn a_ray_is_as_long_as_the_clipping_and_parted_from_the_next_by_a_pixel() {
    let (_, dscene) = still(|camera| {
        flat(camera);
        camera.node.set("nearclip", Value::Float(0.5));
        camera.node.set("farclip", Value::Float(10.0));
    });
    let (ray, _) = ray_of(&dscene, [300, 300], 0.0, [0.0; 2]);
    assert!(near(&[ray.tmin, ray.tmax, ray.p[2]], &[0.0, 9.5, 0.5]));
    // A pixel is 4 / 1024 of the viewplane across and 2 / 512 up.
    assert!(near(&[ray.dp, ray.dd], &[4.0 / 1024.0, 0.0]) && ray.dd == 0.0);
}

#[test]
fn with_an_open_lens_the_rays_of_a_pixel_meet_where_it_is_sharp() {
    let (_, pinhole) = still(flat);
    let (_, dscene) = still(|camera| {
        flat(camera);
        camera.node.set("focaldistance", Value::Float(4.0));
        camera.node.set("aperturesize", Value::Float(0.1));
    });
    let pixel = [700, 100];
    let straight = ray_of(&pinhole, pixel, 0.0, [0.0; 2]).0.p;
    let focus = [straight[0], straight[1], 4.0];
    let (first, _) = ray_of(&dscene, pixel, 0.0, [0.2, 0.7]);
    let (second, _) = ray_of(&dscene, pixel, 0.0, [0.9, 0.1]);
    assert!(near(&at_depth(&first, 4.0), &focus) && near(&at_depth(&second, 4.0), &focus), "{first:?} {second:?}");
    assert!(first.d != second.d && first.p != second.p);
}

#[test]
fn the_distance_to_an_orthographic_camera_is_to_its_plane() {
    let (_, dscene) = still(flat);
    let kg = KernelGlobal::of(&dscene);
    assert!(near(&[camera_distance(&kg, [5.0, 5.0, 3.0])], &[3.0]));
}
#[test]
fn a_joined_orthographic_camera_sends_parallel_rays_from_a_grid() {
    let mut rig = Rig::new(60, 60, [0.1; 3]).orthographic();
    // `ortho_scale` of 6 over 60 pixels: 0.1 apart.
    let a = through(&mut rig, 10, 30);
    let b = through(&mut rig, 11, 30);
    let c = through(&mut rig, 10, 31);
    assert_eq!(a.d, [0.0, 0.0, -1.0]);
    assert_eq!((a.d, b.d), (c.d, a.d));
    assert!((b.p[0] - a.p[0] - 0.1).abs() < 1e-5 && (c.p[1] - a.p[1] - 0.1).abs() < 1e-5);
    // The pixel's middle is half a pixel in: x = -3 + 10.5 * 0.1.
    assert!((a.p[0] - (-3.0 + 10.5 * 0.1)).abs() < 1e-3, "{:?}", a.p);
    // Starts at the near clip plane, 0.1 in front of the camera at z = 5.
    assert!((a.p[2] - 4.9).abs() < 1e-5 && (a.tmax - 99.9).abs() < 1e-3);
}