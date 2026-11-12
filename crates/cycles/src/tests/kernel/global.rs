// SPDX-License-Identifier: Apache-2.0
//! Tests of `kernel/global.rs`: the scene as a kernel reads it, from the
//! words the managers wrote.
use super::rig::{Rig, cube, plane, translation};
use crate::kernel::camera::CameraType;
use crate::kernel::util::lookup_table_read;
use crate::kernel::*;
use crate::scene::{Camera, DeviceScene, LookupTableSet, MotionType, Surface};
#[test]
fn the_globals_take_what_a_ray_needs_out_of_the_constant_data() {
    let (mut camera, mut dscene, mut lookup) = (Camera::new(), DeviceScene::new(), LookupTableSet::new());
    camera.device_update(&mut dscene, &mut lookup, MotionType::None);
    dscene.data.set_float2("integrator", "pixel_jitter", [0.25, 0.75]);
    dscene.data.set_int("tables", "filter_table_offset", 256);
    let kg = KernelGlobal::of(&dscene);
    assert_eq!((kg.cam.kind, kg.integrator.pixel_jitter, kg.filter_table_offset), (Some(CameraType::Perspective), [0.25, 0.75], 256));
    let size = |name: &str| constant(name).expect("in data/constant.tsv") as usize;
    assert_eq!((kg.filter_table_size, kg.shutter_table_size), (size("FILTER_TABLE_SIZE"), size("SHUTTER_TABLE_SIZE")));
    assert_eq!(*kg.scene, dscene);
}
#[test]
fn a_table_that_was_never_added_is_read_from_the_start() {
    let mut dscene = DeviceScene::new();
    dscene.data.set_int("tables", "filter_table_offset", -1);
    assert_eq!(KernelGlobal::of(&dscene).filter_table_offset, 0);
}
#[test]
fn the_camera_the_film_and_the_integrator_are_read_from_the_constant_data() {
    let mut rig = Rig::new(64, 32, [0.25, 0.5, 1.0]);
    let kg = rig.kernel();
    assert_eq!(kg.cam.kind, Some(CameraType::Perspective));
    assert_eq!((kg.cam.width, kg.cam.height), (64.0, 32.0));
    assert!((kg.cam.cliplength - 99.9).abs() < 1e-3 && (kg.cam.nearclip - 0.1).abs() < 1e-6);
    // The camera is at (0, 0, 5).
    assert_eq!([kg.cam.cameratoworld.x[3], kg.cam.cameratoworld.y[3], kg.cam.cameratoworld.z[3]], [0.0, 0.0, 5.0]);
    assert_eq!((kg.film.pass_stride, kg.film.pass_combined), (4, 0));
    assert_eq!(kg.film.pass_sample_count, constant("PASS_UNUSED").unwrap() as i32);
    assert!(kg.film.exposure == 1.0);
    // `max_bounce` is one over what the integrator was told (1 + 7).
    assert_eq!((kg.integrator.min_bounce, kg.integrator.max_bounce), (1, 8));
    assert_eq!(kg.integrator.sample_clamp_direct, f32::MAX);
    assert_eq!(kg.integrator.pixel_jitter[0], f32::MAX);
}
#[test]
fn the_world_is_a_shader_after_the_scene_s_and_visible_to_every_ray() {
    let mut rig = Rig::new(8, 8, [0.25, 0.5, 1.0]);
    rig.add(&plane(1.0, 0.0), Surface::Diffuse { color: [0.5; 3] }, translation(0.0, 0.0, 0.0));
    let kg = rig.kernel();
    // One scene shader, then the world's, then the default surface.
    assert_eq!(kg.shader.len(), 3);
    assert_eq!(kg.background.surface_shader as u32 & crate::kernel::bit::SHADER_MASK, 1);
    assert_eq!(kg.background.surface_shader as u32 & crate::kernel::bit::SHADER_EXCLUDE_ANY, 0);
    assert!(!kg.background.transparent);
    assert_eq!(kg.shader[1].constant_emission, [0.25, 0.5, 1.0]);
    assert_eq!(kg.shader[0].constant_emission, [0.0; 3]);
}
#[test]
fn an_object_carries_its_transform_its_inverse_and_where_its_arrays_start() {
    let mut rig = Rig::new(8, 8, [0.1; 3]);
    let mesh = rig.add(&cube(1.0), Surface::Diffuse { color: [0.5; 3] }, translation(2.0, 0.0, 0.0));
    rig.instance(mesh, translation(-3.0, 0.0, 0.0));
    let kg = rig.kernel();
    assert_eq!(kg.object.len(), 2);
    assert_eq!(kg.object[0].tfm.x[3], 2.0);
    assert_eq!(kg.object[1].tfm.x[3], -3.0);
    assert_eq!(kg.object[1].itfm.x[3], 3.0);
    // The mesh is shared: both objects read the same positions, where the
    // record of `data/record.tsv` puts them.
    assert_eq!(kg.object[0].position_offset, kg.object[1].position_offset);
    assert_eq!(kg.object[1].position_offset, kg.scene.object.int(1, "position_offset"));
    assert_eq!(kg.scene.tri_vindex.len(), 12);
}
#[test]
fn the_filter_table_is_read_with_linear_interpolation() {
    let mut rig = Rig::new(8, 8, [0.1; 3]);
    let kg = rig.kernel();
    let at = |x: f32| lookup_table_read(&kg.scene.lookup_table, x, kg.filter_table_offset, kg.filter_table_size);
    // The filter table is a symmetric table over 0..1 centred on 0.5.
    assert!((at(0.5) - 0.5).abs() < 0.01);
    assert!(at(0.0) <= at(0.25) && at(0.25) <= at(0.5) && at(0.5) <= at(0.75) && at(0.75) <= at(0.99) && at(0.0) < at(0.99));
    // The last entry of the filter table is left at zero, as Cycles leaves it.
    assert_eq!(at(1.0), 0.0);
    assert_eq!(at(-1.0), at(0.0));
    assert_eq!(at(2.0), at(1.0));
}
