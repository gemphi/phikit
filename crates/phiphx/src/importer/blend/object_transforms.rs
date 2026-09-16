//! Object transform extraction — a port of Blender's `BKE_object_to_mat4`
//! (`blenkernel/intern/object.cc`) and `eulO_to_mat3` / `quat_to_mat3` /
//! `axis_angle_to_mat3` (`blenlib/intern/math_rotation.cc`).
//!
//! Since Blender 4.x the world matrix (`object_to_world`) is **runtime-only**
//! and is not stored in the file, so it has to be rebuilt exactly the way
//! Blender does: local = T(loc+dloc) · R(rotmode: rot/drot | quat/dquat |
//! rotAxis/rotAngle) · S(size·dscale); world = parent.world · parentinv ·
//! local (see [`BlendFile::resolve_world_matrices`] in `scene.rs`).
//!
//! Matrices use Blender's `float[4][4]` layout: `m[col][row]`, translation in
//! `m[3]`. This is also glam's column layout (`Mat4::from_cols_array_2d`).

use super::types::*;
use glam::{Mat3, Mat4, Quat, Vec3};

/// Blender `eObject_Partype` (low 4 bits of `Object.partype`).
pub const PAROBJECT: u8 = 0;

/// Everything `BKE_object_to_mat4` needs, read DNA-driven from an `OB` block.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ObjectTransformRaw {
    pub loc: [f32; 3],
    pub dloc: [f32; 3],
    pub rot: [f32; 3],
    pub drot: [f32; 3],
    pub quat: [f32; 4],
    pub dquat: [f32; 4],
    pub rot_axis: [f32; 3],
    pub drot_axis: [f32; 3],
    pub rot_angle: f32,
    pub drot_angle: f32,
    pub size: [f32; 3],
    pub dscale: [f32; 3],
    pub rotmode: i16,
    pub parentinv: [[f32; 4]; 4],
    pub parent_address: u64,
    pub partype: u8,
    pub ob_type: i16,
}

impl Default for ObjectTransformRaw {
    fn default() -> Self {
        Self {
            loc: [0.0; 3],
            dloc: [0.0; 3],
            rot: [0.0; 3],
            drot: [0.0; 3],
            quat: [1.0, 0.0, 0.0, 0.0],
            dquat: [1.0, 0.0, 0.0, 0.0],
            rot_axis: [0.0, 1.0, 0.0],
            drot_axis: [0.0, 1.0, 0.0],
            rot_angle: 0.0,
            drot_angle: 0.0,
            size: [1.0; 3],
            dscale: [1.0; 3],
            rotmode: 1,
            parentinv: IDENTITY4,
            parent_address: 0,
            partype: PAROBJECT,
            ob_type: 1,
        }
    }
}

pub(crate) const IDENTITY4: [[f32; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// `Object.type` → the name Blender's outliner/RNA uses (`OB_*` in
/// `DNA_object_types.h`, Blender 5.2).
pub fn object_type_name(t: i16) -> String {
    match t {
        0 => "EMPTY",
        1 => "MESH",
        2 => "CURVE",
        3 => "SURFACE",
        4 => "FONT",
        5 => "META",
        10 => "LIGHT",
        11 => "CAMERA",
        12 => "SPEAKER",
        13 => "LIGHT_PROBE",
        22 => "LATTICE",
        25 => "ARMATURE",
        26 => "GPENCIL",
        27 => "CURVES",
        28 => "POINTCLOUD",
        29 => "VOLUME",
        30 => "GREASEPENCIL",
        other => return format!("TYPE_{other}"),
    }
    .to_string()
}

impl BlendFile {
    fn f32_at(&self, data: &[u8], off: usize) -> f32 {
        if off + 4 <= data.len() {
            self.read_f32(&data[off..off + 4])
        } else {
            0.0
        }
    }
    fn f32s<const N: usize>(&self, data: &[u8], off: Option<usize>, default: [f32; N]) -> [f32; N] {
        let Some(off) = off else { return default };
        if off + 4 * N > data.len() {
            return default;
        }
        let mut out = [0.0; N];
        for (i, v) in out.iter_mut().enumerate() {
            *v = self.f32_at(data, off + 4 * i);
        }
        out
    }
    fn i16_at(&self, data: &[u8], off: Option<usize>, default: i16) -> i16 {
        match off {
            Some(o) if o + 2 <= data.len() => self.read_u16(&data[o..o + 2]) as i16,
            _ => default,
        }
    }

    /// Read every transform-related field of an `Object` block via the file's
    /// DNA (no hard-coded offsets).
    pub(crate) fn read_object_transform_raw(
        &self,
        block: &BlendBlock,
        dna: &DnaType,
    ) -> ObjectTransformRaw {
        let ps = self.header.pointer_size as usize;
        let le = self.header.little_endian;
        let off = |name: &str| Self::struct_field_offset_exact("Object", name, dna, ps);
        let d = &block.data;
        let dflt = ObjectTransformRaw::default();

        let mut parentinv = IDENTITY4;
        if let Some(o) = off("parentinv[4][4]") {
            if o + 64 <= d.len() {
                for c in 0..4 {
                    for r in 0..4 {
                        parentinv[c][r] = self.f32_at(d, o + (c * 4 + r) * 4);
                    }
                }
            }
        }
        let parent_address = off("*parent")
            .map(|o| Self::read_ptr(d, o, ps, le))
            .unwrap_or(0);
        let partype = off("partype")
            .and_then(|o| d.get(o).copied())
            .map(|b| b & 0x0F)
            .unwrap_or(PAROBJECT);

        ObjectTransformRaw {
            loc: self.f32s(d, off("loc[3]"), dflt.loc),
            dloc: self.f32s(d, off("dloc[3]"), dflt.dloc),
            rot: self.f32s(d, off("rot[3]"), dflt.rot),
            drot: self.f32s(d, off("drot[3]"), dflt.drot),
            quat: self.f32s(d, off("quat[4]"), dflt.quat),
            dquat: self.f32s(d, off("dquat[4]"), dflt.dquat),
            rot_axis: self.f32s(d, off("rotAxis[3]"), dflt.rot_axis),
            drot_axis: self.f32s(d, off("drotAxis[3]"), dflt.drot_axis),
            rot_angle: off("rotAngle").map(|o| self.f32_at(d, o)).unwrap_or(0.0),
            drot_angle: off("drotAngle").map(|o| self.f32_at(d, o)).unwrap_or(0.0),
            size: self.f32s(d, off("size[3]"), dflt.size),
            dscale: self.f32s(d, off("dscale[3]"), dflt.dscale),
            rotmode: self.i16_at(d, off("rotmode"), 1),
            parentinv,
            parent_address,
            partype,
            ob_type: self.i16_at(d, off("type"), 1),
        }
    }

    /// Legacy entry kept for `object.rs`: returns
    /// `(loc, rot, quat, size, matrix_LOCAL, type_name)`. The matrix is the
    /// object's **local** matrix (`BKE_object_to_mat4`); the world matrix is
    /// resolved after all objects are read (parent chain) in `scene.rs`.
    pub(crate) fn extract_object_transforms(
        &self,
        block: &BlendBlock,
        dna: &DnaType,
        _obj_idx: usize,
    ) -> (
        [f32; 3],
        [f32; 3],
        [f32; 4],
        [f32; 3],
        [[f32; 4]; 4],
        String,
    ) {
        let raw = self.read_object_transform_raw(block, dna);
        (
            raw.loc,
            raw.rot,
            raw.quat,
            raw.size,
            object_to_mat4(&raw),
            object_type_name(raw.ob_type),
        )
    }

    /// Fallback used only when the file has no `Object` struct in its DNA
    /// (never the case for a valid file): identity transform, MESH.
    #[allow(dead_code)]
    pub(crate) fn extract_transforms_fallback(
        &self,
        _block: &BlendBlock,
    ) -> ([f32; 3], [f32; 3], [f32; 3], String) {
        ([0.0; 3], [0.0; 3], [1.0; 3], "MESH".to_string())
    }
}

// ── math_rotation ports ───────────────────────────────────────────────────

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
mod tests {
    use super::*;

    fn approx(a: [[f32; 4]; 4], b: [[f32; 4]; 4]) -> bool {
        a.iter()
            .flatten()
            .zip(b.iter().flatten())
            .all(|(x, y)| (x - y).abs() < 1e-5)
    }

    #[test]
    fn identity_object_is_identity() {
        assert!(approx(object_to_mat4(&ObjectTransformRaw::default()), IDENTITY4));
    }

    #[test]
    fn translation_lands_in_column_3() {
        let raw = ObjectTransformRaw { loc: [1.0, 2.0, 3.0], ..Default::default() };
        let m = object_to_mat4(&raw);
        assert_eq!(m[3], [1.0, 2.0, 3.0, 1.0]);
    }

    #[test]
    fn euler_xyz_90deg_about_z_maps_x_to_y() {
        let raw = ObjectTransformRaw {
            rot: [0.0, 0.0, std::f32::consts::FRAC_PI_2],
            ..Default::default()
        };
        let m = Mat4::from_cols_array_2d(&object_to_mat4(&raw));
        let v = m.transform_vector3(Vec3::X);
        assert!((v - Vec3::Y).length() < 1e-5, "{v:?}");
    }

    #[test]
    fn scale_and_delta_scale_multiply() {
        let raw = ObjectTransformRaw { size: [2.0; 3], dscale: [0.5, 1.0, 1.0], ..Default::default() };
        let m = object_to_mat4(&raw);
        assert!((m[0][0] - 1.0).abs() < 1e-6 && (m[1][1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn child_inherits_parent_translation() {
        let parent = ObjectTransformRaw { loc: [10.0, 0.0, 0.0], ..Default::default() };
        let child = ObjectTransformRaw { loc: [0.0, 1.0, 0.0], ..Default::default() };
        let pw = object_to_mat4(&parent);
        let w = compose_world(&pw, &IDENTITY4, &object_to_mat4(&child));
        assert_eq!(w[3], [10.0, 1.0, 0.0, 1.0]);
    }
}
