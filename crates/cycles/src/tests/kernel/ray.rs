// SPDX-License-Identifier: Apache-2.0
use crate::kernel::*;

#[test]
fn a_new_ray_starts_from_nothing() {
    let ray = Ray::default();
    let none = |name: &str| constant(name).expect("in data/constant.tsv") as i32;
    assert_eq!((ray.own.prim, ray.own.light_prim), (none("PRIM_NONE"), none("PRIM_NONE")));
    assert_eq!((ray.own.object, ray.own.light_object), (none("OBJECT_NONE"), none("OBJECT_NONE")));
    assert_eq!((ray.p, ray.d, ray.tmin, ray.tmax, ray.time, ray.dp, ray.dd), ([0.0; 3], [0.0; 3], 0.0, 0.0, 0.0, 0.0, 0.0));
}
