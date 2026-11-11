// SPDX-License-Identifier: Apache-2.0
use crate::device::*;

#[test]
fn a_denoiser_is_named_as_cycles_names_it() {
    assert_eq!(DenoiserType::Optix.label(), "OptiX");
    assert_eq!(DenoiserType::OpenImageDenoise.label(), "OpenImageDenoise");
    assert_eq!(DenoiserType::None.label(), "UNKNOWN");
}

#[test]
fn the_denoisers_keep_cycles_numbers() {
    assert_eq!([DenoiserType::None as i32, DenoiserType::Optix as i32, DenoiserType::OpenImageDenoise as i32], [0, 2, 4]);
    assert_eq!([DenoiserPrefilter::None as i32, DenoiserPrefilter::Fast as i32, DenoiserPrefilter::Accurate as i32], [1, 2, 3]);
    assert_eq!([DenoiserQuality::High as i32, DenoiserQuality::Balanced as i32, DenoiserQuality::Fast as i32], [1, 2, 3]);
}

#[test]
fn the_passes_of_a_mask_are_named_in_the_order_of_their_bits() {
    let mask = DenoiserPass::MOTION | DenoiserPass::ALBEDO | DenoiserPass::ROUGHNESS | DenoiserPass::SPECULAR_ALBEDO;
    assert_eq!(DenoiserPass::label_list(mask), ["Albedo", "Specular Albedo", "Roughness", "Motion"]);
    assert!(DenoiserPass::label_list(DenoiserPass::NONE).is_empty());
}

#[test]
fn the_backward_motion_pass_has_no_label() {
    assert_eq!(DenoiserPass::label_list(DenoiserPass::BACKWARD_MOTION | DenoiserPass::DEPTH), ["Depth"]);
}

#[test]
fn the_settings_start_as_cycles_starts_them() {
    let param = DenoiseParam::default();
    assert!(!param.enabled && param.use_gpu && !param.temporally_stable);
    assert_eq!(param.denoiser, DenoiserType::OpenImageDenoise);
    assert_eq!(param.pass_mask, DenoiserPass::ALBEDO | DenoiserPass::NORMAL);
    assert_eq!((param.start_sample, param.upscale_factor), (0, 1.0));
    assert_eq!((param.prefilter, param.quality), (DenoiserPrefilter::Fast, DenoiserQuality::High));
}
