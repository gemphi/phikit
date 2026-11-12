// SPDX-License-Identifier: Apache-2.0
//! Tests of `kernel/integrator/state.rs` and `layout.rs`: the state of a
//! path is laid out as `state_template.h` has it.
use std::mem::{offset_of, size_of};

use crate::kernel::integrator::*;
use crate::kernel::types::NONE;
use crate::kernel::{Intersection, Ray};

/// Each member of a struct, at the word the generated layout puts it.
macro_rules! members {
    ($layout:expr, $type:ty, $($name:ident),* $(,)?) => {
        $(
            let member = $layout.member(stringify!($name)).unwrap_or_else(|| panic!("data/state.tsv has no {}", stringify!($name)));
            assert_eq!(offset_of!($type, $name), member.word * 4, "{}", stringify!($name));
        )*
    };
}

#[test]
fn the_path_struct_is_the_members_every_path_has_in_the_templates_order() {
    let layout = StateLayout::of("path").expect("state_template.h has path");
    members!(
        layout, PathState, render_pixel_index, sample, bounce, transparent_bounce, diffuse_bounce, glossy_bounce, transmission_bounce, volume_bounce, volume_bounds_bounce, queued_kernel, rng_pixel, rng_offset, visibility, flag, mnee, optical_depth, mis_ray_pdf, mis_origin_n, min_ray_pdf, continuation_probability, throughput, shader_sort_key
    );
    assert_eq!(size_of::<PathState>(), layout.word_count() * 4);
    assert_eq!(layout.member.len(), 22);
}

#[test]
fn the_ray_and_the_hit_are_laid_out_as_their_templates_are() {
    let ray = StateLayout::of("ray").expect("state_template.h has ray");
    // Cycles names the position and direction `P` and `D`.
    for (name, offset) in [("P", offset_of!(RayState, p)), ("dP", offset_of!(RayState, dp)), ("D", offset_of!(RayState, d)), ("dD", offset_of!(RayState, dd)), ("tmin", offset_of!(RayState, tmin)), ("tmax", offset_of!(RayState, tmax)), ("time", offset_of!(RayState, time))] {
        assert_eq!(offset, ray.member(name).unwrap_or_else(|| panic!("no {name}")).word * 4, "{name}");
    }
    assert_eq!(size_of::<RayState>(), ray.word_count() * 4);
    let isect = StateLayout::of("isect").expect("state_template.h has isect");
    members!(isect, IsectState, t, u, v, prim, object);
    assert_eq!(offset_of!(IsectState, kind), isect.member("type").unwrap().word * 4);
    assert_eq!(size_of::<IsectState>(), isect.word_count() * 4);
}

#[test]
fn a_member_takes_a_word_for_each_32_bits_of_its_type() {
    let path = StateLayout::of("path").unwrap();
    let word = |name: &str| path.member(name).unwrap().word_count;
    assert_eq!((word("bounce"), word("visibility"), word("throughput"), word("mis_origin_n")), (1, 1, 3, 3));
    // The members of a feature a gate has not are not in the layout.
    assert!(path.member("portal_bounce").is_none() && path.member("unguided_throughput").is_none());
    assert!(StateLayout::of("no_such_struct").is_none());
}

#[test]
fn a_state_is_plain_bytes_a_device_can_hold() {
    let state = IntegratorState::new();
    let bytes: &[u8] = bytemuck::bytes_of(&state);
    assert_eq!(bytes.len(), size_of::<IntegratorState>());
    assert!(bytes.iter().all(|byte| *byte == 0));
    assert_eq!(state.path.queued_kernel, 0);
}

#[test]
fn a_ray_and_a_hit_go_in_and_out_of_the_state() {
    let mut state = IntegratorState::new();
    let ray = Ray { p: [1.0, 2.0, 3.0], d: [0.0, 0.0, -1.0], tmin: 0.5, tmax: 9.0, time: 0.25, dp: 0.1, dd: 0.2, ..Ray::default() };
    state.write_ray(&ray);
    assert_eq!(state.read_ray(), ray);
    let isect = Intersection { t: 2.0, u: 0.25, v: 0.5, prim: 4, object: 1, kind: 1 };
    state.write_isect(&isect);
    assert_eq!(state.read_isect(), isect);
    state.clear_isect();
    assert_eq!((state.isect.prim, state.isect.object), (NONE, NONE));
}
