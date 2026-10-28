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

use crate::blend::file::types::*;
pub(crate) use crate::blend::object::rotation::{compose_world, object_to_mat4};

/// Blender `eObject_Partype` (low 4 bits of `Object.partype`).
pub const PAROBJECT: u8 = 0;

/// Everything `BKE_object_to_mat4` needs, read DNA-driven from an `OB` block.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObjectTransformRaw {
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
    /// `Object.par1`, `par2`, `par3`: the parent's vertex indices a
    /// `PARVERT1` (the first) or `PARVERT3` child hangs from.
    pub par: [i32; 3],
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
            par: [0; 3],
            ob_type: 1,
        }
    }
}

impl ObjectTransformRaw {
    /// An object's own channels from a location, a quaternion (`w, x, y,
    /// z`) and a scale, in quaternion rotation mode — what a transform built
    /// in code (a game's ball) hands the motion.
    pub fn from_parts(loc: [f32; 3], quat: [f32; 4], size: [f32; 3]) -> Self {
        Self { loc, quat, size, rotmode: 0, ..Self::default() }
    }
}

pub(crate) const IDENTITY4: [[f32; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

impl BlendFile {
    pub(crate) fn f32_at(&self, data: &[u8], off: usize) -> f32 {
        if off + 4 <= data.len() {
            self.read_f32(&data[off..off + 4])
        } else {
            0.0
        }
    }
    pub(crate) fn f32s<const N: usize>(&self, data: &[u8], off: Option<usize>, default: [f32; N]) -> [f32; N] {
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
    pub(crate) fn i32_at(&self, data: &[u8], off: usize) -> i32 {
        match off + 4 <= data.len() {
            true => self.read_i32(&data[off..off + 4]),
            false => 0,
        }
    }
    pub(crate) fn i16_at(&self, data: &[u8], off: Option<usize>, default: i16) -> i16 {
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
            par: ["par1", "par2", "par3"].map(|n| off(n).map_or(0, |o| self.i32_at(d, o))),
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
            crate::blend::object::kind::ObjectKind(raw.ob_type).name(),
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

