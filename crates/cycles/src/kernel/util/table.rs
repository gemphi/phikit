// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/util/lookup_table.h`
//! (`lookup_table_read`, lines 14-29). The 2D and 3D reads come with the
//! closures that use them.

/// `lookup_table_read`: the table of `size` entries at `offset` in
/// `lookup_table`, read at `x` from 0 to 1 with the two nearest entries
/// mixed. An `x` outside is read at the nearer end.
pub fn lookup_table_read(lookup_table: &[f32], x: f32, offset: usize, size: usize) -> f32 {
    let last = size.saturating_sub(1);
    let x = x.clamp(0.0, 1.0) * last as f32;
    let index = (x as usize).min(last);
    let next = (index + 1).min(last);
    let t = x - index as f32;
    let at = |index: usize| lookup_table.get(index + offset).copied().unwrap_or(0.0);
    match t == 0.0 {
        true => at(index),
        false => (1.0 - t) * at(index) + t * at(next),
    }
}
