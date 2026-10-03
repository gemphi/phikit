//! Rotation / matrix math — ports of `eulO_to_mat3`, `quat_to_mat3`,
//! `axis_angle_to_mat3` (`blenlib/intern/math_rotation.cc`) and
//! `BKE_object_to_mat4` / `solve_parenting` (`blenkernel/intern/object.cc`).
//! Matrices use Blender's `m[col][row]` layout (glam column order).

use crate::blend::object::transform::ObjectTransformRaw;
use glam::{Mat3, Mat4, Quat, Vec3};


/// `eulO_to_mat3`: Blender's rotation-order table (`rotOrders[]`).
fn eul_o_to_mat3(e: [f32; 3], order: i16) -> Mat3 {
    // (axis i, j, k), parity
    let (axis, parity): ([usize; 3], bool) = match order {
        2 => ([0, 2, 1], true),  // XZY
        3 => ([1, 0, 2], true),  // YXZ
        4 => ([1, 2, 0], false), // YZX
        5 => ([2, 0, 1], false), // ZXY
        6 => ([2, 1, 0], true),  // ZYX
        _ => ([0, 1, 2], false), // XYZ (and any invalid order)
    };
    let (i, j, k) = (axis[0], axis[1], axis[2]);
    let (ti, tj, th) = if parity {
        (-e[i] as f64, -e[j] as f64, -e[k] as f64)
    } else {
        (e[i] as f64, e[j] as f64, e[k] as f64)
    };
    let (ci, cj, ch) = (ti.cos(), tj.cos(), th.cos());
    let (si, sj, sh) = (ti.sin(), tj.sin(), th.sin());
    let (cc, cs, sc, ss) = (ci * ch, ci * sh, si * ch, si * sh);
    let mut m = [[0.0f64; 3]; 3]; // m[col][row], Blender layout
    m[i][i] = cj * ch;
    m[j][i] = sj * sc - cs;
    m[k][i] = sj * cc + ss;
    m[i][j] = cj * sh;
    m[j][j] = sj * ss + cc;
    m[k][j] = sj * cs - sc;
    m[i][k] = -sj;
    m[j][k] = cj * si;
    m[k][k] = cj * ci;
    Mat3::from_cols(
        Vec3::new(m[0][0] as f32, m[0][1] as f32, m[0][2] as f32),
        Vec3::new(m[1][0] as f32, m[1][1] as f32, m[1][2] as f32),
        Vec3::new(m[2][0] as f32, m[2][1] as f32, m[2][2] as f32),
    )
}

/// Blender quaternions are stored `(w, x, y, z)`.
fn quat_to_mat3(q: [f32; 4]) -> Mat3 {
    let q = Quat::from_xyzw(q[1], q[2], q[3], q[0]);
    let q = if q.length_squared() > 0.0 { q.normalize() } else { Quat::IDENTITY };
    Mat3::from_quat(q)
}

fn axis_angle_to_mat3(axis: [f32; 3], angle: f32) -> Mat3 {
    let a = Vec3::from(axis);
    if a.length_squared() <= f32::EPSILON {
        return Mat3::IDENTITY;
    }
    Mat3::from_axis_angle(a.normalize(), angle)
}

/// `BKE_object_rot_to_mat3(ob, mat, use_drot = true)`.
fn object_rot_to_mat3(raw: &ObjectTransformRaw) -> Mat3 {
    let (rmat, dmat) = match raw.rotmode {
        0 => (quat_to_mat3(raw.quat), quat_to_mat3(raw.dquat)), // ROT_MODE_QUAT
        -1 => (
            axis_angle_to_mat3(raw.rot_axis, raw.rot_angle),
            axis_angle_to_mat3(raw.drot_axis, raw.drot_angle),
        ),
        order => (eul_o_to_mat3(raw.rot, order), eul_o_to_mat3(raw.drot, order)),
    };
    // Blender: mul_m3_m3m3(mat, dmat, rmat)  ⇒  mat = dmat · rmat
    dmat * rmat
}

/// `BKE_object_to_mat4`: local matrix from loc/rot/scale (+ deltas).
pub(crate) fn object_to_mat4(raw: &ObjectTransformRaw) -> [[f32; 4]; 4] {
    let scale = Vec3::from(raw.size) * Vec3::from(raw.dscale);
    let smat = Mat3::from_diagonal(scale);
    let rmat = object_rot_to_mat3(raw);
    // BKE_object_to_mat3: mul_m3_m3m3(mat, rmat, smat) ⇒ rmat · smat
    let m3 = rmat * smat;
    let t = Vec3::from(raw.loc) + Vec3::from(raw.dloc);
    let m4 = Mat4::from_cols(
        m3.x_axis.extend(0.0),
        m3.y_axis.extend(0.0),
        m3.z_axis.extend(0.0),
        t.extend(1.0),
    );
    m4.to_cols_array_2d()
}

/// world = parent_world · parentinv · local  (`solve_parenting`, PAROBJECT).
pub(crate) fn compose_world(
    parent_world: &[[f32; 4]; 4],
    parentinv: &[[f32; 4]; 4],
    local: &[[f32; 4]; 4],
) -> [[f32; 4]; 4] {
    let p = Mat4::from_cols_array_2d(parent_world);
    let pi = Mat4::from_cols_array_2d(parentinv);
    let l = Mat4::from_cols_array_2d(local);
    (p * pi * l).to_cols_array_2d()
}


#[cfg(test)]
#[path = "rotation_tests.rs"]
mod tests;
