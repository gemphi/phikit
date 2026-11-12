// SPDX-FileCopyrightText: 2018-2023 Blender Authors
// SPDX-License-Identifier: Apache-2.0
//! Ported from `tests/python/modules/render_report.py:125-205`
//! (`diff_output`). Blender does not compare in Python: it runs
//!
//! ```text
//! oiiotool reference.png render.png --fail T --failpercent P --diff
//! ```
//!
//! (lines 138-144) and takes a non-zero exit as a failed test (line 148).
//! `--diff` is OpenImageIO's `ImageBufAlgo::compare` with `failthresh = T`:
//! a pixel *fails* when any channel differs by more than `T` (strictly), and
//! oiiotool's `--failpercent P` fails the run when the failed pixels are more
//! than `P` per cent of all pixels (strictly). The hard-fail limit is
//! infinity, so it never decides. That library is outside this repository;
//! the rule is reproduced from its documented behaviour and each test of it
//! says what it pins.
//!
//! The diff images are lines 183-205: `|reference - render| * 16` on R,G,B
//! (`--ch R,G,B ... --sub --abs --mulc 16`), and the same on A.
use super::{Allowance, Image, TruthError};

/// What `compare` found (OpenImageIO's `CompareResults`).
#[derive(Debug, Clone, PartialEq)]
pub struct Comparison {
    /// `nfail`: pixels with a channel over the threshold.
    pub failed_pixel: usize,
    pub pixel: usize,
    /// `maxerror`.
    pub max_error: f32,
    /// `meanerror`.
    pub mean_error: f64,
    /// `failed_pixel` over `pixel`, in per cent.
    pub failed_percent: f64,
    /// oiiotool's exit status was not zero: more pixels failed than the
    /// allowance gives.
    pub failed: bool,
}

/// `oiiotool a b --fail threshold --failpercent percent --diff`.
pub fn compare(reference: &Image, render: &Image, allowance: Allowance) -> Result<Comparison, TruthError> {
    if (reference.width, reference.height) != (render.width, render.height) {
        return Err(TruthError::Size { reference: (reference.width, reference.height), render: (render.width, render.height) });
    }
    let pixel = reference.width * reference.height;
    let channel = reference.channel_count.max(render.channel_count);
    let (mut failed_pixel, mut max_error, mut total) = (0usize, 0.0f32, 0.0f64);
    for index in 0..pixel {
        let mut failed = false;
        for c in 0..channel {
            let (a, b) = (reference.channel(index, c), render.channel(index, c));
            // Equal values (and equal infinities) are no error; a NaN or an
            // infinity against anything else is a failure.
            let error = if a == b { 0.0 } else if a.is_finite() && b.is_finite() { (a - b).abs() } else { f32::INFINITY };
            total += f64::from(error.min(f32::MAX));
            max_error = max_error.max(error);
            failed |= error > allowance.threshold;
        }
        failed_pixel += usize::from(failed);
    }
    let failed_percent = if pixel == 0 { 0.0 } else { failed_pixel as f64 * 100.0 / pixel as f64 };
    // `nfail > failpercent / 100 * npixels`
    let failed = failed_pixel as f64 > allowance.percent / 100.0 * pixel as f64;
    let mean_error = if pixel * channel == 0 { 0.0 } else { total / (pixel * channel) as f64 };
    Ok(Comparison { failed_pixel, pixel, max_error, mean_error, failed_percent, failed })
}

/// `|reference - render| * 16` on `channel` (lines 183-205): the picture a
/// report shows next to the two renders. Clamped to 0..1 when written.
pub fn diff_image(reference: &Image, render: &Image, channel: &[usize]) -> Result<Image, TruthError> {
    if (reference.width, reference.height) != (render.width, render.height) {
        return Err(TruthError::Size { reference: (reference.width, reference.height), render: (render.width, render.height) });
    }
    let mut diff = Image::new(reference.width, reference.height, channel.len());
    for index in 0..reference.width * reference.height {
        for (at, &c) in channel.iter().enumerate() {
            diff.pixel[index * channel.len() + at] = (reference.channel(index, c) - render.channel(index, c)).abs() * 16.0;
        }
    }
    Ok(diff)
}

#[cfg(test)]
mod test;
