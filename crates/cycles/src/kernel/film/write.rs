// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/film/write.h` (lines 20-130): the
//! pixel of a path in the render buffer and the writes into a pass. A path
//! owns its pixel while it runs, so a write is a plain add (Cycles' CPU
//! build does not define `__ATOMIC_PASS_WRITE__` either).
use crate::kernel::KernelGlobals;
use crate::kernel::integrator::IntegratorState;

/// `film_pass_pixel_render_buffer`: the floats of the path's pixel
/// (`render_pixel_index * pass_stride`), within `render_buffer`, which
/// starts at pixel `first`.
pub fn film_pass_pixel_render_buffer<'a>(kg: &KernelGlobals, state: &IntegratorState, render_buffer: &'a mut [f32], first: u32) -> &'a mut [f32] {
    let start = (state.path.render_pixel_index - first) as usize * kg.film.pass_stride as usize;
    &mut render_buffer[start..start + kg.film.pass_stride as usize]
}

/// `film_write_pass_float3`.
pub fn film_write_pass_spectrum(buffer: &mut [f32], value: [f32; 3]) {
    buffer[0] += value[0];
    buffer[1] += value[1];
    buffer[2] += value[2];
}

/// `film_write_pass_float4`.
pub fn film_write_pass_float4(buffer: &mut [f32], value: [f32; 4]) {
    for (slot, part) in buffer.iter_mut().zip(value) {
        *slot += part;
    }
}
