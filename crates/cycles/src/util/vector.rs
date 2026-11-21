// SPDX-FileCopyrightText: 2011-2013 Intel Corporation
// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/util/math_float3.h` and `math_float4.h`: what
//! is done to a `float3` and a `float4`, on plain arrays.

/// `a + b`.
pub fn add<const N: usize>(a: [f32; N], b: [f32; N]) -> [f32; N] {
    std::array::from_fn(|index| a[index] + b[index])
}

/// `a - b`.
pub fn sub<const N: usize>(a: [f32; N], b: [f32; N]) -> [f32; N] {
    std::array::from_fn(|index| a[index] - b[index])
}

/// `a * factor`.
pub fn scale<const N: usize>(a: [f32; N], factor: f32) -> [f32; N] {
    a.map(|part| part * factor)
}

/// `dot`.
pub fn dot<const N: usize>(a: [f32; N], b: [f32; N]) -> f32 {
    a.iter().zip(b).map(|(a, b)| a * b).sum()
}

/// `cross`.
pub fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

/// `len`.
pub fn len<const N: usize>(a: [f32; N]) -> f32 {
    dot(a, a).sqrt()
}

/// `normalize`: a vector of no length comes out as not-a-number, as in
/// Cycles.
pub fn normalize<const N: usize>(a: [f32; N]) -> [f32; N] {
    let length = len(a);
    a.map(|part| part / length)
}

/// `safe_divide` by a float: zero when divided by zero.
pub fn safe_divide(a: [f32; 3], by: f32) -> [f32; 3] {
    match by != 0.0 {
        true => a.map(|part| part / by),
        false => [0.0; 3],
    }
}

/// `mix`: from `a` at 0 to `b` at 1, as a step from `a`.
pub fn mix<const N: usize>(a: [f32; N], b: [f32; N], t: f32) -> [f32; N] {
    std::array::from_fn(|index| a[index] + t * (b[index] - a[index]))
}

/// `interp`: from `a` at 0 to `b` at 1.
pub fn interp<const N: usize>(a: [f32; N], b: [f32; N], t: f32) -> [f32; N] {
    std::array::from_fn(|index| (1.0 - t) * a[index] + t * b[index])
}

/// `min`, part by part. A part of `a` that is not a number leaves `b`'s.
pub fn min(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|index| match a[index] < b[index] {
        true => a[index],
        false => b[index],
    })
}

/// `max`, part by part. A part of `a` that is not a number leaves `b`'s.
pub fn max(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|index| match a[index] > b[index] {
        true => a[index],
        false => b[index],
    })
}

/// `isfinite_safe`: no part is infinite or not a number.
pub fn is_finite(a: [f32; 3]) -> bool {
    a.iter().all(|part| part.is_finite())
}
