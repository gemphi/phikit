// SPDX-FileCopyrightText: 2011-2026 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/util/types_normal.h` (`packed_normal`): a
//! unit vector in 32 bits, folded onto an octahedron and stored as two
//! 16-bit numbers.

use super::vector::normalize;

/// `packed_normal`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct PackedNormal(pub u32);

/// The largest 16-bit number.
const MU: u32 = (1 << 16) - 1;

impl PackedNormal {
    /// `packed_normal(float3)`.
    #[allow(clippy::manual_clamp, reason = "clamp keeps a not-a-number; Cycles makes it zero")]
    pub fn encode(normal: [f32; 3]) -> PackedNormal {
        let half = MU as f32 / 2.0;
        let inv_l1 = 1.0 / (normal[0].abs() + normal[1].abs() + normal[2].abs() + 1e-6);
        let (x, y) = (normal[0] * inv_l1, normal[1] * inv_l1);
        // Under the horizon the octahedron's lower half folds outward.
        let (x, y) = match normal[2] >= 0.0 {
            true => (x, y),
            false => ((1.0 - y.abs()) * 1.0f32.copysign(x), (1.0 - x.abs()) * 1.0f32.copysign(y)),
        };
        // `max` first, so a part that is not a number becomes zero, as
        // Cycles' `clamp` makes it.
        let pack = |part: f32| (part * half + (half + 0.5)).max(0.0).min(MU as f32) as u32;
        PackedNormal(pack(x) | (pack(y) << 16))
    }

    /// `decode`.
    pub fn decode(self) -> [f32; 3] {
        let inv_half = 2.0 / MU as f32;
        let x = (self.0 & MU) as f32 * inv_half - 1.0;
        let y = ((self.0 >> 16) & MU) as f32 * inv_half - 1.0;
        let z = 1.0 - x.abs() - y.abs();
        let fold = (-z).max(0.0);
        normalize([x + fold.copysign(-x), y + fold.copysign(-y), z])
    }
}
