//! The cubic Bezier a B-Bone follows: forward differencing, evaluation and handle easing.
use glam::Vec3;

/// `interpf(target, origin, fac)`.
pub(super) fn interpf(target: f32, origin: f32, fac: f32) -> f32 {
    fac * target + (1.0 - fac) * origin
}

/// `BKE_curve_forward_diff_bezier`, one axis.
pub(super) fn forward_diff(q0: f32, q1: f32, q2: f32, q3: f32, it: usize) -> Vec<f32> {
    let f = it as f32;
    let (r0, r1) = (q0, 3.0 * (q1 - q0) / f);
    let r2 = 3.0 * (q0 - 2.0 * q1 + q2) / (f * f);
    let r3 = (q3 - q0 + 3.0 * (q1 - q2)) / (f * f * f);
    let (mut a0, mut a1, mut a2, a3) = (r0, r1 + r2 + r3, 2.0 * r2 + 6.0 * r3, 6.0 * r3);
    let mut out = Vec::with_capacity(it + 1);
    for _ in 0..=it {
        out.push(a0);
        a0 += a1;
        a1 += a2;
        a2 += a3;
    }
    out
}

/// `equalize_cubic_bezier`: parameter values that cut the curve into arcs
/// whose lengths follow `scales`.
pub(super) fn equalize(control: [Vec3; 4], temp: usize, final_segments: usize, scales: &[f32]) -> Vec<f32> {
    let axes: [Vec<f32>; 3] =
        [0, 1, 2].map(|k| forward_diff(control[0][k], control[1][k], control[2][k], control[3][k], temp));
    let coords: Vec<Vec3> = (0..=temp).map(|i| Vec3::new(axes[0][i], axes[1][i], axes[2][i])).collect();
    let mut pdist = vec![0.0f32; temp + 1];
    for i in 0..temp {
        pdist[i + 1] = pdist[i] + (coords[i + 1] - coords[i]).length();
    }
    let sum: f32 = scales[..final_segments].iter().sum();
    let step = pdist[temp] / sum;
    let mut out = vec![0.0f32; final_segments + 1];
    let (mut dist, mut nr) = (0.0f32, 1usize);
    for i in 1..=final_segments {
        dist += scales[i - 1] * step;
        while nr < temp && dist >= pdist[nr] {
            nr += 1;
        }
        let fac = (pdist[nr] - dist) / (pdist[nr] - pdist[nr - 1]);
        out[i] = (nr as f32 - fac) / temp as f32;
    }
    out[final_segments] = 1.0;
    out
}

/// `evaluate_cubic_bezier`: position and tangent at `t`.
pub(super) fn bezier_at(c: [Vec3; 4], t: f32) -> (Vec3, Vec3) {
    let l1 = [c[0].lerp(c[1], t), c[1].lerp(c[2], t), c[2].lerp(c[3], t)];
    let l2 = [l1[0].lerp(l1[1], t), l1[1].lerp(l1[2], t)];
    let tangent = l2[1] - l2[0];
    (l2[0] + tangent * t, tangent)
}

/// `ease_handle_axis`: fade to the second derivative when the handle is short.
pub(super) fn ease_axis(d1: Vec3, d2: Vec3) -> Vec3 {
    const GAP: f32 = 0.1;
    let len2 = d2.length_squared();
    if len2 == 0.0 {
        return d1;
    }
    let ratio = d1.length_squared() / len2;
    if ratio < GAP * GAP { d1 + d2 * (GAP - ratio.sqrt()) } else { d1 }
}
