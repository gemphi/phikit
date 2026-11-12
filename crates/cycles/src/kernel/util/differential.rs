// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/util/differential.h` (lines 113-126)
//! and `kernel/types.h` (`differential3`): how far a ray is from the rays of
//! the pixels beside it, which says how wide a texture lookup should be.
//! The rest of that file comes with the shading that uses it.

use crate::util::vector::len;

/// `differential3`: the change of a point or a direction one pixel across
/// and one pixel down.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Differential3 {
    pub dx: [f32; 3],
    pub dy: [f32; 3],
}

/// `differential_zero_compact`.
pub fn differential_zero_compact() -> f32 {
    0.0
}

/// `differential_make_compact`: the two changes as one number, the mean of
/// their lengths.
pub fn differential_make_compact(change: &Differential3) -> f32 {
    0.5 * (len(change.dx) + len(change.dy))
}

#[cfg(test)]
mod test;
