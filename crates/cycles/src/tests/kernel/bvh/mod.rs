// SPDX-License-Identifier: Apache-2.0
//! Tests of `src/kernel/bvh`, one file for each file there. This file
//! tests `mod.rs` and holds the scenes the others send rays into.

mod data;
mod node;
mod traversal;
mod util;

use crate::kernel::bvh::*;
use crate::kernel::{Intersection, KernelGlobal, Ray, flag};
use crate::scene::{Mesh, Object};
use crate::tests::bvh::strip;
pub(crate) use crate::tests::scene::geometry::Rig;
pub(crate) use crate::tests::scene::mesh::square;
use crate::util::Transform;

/// A scene of these meshes, each placed once where it is, written to a
/// device scene.
pub(crate) fn rig_of(mesh: Vec<Mesh>) -> Rig {
    let mut rig = Rig::new(mesh);
    rig.update();
    rig
}

/// A scene of one square placed at each of `place`: every object after
/// the first is an instance, so the square keeps a tree of its own.
pub(crate) fn instanced(place: &[Transform]) -> Rig {
    let mut rig = Rig::new(vec![square()]);
    rig.object = place.iter().map(|_| Object::new(0)).collect();
    for (object, place) in rig.object.iter_mut().zip(place) {
        object.set_transform(*place);
    }
    rig.update();
    rig
}

/// A ray from `p` along z, as long as `tmax`.
pub(crate) fn along_z(p: [f32; 3], tmax: f32) -> Ray {
    Ray { p, d: [0.0, 0.0, 1.0], tmax, ..Ray::default() }
}

/// Every kind of ray.
pub(crate) fn all() -> u32 {
    flag("PATH_RAY_VISIBILITY_ALL")
}

/// What `ray` hits in `rig`, if anything.
pub(crate) fn hit(rig: &Rig, ray: &Ray, visibility: u32) -> Option<Intersection> {
    let kg = KernelGlobal::new(&rig.dscene);
    let mut isect = Intersection::default();
    scene_intersect(&kg, ray, visibility, &mut isect).then_some(isect)
}

#[test]
fn a_ray_that_is_no_ray_hits_nothing() {
    let rig = rig_of(vec![square()]);
    let good = along_z([0.25, 0.5, -1.0], 10.0);
    assert!(hit(&rig, &good, all()).is_some());
    assert!(hit(&rig, &Ray { d: [0.0; 3], ..good }, all()).is_none());
    assert!(hit(&rig, &Ray { p: [f32::NAN, 0.5, -1.0], ..good }, all()).is_none());
    assert!(hit(&rig, &Ray { d: [f32::INFINITY, 0.0, 1.0], ..good }, all()).is_none());
    assert!(hit(&rig, &Ray { tmin: f32::MAX, ..good }, all()).is_none());
}

#[test]
fn a_shadow_ray_asks_only_whether_anything_is_in_the_way() {
    let rig = rig_of(vec![strip(8)]);
    let kg = KernelGlobal::new(&rig.dscene);
    let shadow = flag("PATH_RAY_VISIBILITY_SHADOW_OPAQUE");
    assert!(scene_intersect_shadow(&kg, &along_z([4.5, 0.5, -1.0], 10.0), shadow));
    // Between two squares of the strip, and short of it.
    assert!(!scene_intersect_shadow(&kg, &along_z([3.5, 0.5, -1.0], 10.0), shadow));
    assert!(!scene_intersect_shadow(&kg, &along_z([4.5, 0.5, -1.0], 0.5), shadow));
}
