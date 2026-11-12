// SPDX-License-Identifier: Apache-2.0
use crate::kernel::camera::CameraType;
use crate::kernel::*;
use crate::scene::{Camera, DeviceScene, LookupTableSet, MotionType};

#[test]
fn the_globals_take_what_a_ray_needs_out_of_the_constant_data() {
    let (mut camera, mut dscene, mut lookup) = (Camera::new(), DeviceScene::new(), LookupTableSet::new());
    camera.device_update(&mut dscene, &mut lookup, MotionType::None);
    dscene.data.set_float2("integrator", "pixel_jitter", [0.25, 0.75]);
    dscene.data.set_int("tables", "filter_table_offset", 256);
    let kg = KernelGlobal::new(&dscene);
    assert_eq!((kg.cam.kind, kg.pixel_jitter, kg.filter_table_offset), (Some(CameraType::Perspective), [0.25, 0.75], 256));
    let size = |name: &str| constant(name).expect("in data/constant.tsv") as usize;
    assert_eq!((kg.filter_table_size, kg.shutter_table_size), (size("FILTER_TABLE_SIZE"), size("SHUTTER_TABLE_SIZE")));
    assert!(std::ptr::eq(kg.scene, &dscene));
}

#[test]
fn a_table_that_was_never_added_is_read_from_the_start() {
    let mut dscene = DeviceScene::new();
    dscene.data.set_int("tables", "filter_table_offset", -1);
    assert_eq!(KernelGlobal::new(&dscene).filter_table_offset, 0);
}
