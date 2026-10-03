//! The 3D cursor (`Scene.cursor`, a `View3DCursor`): where new objects go
//! and what the viewport marks with a red and white ring. It turns as an
//! object does — a quaternion, an Euler order or an axis and angle, by
//! `rotation_mode` (`View3DCursor::rotation`).

use crate::blend::file::types::*;
use crate::blend::object::transform::{ObjectTransformRaw, object_to_mat4};

/// The 3D cursor: its place and turn as one matrix.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlendCursor {
    /// `m[col][row]`, the location in `[3]`.
    pub matrix: [[f32; 4]; 4],
}

impl Default for BlendCursor {
    /// At the origin, unturned, as a new scene has it.
    fn default() -> Self {
        Self { matrix: glam::Mat4::IDENTITY.to_cols_array_2d() }
    }
}

impl BlendCursor {
    pub fn location(&self) -> [f32; 3] {
        [self.matrix[3][0], self.matrix[3][1], self.matrix[3][2]]
    }
}

impl BlendFile {
    /// The cursor of the scene block `sc`.
    pub(crate) fn cursor(&self, sc: &BlendBlock, dna: &DnaType) -> BlendCursor {
        let Some(base) = self.off("Scene", "cursor", dna) else { return BlendCursor::default() };
        let at = |f: &str| self.off("View3DCursor", f, dna).map(|o| base + o);
        let d = &sc.data;
        let dflt = ObjectTransformRaw::default();
        let raw = ObjectTransformRaw {
            loc: self.f32s(d, at("location[3]"), dflt.loc),
            rot: self.f32s(d, at("rotation_euler[3]"), dflt.rot),
            quat: self.f32s(d, at("rotation_quaternion[4]"), dflt.quat),
            rot_axis: self.f32s(d, at("rotation_axis[3]"), dflt.rot_axis),
            rot_angle: self.f32s(d, at("rotation_angle"), [0.0])[0],
            rotmode: self.i16_at(d, at("rotation_mode"), 1),
            ..dflt
        };
        BlendCursor { matrix: object_to_mat4(&raw) }
    }
}
