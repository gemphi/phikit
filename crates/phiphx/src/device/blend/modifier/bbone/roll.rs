//! Roll and orientation: `vec_roll_to_mat3` and its inverse.
use glam::{Mat3, Quat, Vec3};

/// `vec_roll_to_mat3_normalized`: the rotation taking +Y onto `nor`, then
/// `roll` radians about `nor`.
pub(super) fn vec_roll_to_mat3(vec: Vec3, roll: f32) -> Mat3 {
    const SAFE: f32 = 6.1e-3;
    const CRITICAL: f32 = 2.5e-4;
    let nor = vec.normalize_or_zero();
    let (x, y, z) = (nor.x, nor.y, nor.z);
    let mut theta = 1.0 + y;
    let theta_alt = x * x + z * z;
    let b = if theta > SAFE || theta_alt > CRITICAL * CRITICAL {
        if theta <= SAFE {
            theta = theta_alt * 0.5 + theta_alt * theta_alt * 0.125;
        }
        // Blender writes `bMatrix[col][row]`, which is glam's column order.
        Mat3::from_cols(
            Vec3::new(1.0 - x * x / theta, -x, -x * z / theta),
            Vec3::new(x, y, z),
            Vec3::new(-x * z / theta, -z, 1.0 - z * z / theta),
        )
    } else {
        Mat3::from_cols(Vec3::new(-1.0, 0.0, 0.0), Vec3::new(0.0, -1.0, 0.0), Vec3::Z)
    };
    Mat3::from_axis_angle(nor, roll) * b
}

/// `mat3_vec_to_roll`: the twist of `mat` about `vec`, as a roll angle.
pub(super) fn mat3_vec_to_roll(mat: Mat3, vec: Vec3) -> f32 {
    let inv = vec_roll_to_mat3(vec, 0.0).inverse();
    let roll = inv * orthonormal(mat);
    let q = Quat::from_mat3(&orthonormal(roll));
    // `quat_split_swing_and_twist(q, axis = 1)` on a canonical quaternion.
    let (w, y) = if q.w < 0.0 { (-q.w, -q.y) } else { (q.w, q.y) };
    2.0 * y.atan2(w)
}

pub(super) fn orthonormal(m: Mat3) -> Mat3 {
    Mat3::from_cols(m.x_axis.normalize_or_zero(), m.y_axis.normalize_or_zero(), m.z_axis.normalize_or_zero())
}

/// `cubic_tangent_factor_circle_v3`.
pub(super) fn circle_factor(l: Vec3, r: Vec3) -> f32 {
    const EPS: f32 = 1e-5;
    let dot = l.dot(r);
    if dot > 1.0 - EPS {
        return (1.0 / 3.0) * 0.75;
    }
    if dot < -1.0 + EPS {
        return 0.5;
    }
    let angle = dot.acos() / 2.0;
    let (sin, cos) = angle.sin_cos();
    ((1.0 - cos) / (sin * 2.0)) / sin
}
