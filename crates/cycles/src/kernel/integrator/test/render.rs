// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! The kernels together, against closed forms: a plane that emits, a white
//! object in a uniform world, a diffuse plane under the sky, the film.
use crate::kernel::test::rig::{Rig, cube, plane, translation};
use crate::scene::Surface;

fn near(a: [f32; 3], b: [f32; 3], tolerance: f32) -> bool {
    (0..3).all(|c| (a[c] - b[c]).abs() < tolerance)
}

#[test]
fn an_emission_plane_seen_directly_is_its_strength_times_its_colour() {
    let mut rig = Rig::new(16, 16, [0.0; 3]);
    rig.add(&plane(50.0, 0.0), Surface::Emission { color: [1.0, 0.5, 0.25], strength: 3.0 }, translation(0.0, 0.0, 0.0));
    let image = rig.render(4);
    assert_eq!(image.len(), 256);
    for (index, pixel) in image.iter().enumerate() {
        assert!(near(*pixel, [3.0, 1.5, 0.75], 1e-5), "pixel {index}: {pixel:?}");
    }
}

#[test]
fn an_emission_plane_seen_from_behind_is_the_same() {
    let mut rig = Rig::new(8, 8, [0.0; 3]);
    // Turn the plane over: its front faces away from the camera.
    rig.add(&plane(50.0, 0.0), Surface::Emission { color: [1.0; 3], strength: 2.0 }, [[1.0, 0.0, 0.0, 0.0], [0.0, -1.0, 0.0, 0.0], [0.0, 0.0, -1.0, 0.0], [0.0, 0.0, 0.0, 1.0]]);
    assert!(rig.render(2).iter().all(|pixel| near(*pixel, [2.0; 3], 1e-5)));
}

#[test]
fn the_background_is_the_world_colour_where_nothing_is_hit() {
    let mut rig = Rig::new(8, 8, [0.25, 0.5, 1.0]);
    rig.scene.world.strength = 2.0;
    assert!(rig.render(2).iter().all(|pixel| near(*pixel, [0.5, 1.0, 2.0], 1e-6)));
}

#[test]
fn a_white_diffuse_object_in_a_uniform_world_is_the_world() {
    // The furnace test: an albedo of 1 sends all the light back, so every
    // path leaves the convex object after one bounce with a weight of one.
    let mut rig = Rig::new(24, 24, [0.4, 0.7, 0.2]);
    rig.add(&cube(0.8), Surface::Diffuse { color: [1.0; 3] }, translation(0.0, 0.0, 0.0));
    for pixel in rig.render(8) {
        assert!(near(pixel, [0.4, 0.7, 0.2], 1e-5), "{pixel:?}");
    }
}

#[test]
fn a_grey_diffuse_object_in_a_uniform_world_is_the_albedo_times_the_world() {
    let mut rig = Rig::new(24, 24, [0.5, 0.5, 0.5]);
    rig.add(&cube(0.8), Surface::Diffuse { color: [0.8, 0.4, 0.2] }, translation(0.0, 0.0, 0.0));
    let image = rig.render(8);
    // The middle pixels see the cube; the corners see the world.
    assert!(near(image[12 * 24 + 12], [0.4, 0.2, 0.1], 1e-5), "{:?}", image[12 * 24 + 12]);
    assert!(near(image[0], [0.5; 3], 1e-6));
}

#[test]
fn a_diffuse_plane_under_the_sky_is_its_albedo_times_the_sky() {
    // Nothing above the plane blocks the sky, so the radiance leaving it is
    // the albedo times the sky: the same from every angle.
    let mut rig = Rig::new(16, 16, [0.6; 3]);
    rig.add(&plane(100.0, 0.0), Surface::Diffuse { color: [0.5; 3] }, translation(0.0, 0.0, 0.0));
    assert!(rig.render(4).iter().all(|pixel| near(*pixel, [0.3; 3], 1e-5)));
}

#[test]
fn an_orthographic_camera_sees_the_same_plane_the_same() {
    let mut rig = Rig::new(8, 8, [0.6; 3]).orthographic();
    rig.add(&plane(100.0, 0.0), Surface::Diffuse { color: [0.5; 3] }, translation(0.0, 0.0, 0.0));
    assert!(rig.render(2).iter().all(|pixel| near(*pixel, [0.3; 3], 1e-5)));
}

#[test]
fn an_instance_is_drawn_where_it_is_placed() {
    let mut rig = Rig::new(32, 32, [0.0; 3]);
    let mesh = rig.add(&plane(0.5, 0.0), Surface::Emission { color: [1.0; 3], strength: 1.0 }, translation(-1.5, 0.0, 0.0));
    rig.instance(mesh, translation(1.5, 0.0, 0.0));
    let image = rig.render(4);
    let lit = |x: usize, y: usize| image[y * 32 + x][0] > 0.99;
    // The camera sees 3.6 across at the plane's distance, so the planes
    // at x = -1.5 and 1.5 fill the outer sixth of each side of the picture.
    let row = 16;
    let lit_columns: Vec<usize> = (0..32).filter(|x| lit(*x, row)).collect();
    assert!(!lit_columns.is_empty() && lit_columns.iter().any(|x| *x < 16) && lit_columns.iter().any(|x| *x >= 16), "{lit_columns:?}");
    assert!(!lit(16, row) && !lit(15, row));
    // The same number on each side of the middle of the picture, give or
    // take a pixel at an edge.
    let left = lit_columns.iter().filter(|x| **x < 16).count();
    assert!(left.abs_diff(lit_columns.len() - left) <= 1, "{lit_columns:?}");
}

#[test]
fn a_render_is_the_same_each_time() {
    let draw = || {
        let mut rig = Rig::new(12, 12, [0.3; 3]);
        rig.add(&cube(1.0), Surface::Diffuse { color: [0.7; 3] }, translation(0.0, 0.0, 0.0));
        rig.render(5)
    };
    assert_eq!(draw(), draw());
}
#[test]
fn a_diffuse_floor_under_an_emitting_ceiling_is_its_albedo_times_the_ceiling() {
    // A ceiling above the camera, its front turned away from the floor: an
    // emission is seen from either side, so a bounce off the floor finds it.
    let mut rig = Rig::new(8, 8, [0.0; 3]);
    rig.add(&plane(1000.0, 6.0), Surface::Emission { color: [1.0; 3], strength: 2.0 }, translation(0.0, 0.0, 0.0));
    rig.add(&plane(1000.0, 0.0), Surface::Diffuse { color: [0.5; 3] }, translation(0.0, 0.0, 0.0));
    let image = rig.render(64);
    let mean = image.iter().map(|pixel| pixel[0]).sum::<f32>() / image.len() as f32;
    assert!((mean - 1.0).abs() < 0.05, "{mean} {:?}", &image[..4]);
}