// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/device/denoise.h` and `denoise.cpp`
//! (`DenoiserType`, `DenoiserPass`, `DenoiserPrefilter`, `DenoiserQuality`,
//! `DenoiseParams`). The labels are `data/denoise.tsv`.

use crate::util::row_list;

/// `DenoiserType`, by Cycles' own number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DenoiserType {
    /// `DENOISER_NONE`.
    None = 0,
    /// `DENOISER_OPTIX`.
    Optix = 2,
    /// `DENOISER_OPENIMAGEDENOISE`.
    #[default]
    OpenImageDenoise = 4,
}

impl DenoiserType {
    /// The denoiser of Cycles' number; none for a number it does not use.
    pub fn of(number: i32) -> DenoiserType {
        match number {
            2 => DenoiserType::Optix,
            4 => DenoiserType::OpenImageDenoise,
            _ => DenoiserType::None,
        }
    }

    /// `denoiserTypeToHumanReadable`.
    pub fn label(self) -> &'static str {
        label("type", self as i32)
    }
}

/// `DenoiserPass`: one guiding pass, a bit of a `DenoiserPassMask`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DenoiserPass;

impl DenoiserPass {
    pub const NONE: i32 = 0;
    pub const ALBEDO: i32 = 1 << 0;
    pub const SPECULAR_ALBEDO: i32 = 1 << 1;
    pub const NORMAL: i32 = 1 << 2;
    pub const ROUGHNESS: i32 = 1 << 3;
    pub const DEPTH: i32 = 1 << 4;
    pub const MOTION: i32 = 1 << 5;
    pub const BACKWARD_MOTION: i32 = 1 << 6;

    /// The labels of the passes in `mask`, in the order of their bits. A
    /// pass with no label is left out.
    pub fn label_list(mask: i32) -> Vec<&'static str> {
        let labelled = row_list(TABLE).filter(|row| row.first() == Some(&"pass"));
        let set = labelled.filter(|row| row.get(2).and_then(|value| value.parse::<i32>().ok()).is_some_and(|bit| mask & bit != 0));
        set.filter_map(|row| row.get(3).copied()).filter(|label| !label.is_empty()).collect()
    }
}

/// `DenoiserPrefilter`: how noisy guiding passes are treated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DenoiserPrefilter {
    /// `DENOISER_PREFILTER_NONE`: the guiding passes are taken as noise-free.
    None = 1,
    /// `DENOISER_PREFILTER_FAST`: colour and guiding passes together.
    #[default]
    Fast = 2,
    /// `DENOISER_PREFILTER_ACCURATE`: guiding passes first, then colour.
    Accurate = 3,
}

impl DenoiserPrefilter {
    /// The prefilter of Cycles' number; the default for any other.
    pub fn of(number: i32) -> DenoiserPrefilter {
        match number {
            1 => DenoiserPrefilter::None,
            3 => DenoiserPrefilter::Accurate,
            _ => DenoiserPrefilter::Fast,
        }
    }
}

/// `DenoiserQuality`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DenoiserQuality {
    #[default]
    High = 1,
    Balanced = 2,
    Fast = 3,
}

impl DenoiserQuality {
    /// The quality of Cycles' number; the default for any other.
    pub fn of(number: i32) -> DenoiserQuality {
        match number {
            2 => DenoiserQuality::Balanced,
            3 => DenoiserQuality::Fast,
            _ => DenoiserQuality::High,
        }
    }
}

/// `DenoiseParams`: the denoiser's settings.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DenoiseParam {
    /// `use`: denoise the image.
    pub enabled: bool,
    /// `type`.
    pub denoiser: DenoiserType,
    /// The viewport sample denoising starts at.
    pub start_sample: i32,
    /// `passes`: a mask of [`DenoiserPass`] bits.
    pub pass_mask: i32,
    pub temporally_stable: bool,
    /// Let OpenImageDenoise use a GPU when it can.
    pub use_gpu: bool,
    pub prefilter: DenoiserPrefilter,
    pub quality: DenoiserQuality,
    pub upscale_factor: f32,
}

impl Default for DenoiseParam {
    fn default() -> Self {
        Self {
            enabled: false,
            denoiser: DenoiserType::OpenImageDenoise,
            start_sample: 0,
            pass_mask: DenoiserPass::ALBEDO | DenoiserPass::NORMAL,
            temporally_stable: false,
            use_gpu: true,
            prefilter: DenoiserPrefilter::Fast,
            quality: DenoiserQuality::High,
            upscale_factor: 1.0,
        }
    }
}

const TABLE: &str = include_str!("../../data/denoise.tsv");

/// The label of the `kind` row whose number is `value`.
fn label(kind: &str, value: i32) -> &'static str {
    let mut found = row_list(TABLE).filter(|row| row.first() == Some(&kind) && row.get(2).and_then(|cell| cell.parse::<i32>().ok()) == Some(value));
    found.next().and_then(|row| row.get(3).copied()).unwrap_or("UNKNOWN")
}
