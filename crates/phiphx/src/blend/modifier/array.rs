//! Array modifier (`ArrayModifierData`, Blender `MOD_array.cc`): repeat the
//! mesh `count` times, each copy displaced by the sum of the enabled offsets —
//! a constant vector, a fraction of the mesh's own bounding box, and/or
//! another object's transform relative to this one — and merges the copies
//! where they meet ("Merge", `array_merge.rs`).
//! Not yet: start/end caps, fit-to-curve and fit-to-length.

use super::{BModifier, BlendEvalMesh, EvalCtx};
#[path = "array_merge.rs"]
mod array_merge;
use super::driver::BModValue;
use crate::blend::file::types::*;
use glam::{Mat4, Vec3};

const OFF_CONST: u32 = 1 << 0;
const OFF_RELATIVE: u32 = 1 << 1;
const OFF_OBJ: u32 = 1 << 2;
/// `MOD_ARR_MERGE` / `MOD_ARR_MERGEFINAL` (`ArrayModifierData.flags`).
const MERGE: u32 = 1 << 0;
const MERGE_FINAL: u32 = 1 << 1;

/// "Merge": the distance, and whether the last copy meets the first.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct BArrayMerge {
    pub dist: f32,
    pub first_last: bool,
}

pub(crate) struct BArray {
    pub count: usize,
    pub offset_type: u32,
    pub constant: Vec3,
    pub relative: Vec3,
    pub offset_ob: u64,
    pub merge: Option<BArrayMerge>,
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
    let flags = f.int_field(md, "ArrayModifierData", "flags", dna);
    Some(Box::new(BArray {
        // Blender clamps to at least one copy — the original.
        count: f.int_field(md, "ArrayModifierData", "count", dna).max(1) as usize,
        offset_type: f.int_field(md, "ArrayModifierData", "offset_type", dna),
        constant: v3("offset[3]"),
        relative: v3("scale[3]"),
        offset_ob: f.ptr(md, f.off("ArrayModifierData", "*offset_ob", dna)),
        merge: (flags & MERGE != 0).then(|| BArrayMerge {
            dist: f.float_field(md, "ArrayModifierData", "merge_dist", dna, 0.01),
            first_last: flags & MERGE_FINAL != 0,
        }),
    }))
}

impl BArray {
    /// A plain Array for the driven-setting tests (`driven_tests.rs`).
    #[cfg(test)]
    pub(crate) fn for_test() -> Self {
        Self { count: 2, offset_type: 0, constant: Default::default(), relative: Default::default(), offset_ob: 0, merge: None }
    }

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

impl BArray {
    fn driven(&mut self, prop: &str, component: usize, value: BModValue) -> bool {
        let v = value.as_f32();
        match prop {
            // Blender's own minimum: an Array of zero copies is one copy.
            "count" => self.count = (v as i64).max(1) as usize,
            "constant_offset_displace" if component < 3 => self.constant[component] = v,
            "relative_offset_displace" if component < 3 => self.relative[component] = v,
            _ => return false,
        }
        true
    }
}
impl BModifier for BArray {
    fn clone_box(&self) -> Box<dyn BModifier> {
        Box::new(Self {
            count: self.count,
            offset_type: self.offset_type,
            constant: self.constant,
            relative: self.relative,
            offset_ob: self.offset_ob,
            merge: self.merge,
        })
    }

    fn set_driven(&mut self, prop: &str, component: usize, value: BModValue) -> bool {
        self.driven(prop, component, value)
    }

    fn apply(&self, mesh: &mut BlendEvalMesh, ctx: &EvalCtx) {
        if self.count <= 1 || mesh.vertices.is_empty() {
            return;
        }
        let step = self.step(mesh, ctx);
        let verts = mesh.vertices.len() as u32;
        let base_faces = mesh.faces.len();
        let base_uvs = mesh.loop_uvs.len();
        let base_colors = mesh.loop_colors.len();
        let base_loose = mesh.loose_edges.len();
        let base_creases = mesh.edge_creases.len();
        let vert_creases = mesh.vert_creases.len() == verts as usize;

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
            for n in 0..base_colors {
                mesh.loop_colors.push(mesh.loop_colors[n]);
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
            // Duplicate loose edges with vertex offset for this copy.
            for n in 0..base_loose {
                let (a, b) = mesh.loose_edges[n];
                mesh.loose_edges.push((a + base, b + base));
            }
            // `edge_interp.copy` / `vert_interp.copy`: the creases come along.
            for n in 0..base_creases {
                let (a, b, c) = mesh.edge_creases[n];
                mesh.edge_creases.push((a + base, b + base, c));
            }
            if vert_creases {
                mesh.vert_creases.extend_from_within(..verts as usize);
            }
        }
        if let Some(m) = self.merge {
            array_merge::array_merge(mesh, verts as usize, self.count, m.dist, m.first_last);
        }
    }
}

/// Silence the unused-import warning when the file is compiled alone.
const _: Option<Mat4> = None;
