// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/integrator.h` and `integrator.cpp`
//! (`HaltonSequence`, `halton`): the pixel jitter of each frame, from a
//! Halton sequence worked out in whole numbers
//! (<https://doi.org/10.1016/0010-4655(91)90064-R>).

/// `HaltonSequence`: where the sequences of base 2 and base 3 are.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HaltonSequence {
    pub a2: i32,
    pub b2: i32,
    pub a3: i32,
    pub b3: i32,
}

impl Default for HaltonSequence {
    fn default() -> Self {
        Self { a2: 0, b2: 1, a3: 0, b3: 1 }
    }
}

impl HaltonSequence {
    /// `reset`: back to the start.
    pub fn reset(&mut self) {
        *self = HaltonSequence::default();
    }

    /// `next`: the next jitter, each part from -0.5 to 0.5.
    pub fn next(&mut self) -> [f32; 2] {
        [halton(&mut self.a2, &mut self.b2, 2) - 0.5, halton(&mut self.a3, &mut self.b3, 3) - 0.5]
    }
}

/// `halton`: step the fraction `a / b` to the sequence's next number.
fn halton(a: &mut i32, b: &mut i32, base: i32) -> f32 {
    let x = *b - *a;
    match x {
        1 => {
            *a = 1;
            *b *= base;
        }
        _ => {
            let mut y = *b / base;
            while x <= y {
                y /= base;
            }
            *a = (1 + base) * y - x;
        }
    }
    *a as f32 / *b as f32
}
