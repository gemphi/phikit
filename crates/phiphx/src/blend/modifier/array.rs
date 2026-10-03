//! Array modifier (`ArrayModifierData`, Blender `MOD_array.cc`): repeat the
//! mesh `count` times, each copy displaced by the sum of the enabled offsets —
//! a constant vector, a fraction of the mesh's own bounding box, and/or
//! another object's transform relative to this one.
//! Not yet: start/end caps, merge, fit-to-curve and fit-to-length.

use super::{BModifier, BlendEvalMesh, EvalCtx};
use crate::blend::file::types::*;
use glam::{Mat4, Vec3};

const OFF_CONST: u32 = 1 << 0;
const OFF_RELATIVE: u32 = 1 << 1;
const OFF_OBJ: u32 = 1 << 2;

pub(crate) struct BArray {
    pub count: usize,
    pub offset_type: u32,
    pub constant: Vec3,
    pub relative: Vec3,
    pub offset_ob: u64,
}

pub(super) fn read(
    f: &BlendFile,
    md: &BlendBlock,
    _ob: &BlendBlock,
    dna: &DnaType,
) -> Option<Box<dyn BModifier>> {
    let v3 = |name: &str| {
        f.off("ArrayModifierData", name, dna)
            .and_then(|o| md.data.get(o..o + 12))
            .map_or(Vec3::ZERO, |b| {
                Vec3::new(
                    f.read_f32(&b[0..4]),
                    f.read_f32(&b[4..8]),
                    f.read_f32(&b[8..12]),
                )
            })
    };
    Some(Box::new(BArray {
        // Blender clamps to at least one copy — the original.
        count: f.int_field(md, "ArrayModifierData", "count", dna).max(1) as usize,
        offset_type: f.int_field(md, "ArrayModifierData", "offset_type", dna),
        constant: v3("offset[3]"),
        relative: v3("scale[3]"),
        offset_ob: f.ptr(md, f.off("ArrayModifierData", "*offset_ob", dna)),
    }))
}

impl BArray {
    /// The displacement between one copy and the next, in object space.
    fn step(&self, mesh: &BlendEvalMesh, ctx: &EvalCtx) -> Vec3 {
        let mut step = Vec3::ZERO;
        if self.offset_type & OFF_CONST != 0 {
            step += self.constant;
        }
        if self.offset_type & OFF_RELATIVE != 0 {
            // A fraction of the mesh's own size, as Blender measures it from
            // the bounding box before the modifier runs.
            let (min, max) = bounds(mesh);
            step += self.relative * (max - min);
        }
        if self.offset_type & OFF_OBJ != 0 {
            if let Some(other) = ctx.worlds.get(&self.offset_ob) {
                // The other object's transform expressed in this object's
                // space: `inverse(this) * other`, whose translation is the
                // offset Blender applies.
                let rel = ctx.world.inverse() * *other;
                step += rel.w_axis.truncate();
            }
        }
        step
    }
}

fn bounds(mesh: &BlendEvalMesh) -> (Vec3, Vec3) {
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);
    for v in &mesh.vertices {
        let v = Vec3::from(*v);
        min = min.min(v);
        max = max.max(v);
    }
    if mesh.vertices.is_empty() {
        return (Vec3::ZERO, Vec3::ZERO);
    }
    (min, max)
}

impl BModifier for BArray {
    fn apply(&self, mesh: &mut BlendEvalMesh, ctx: &EvalCtx) {
        if self.count <= 1 || mesh.vertices.is_empty() {
            return;
        }
        let step = self.step(mesh, ctx);
        let verts = mesh.vertices.len() as u32;
        let base_faces = mesh.faces.len();
        let base_uvs = mesh.loop_uvs.len();

        for copy in 1..self.count {
            let shift = step * copy as f32;
            for i in 0..verts as usize {
                let v = Vec3::from(mesh.vertices[i]) + shift;
                mesh.vertices.push(v.into());
            }
            let base = verts * copy as u32;
            for face in 0..base_faces {
                let moved: Vec<u32> = mesh.faces.get(face).iter().map(|c| c + base).collect();
                mesh.faces.push(&moved);
            }
            for n in 0..base_uvs {
                mesh.loop_uvs.push(mesh.loop_uvs[n]);
            }
            for n in 0..base_faces {
                let material = mesh.face_materials.get(n).copied().unwrap_or(0);
                mesh.face_materials.push(material);
                let smooth = mesh.face_smooth.get(n).copied().unwrap_or(true);
                mesh.face_smooth.push(smooth);
            }
            for n in 0..verts as usize {
                let weights = mesh.dverts.get(n).cloned().unwrap_or_default();
                mesh.dverts.push(weights);
            }
        }
    }
}

/// Silence the unused-import warning when the file is compiled alone.
const _: Option<Mat4> = None;
