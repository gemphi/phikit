// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/util/transform.cpp` (`transform_to_quat`,
//! `transform_decompose`, `transform_motion_decompose`) and `transform.h`
//! (`DecomposedTransform`, `quat_interpolate`, `transform_compose`,
//! `transform_motion_array_interpolate`): a transform taken apart into a
//! turn, a move and a scale, so the steps of a motion can be blended.

use super::Transform;
use super::vector::{dot, interp, len, normalize, safe_divide, scale, sub};

/// `DecomposedTransform`: `x` is the turn as a quaternion, `y` the move
/// and the scale along x, `z` and `w` the rest of the scale and the shear.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct DecomposedTransform {
    pub x: [f32; 4],
    pub y: [f32; 4],
    pub z: [f32; 4],
    pub w: [f32; 4],
}

/// `transform_to_quat`: the turn of a matrix with no scale, in doubles as
/// Cycles works it out.
fn to_quat(transform: &Transform) -> [f32; 4] {
    let m = [transform.x, transform.y, transform.z];
    let trace = f64::from(m[0][0] + m[1][1] + m[2][2]);
    match trace > 0.0 {
        true => {
            let root = (trace + 1.0).sqrt();
            let s = 0.5 / root;
            let part = |a: f32, b: f32| (f64::from(a - b) * s) as f32;
            [part(m[2][1], m[1][2]), part(m[0][2], m[2][0]), part(m[1][0], m[0][1]), (root / 2.0) as f32]
        }
        false => {
            let mut i = 0;
            for axis in [1, 2] {
                if m[axis][axis] > m[i][i] {
                    i = axis;
                }
            }
            let (j, k) = ((i + 1) % 3, (i + 2) % 3);
            let mut s = (f64::from(m[i][i] - (m[j][j] + m[k][k])) + 1.0).sqrt();
            let mut q = [0.0f64; 3];
            q[i] = s * 0.5;
            if s != 0.0 {
                s = 0.5 / s;
            }
            q[j] = f64::from(m[j][i] + m[i][j]) * s;
            q[k] = f64::from(m[k][i] + m[i][k]) * s;
            [q[0] as f32, q[1] as f32, q[2] as f32, (f64::from(m[k][j] - m[j][k]) * s) as f32]
        }
    }
}

/// `transform_decompose`.
fn decompose(transform: &Transform) -> DecomposedTransform {
    let mut m = *transform;
    (m.x[3], m.y[3], m.z[3]) = (0.0, 0.0, 0.0);
    // Scale and shear first, a column at a time.
    let (mut colx, mut coly, mut colz) = (m.column(0), m.column(1), m.column(2));
    let mut size = [0.0f32; 3];
    size[0] = len(colx);
    colx = safe_divide(colx, size[0]);
    let shear_z = dot(colx, coly);
    coly = sub(coly, scale(colx, shear_z));
    size[1] = len(coly);
    coly = safe_divide(coly, size[1]);
    let shear_y = dot(colx, colz);
    colz = sub(colz, scale(colx, shear_y));
    let shear_x = dot(coly, colz);
    colz = sub(colz, scale(coly, shear_x));
    size[2] = len(colz);
    colz = safe_divide(colz, size[2]);
    m.set_column(0, colx);
    m.set_column(1, coly);
    m.set_column(2, colz);
    if m.negative_scale() {
        size = scale(size, -1.0);
        m = m * Transform::scale(-1.0, -1.0, -1.0);
    }
    DecomposedTransform {
        x: to_quat(&m),
        y: [transform.x[3], transform.y[3], transform.z[3], size[0]],
        z: [shear_z, shear_y, 0.0, size[1]],
        w: [shear_x, 0.0, 0.0, size[2]],
    }
}

/// `transform_motion_decompose`: every step of a motion taken apart, each
/// turn the short way round from the one before it.
pub fn motion_decompose(motion: &[Transform]) -> Vec<DecomposedTransform> {
    let mut out: Vec<DecomposedTransform> = motion.iter().map(decompose).collect();
    for index in 1..out.len() {
        // A quaternion and its negative are one turn; the nearer one is kept.
        if dot(out[index - 1].x, out[index].x) < 0.0 {
            out[index].x = scale(out[index].x, -1.0);
        }
    }
    // A step with no size has no turn of its own: it takes a neighbour's.
    for index in 0..out.len() {
        let flat = [out[index].y[3], out[index].z[3], out[index].w[3]] == [0.0; 3];
        match (flat, index > 0, index + 1 < out.len()) {
            (true, true, _) => out[index].x = out[index - 1].x,
            (true, false, true) => out[index].x = out[index + 1].x,
            _ => {}
        }
    }
    out
}

/// `quat_interpolate`, as a CPU does it: along the arc between the two.
fn quat_interpolate(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    let cos_theta = dot(a, b);
    match cos_theta > 0.9995 {
        // Nearly the same turn: a straight blend.
        true => normalize(interp(a, b, t)),
        false => {
            let theta = cos_theta.clamp(-1.0, 1.0).acos();
            let across = normalize(sub(b, scale(a, cos_theta)));
            let angle = theta * t;
            std::array::from_fn(|index| a[index] * angle.cos() + across[index] * angle.sin())
        }
    }
}

/// `transform_compose`: the matrix a decomposed transform stands for.
pub fn compose(decomp: &DecomposedTransform) -> Transform {
    let root = std::f32::consts::SQRT_2;
    let [q1, q2, q3, q0] = decomp.x.map(|part| root * part);
    let (qda, qdb, qdc) = (q0 * q1, q0 * q2, q0 * q3);
    let (qaa, qab, qac) = (q1 * q1, q1 * q2, q1 * q3);
    let (qbb, qbc, qcc) = (q2 * q2, q2 * q3, q3 * q3);
    let turn = [[1.0 - qbb - qcc, -qdc + qab, qdb + qac], [qdc + qab, 1.0 - qaa - qcc, -qda + qbc], [-qdb + qac, qda + qbc, 1.0 - qaa - qbb]];
    let size = [[decomp.y[3], decomp.z[2], decomp.w[1]], [decomp.z[0], decomp.z[3], decomp.w[2]], [decomp.z[1], decomp.w[0], decomp.w[3]]];
    let row = |axis: usize| [dot(turn[axis], size[0]), dot(turn[axis], size[1]), dot(turn[axis], size[2]), decomp.y[axis]];
    Transform { x: row(0), y: row(1), z: row(2) }
}

/// `transform_motion_array_interpolate`: the transform at `time`, from 0
/// at the first step to 1 at the last. `motion` has two steps or more.
#[allow(clippy::manual_clamp, reason = "one step alone has no step before the last, and clamp would panic")]
pub fn motion_array_interpolate(motion: &[DecomposedTransform], time: f32) -> Transform {
    let last = motion.len() as i32 - 1;
    let step = ((time * last as f32) as i32).min(last - 1).max(0) as usize;
    let t = time * last as f32 - step as f32;
    let (a, b) = (&motion[step], &motion[(step + 1).min(motion.len() - 1)]);
    compose(&DecomposedTransform { x: quat_interpolate(a.x, b.x, t), y: interp(a.y, b.y, t), z: interp(a.z, b.z, t), w: interp(a.w, b.w, t) })
}

#[cfg(test)]
mod test;
