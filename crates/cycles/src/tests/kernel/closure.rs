// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Tests of `kernel/closure/`: allocation, the diffuse BSDF and the
//! emission closures.
use crate::kernel::closure::*;
use crate::kernel::constant;
use crate::kernel::bit::*;
use crate::kernel::{ShaderClosure, ShaderData};
use crate::util::vector::{dot, len};

fn diffuse_id() -> i32 {
    constant("CLOSURE_BSDF_DIFFUSE_ID").unwrap() as i32
}

#[test]
fn a_bsdf_gets_its_weight_and_a_sample_weight_that_is_its_average() {
    let mut sd = ShaderData::default();
    let closure = bsdf_alloc(&mut sd, [0.3, 0.6, 0.9]).expect("room for a closure");
    assert_eq!(closure.weight, [0.3, 0.6, 0.9]);
    assert!((closure.sample_weight - 0.6).abs() < 1e-6);
    assert_eq!((sd.num_closure, sd.num_closure_left), (1, MAX_CLOSURE - 1));
}

#[test]
fn a_weight_under_the_cutoff_or_negative_makes_no_closure() {
    let mut sd = ShaderData::default();
    assert!(bsdf_alloc(&mut sd, [0.0; 3]).is_none());
    assert!(bsdf_alloc(&mut sd, [1e-7; 3]).is_none());
    assert!(bsdf_alloc(&mut sd, [-1.0, -1.0, -1.0]).is_none());
    assert!(bsdf_alloc(&mut sd, [f32::NAN; 3]).is_none());
    assert_eq!(sd.num_closure, 0);
    // A negative part is dropped, and the rest kept.
    let mut weight = [-1.0, 0.5, 0.25];
    assert!((closure_sample_weight(&mut weight) - 0.25).abs() < 1e-6 && weight == [0.0, 0.5, 0.25]);
}

#[test]
fn closures_fill_the_array_and_stop_when_it_is_full_or_not_wanted() {
    let mut sd = ShaderData::default();
    for _ in 0..MAX_CLOSURE {
        assert!(closure_alloc(&mut sd, 1, [1.0; 3]).is_some());
    }
    assert!(closure_alloc(&mut sd, 1, [1.0; 3]).is_none());
    // A shader run only for its emission has no room.
    let mut emission_only = ShaderData { num_closure_left: 0, ..ShaderData::default() };
    bsdf_diffuse_setup(&mut emission_only, [0.0, 0.0, 1.0], [1.0; 3]);
    assert_eq!((emission_only.num_closure, emission_only.flag), (0, 0));
}

#[test]
fn a_diffuse_closure_sets_the_flags_a_path_looks_for() {
    let mut sd = ShaderData::default();
    bsdf_diffuse_setup(&mut sd, [0.0, 0.0, 1.0], [0.8; 3]);
    assert_eq!(sd.flag, SD_BSDF | SD_BSDF_HAS_EVAL);
    assert_eq!((sd.closure[0].kind, sd.closure[0].n), (diffuse_id(), [0.0, 0.0, 1.0]));
    assert!(closure_is_bsdf(sd.closure[0].kind) && !closure_is_bsdf(0));
}

#[test]
fn a_diffuse_value_and_pdf_are_the_cosine_over_pi() {
    let mut sd = ShaderData::default();
    bsdf_diffuse_setup(&mut sd, [0.0, 0.0, 1.0], [1.0; 3]);
    let sc = &sd.closure[0];
    let wo = [0.6, 0.0, 0.8];
    let (value, pdf) = bsdf_diffuse_eval(sc, wo);
    assert!((pdf - 0.8 / std::f32::consts::PI).abs() < 1e-6 && value == [pdf; 3]);
    // Under the surface: nothing.
    assert_eq!(bsdf_diffuse_eval(sc, [0.0, 0.0, -1.0]), ([0.0; 3], 0.0));
}

#[test]
fn a_diffuse_sample_agrees_with_its_evaluation_and_stays_above_the_surface() {
    let mut sd = ShaderData::default();
    bsdf_diffuse_setup(&mut sd, [0.0, 0.0, 1.0], [1.0; 3]);
    let sc = &sd.closure[0];
    for i in 0..16 {
        for j in 0..16 {
            let (eval, wo, pdf, label) = bsdf_diffuse_sample(sc, [0.0, 0.0, 1.0], [(i as f32 + 0.5) / 16.0, (j as f32 + 0.5) / 16.0]);
            assert_eq!(label, LABEL_REFLECT | LABEL_DIFFUSE);
            assert!((len(wo) - 1.0).abs() < 1e-5);
            if pdf > 0.0 {
                let (value, check) = bsdf_diffuse_eval(sc, wo);
                assert!((pdf - check).abs() < 1e-5 && (eval[0] - value[0]).abs() < 1e-5);
                assert!(dot(wo, [0.0, 0.0, 1.0]) > 0.0);
            }
        }
    }
    // A direction under the geometric normal has no value.
    let (eval, _, pdf, _) = bsdf_diffuse_sample(sc, [0.0, 0.0, -1.0], [0.3, 0.3]);
    assert_eq!((eval, pdf), ([0.0; 3], 0.0));
}

#[test]
fn bsdf_sample_gives_nothing_for_a_closure_that_is_not_ported() {
    let sd = ShaderData::default();
    let other = ShaderClosure { kind: constant("CLOSURE_BSDF_MICROFACET_GGX_ID").unwrap() as i32, ..ShaderClosure::default() };
    assert_eq!(bsdf_sample(&sd, &other, [0.5; 3]).pdf, 0.0);
}

#[test]
fn emissions_add_up_and_set_the_emission_flag() {
    let mut sd = ShaderData::default();
    emission_setup(&mut sd, [1.0, 2.0, 3.0]);
    assert!(sd.flag & SD_EMISSION != 0 && sd.closure_emission_background == [1.0, 2.0, 3.0]);
    background_setup(&mut sd, [1.0; 3]);
    assert_eq!(sd.closure_emission_background, [2.0, 3.0, 4.0]);
    assert_eq!(sd.num_closure, 0);
}

#[test]
fn an_emission_is_seen_from_either_side_but_not_along_the_surface() {
    let n = [0.0, 0.0, 1.0];
    assert_eq!(emissive_simple_eval(n, [0.0, 0.6, 0.8]), [1.0; 3]);
    assert_eq!(emissive_simple_eval(n, [0.0, 0.6, -0.8]), [1.0; 3]);
    assert_eq!(emissive_pdf(n, [1.0, 0.0, 0.0]), 0.0);
}
