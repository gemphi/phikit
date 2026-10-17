//! Mirror modifier (`MirrorModifierData`, Blender `mesh_mirror.cc`): for each
//! enabled axis (X, Y, Z in that order) append a reflected copy — about the
//! object's origin, or about `mirror_ob` — with reversed winding, merge
//! copies that land within `tolerance` of their source (unless "No Merge"),
//! flip vertex groups (`.L` ↔ `.R`) and optionally the UVs.
//! Not yet: bisect, clipping (an edit-mode option).

use crate::blend::file::types::*;
use super::driver::BModValue;
use super::{BModifier, BlendEvalMesh, EvalCtx};
use glam::{Mat4, Vec3};

const AXIS_X: u32 = 1 << 3;
const MIRROR_U: u32 = 1 << 1;
const MIRROR_V: u32 = 1 << 2;
const VGROUP: u32 = 1 << 6;
const NO_MERGE: u32 = 1 << 7;

pub(crate) struct BMirror {
    flag: u32,
    tolerance: f32,
    mirror_ob: u64,
    uv_offset: [f32; 2],
    uv_offset_copy: [f32; 2],
    /// Vertex-group index → its `.L`/`.R` counterpart.
    group_flip: Vec<u32>,
}

pub(super) fn read(f: &BlendFile, md: &BlendBlock, ob: &BlendBlock, dna: &DnaType) -> Option<Box<dyn BModifier>> {
    let flag = f.int_field(md, "MirrorModifierData", "flag", dna);
    let v2 = |name: &str| {
        f.off("MirrorModifierData", name, dna)
            .and_then(|o| md.data.get(o..o + 8))
            .map_or([0.0; 2], |b| [f.read_f32(&b[0..4]), f.read_f32(&b[4..8])])
    };
    let groups = f.mesh_block_of(ob, dna).map(|me| f.vertex_group_names(ob, me, dna)).unwrap_or_default();
    let group_flip = groups
        .iter()
        .enumerate()
        .map(|(i, g)| groups.iter().position(|o| *o == flip_side(g)).unwrap_or(i) as u32)
        .collect();
    Some(Box::new(BMirror {
        flag,
        tolerance: f.float_field(md, "MirrorModifierData", "tolerance", dna, 0.001),
        mirror_ob: f.ptr(md, f.off("MirrorModifierData", "*mirror_ob", dna)),
        uv_offset: v2("uv_offset[2]"),
        uv_offset_copy: v2("uv_offset_copy[2]"),
        group_flip,
    }))
}

/// `merge_threshold` lands on `tolerance`. `bisect_threshold` has no field
/// here — this reader does not bisect — so it reports `false`.
impl BMirror {
    #[cfg(test)]
    pub(crate) fn for_test() -> Self {
        Self { flag: 0, tolerance: 0.001, mirror_ob: 0, uv_offset: [0.0; 2], uv_offset_copy: [0.0; 2], group_flip: Vec::new() }
    }

    /// `tolerance` is private to this module; the test reads it through here
    /// rather than the field being opened up for one assertion.
    #[cfg(test)]
    pub(crate) fn tolerance_for_test(&self) -> f32 {
        self.tolerance
    }

    fn driven(&mut self, prop: &str, value: BModValue) -> bool {
        match prop {
            "merge_threshold" => self.tolerance = value.as_f32().max(0.0),
            _ => return false,
        }
        true
    }
}
impl BModifier for BMirror {
    fn clone_box(&self) -> Box<dyn BModifier> {
        Box::new(Self {
            flag: self.flag,
            tolerance: self.tolerance,
            mirror_ob: self.mirror_ob,
            uv_offset: self.uv_offset,
            uv_offset_copy: self.uv_offset_copy,
            group_flip: self.group_flip.clone(),
        })
    }

    fn set_driven(&mut self, prop: &str, _component: usize, value: BModValue) -> bool {
        self.driven(prop, value)
    }

    fn apply(&self, mesh: &mut BlendEvalMesh, ctx: &EvalCtx) {
        for axis in 0..3 {
            if self.flag & (AXIS_X << axis) != 0 {
                self.mirror_axis(mesh, axis, ctx);
            }
        }
    }
}

impl BMirror {
    fn mirror_axis(&self, m: &mut BlendEvalMesh, axis: usize, ctx: &EvalCtx) {
        let mut mtx = Mat4::IDENTITY;
        mtx.col_mut(axis)[axis] = -1.0;
        if let Some(mw) = ctx.worlds.get(&self.mirror_ob).filter(|_| self.mirror_ob != 0) {
            let tmp = mw.inverse() * ctx.world;
            mtx = tmp.inverse() * mtx * tmp;
        }
        let n = m.vertices.len() as u32;
        let merge = self.flag & NO_MERGE == 0;
        // Index of each mirrored vertex: merged onto its source or appended.
        let mut map = Vec::with_capacity(n as usize);
        let mut added = Vec::new();
        for (i, v) in m.vertices.iter().enumerate() {
            let p = mtx.transform_point3(Vec3::from(*v));
            if merge && p.distance(Vec3::from(*v)) <= self.tolerance {
                map.push(i as u32);
            } else {
                map.push(n + added.len() as u32);
                added.push((i, p.to_array()));
            }
        }
        let flip_groups = self.flag & VGROUP != 0;
        if m.dverts.len() == n as usize {
            for &(i, _) in &added {
                let dv = m.dverts[i].iter().map(|&(g, w)| {
                    let g2 = if flip_groups { self.group_flip.get(g as usize).copied().unwrap_or(g) } else { g };
                    (g2, w)
                });
                m.dverts.push(dv.collect());
            }
        }
        m.vertices.extend(added.iter().map(|a| a.1));
        let with_uvs = m.loop_uvs.len() >= m.faces.corners();
        let with_colors = m.loop_colors.len() >= m.faces.corners();
        let (faces, mut corner) = (m.faces.len(), 0);
        for fi in 0..faces {
            let f = m.faces[fi].to_vec();
            let k = f.len();
            if merge && f.iter().all(|&v| map[v as usize] == v) {
                corner += k;
                continue; // lies on the mirror plane: the copy would duplicate it
            }
            // Reverse winding: first corner stays, the rest run backwards.
            let order: Vec<usize> = std::iter::once(0).chain((1..k).rev()).collect();
            let copy: Vec<u32> = order.iter().map(|&c| map[f[c] as usize]).collect();
            m.faces.push(&copy);
            if with_uvs {
                for &c in &order {
                    let mut uv = m.loop_uvs[corner + c];
                    if self.flag & MIRROR_U != 0 { uv[0] = 1.0 - uv[0] + self.uv_offset[0]; }
                    if self.flag & MIRROR_V != 0 { uv[1] = 1.0 - uv[1] + self.uv_offset[1]; }
                    m.loop_uvs.push([uv[0] + self.uv_offset_copy[0], uv[1] + self.uv_offset_copy[1]]);
                }
            }
            if with_colors {
                // The copy's corners keep their colours (the loop data of a face goes to its twin).
                for &c in &order {
                    m.loop_colors.push(m.loop_colors[corner + c]);
                }
            }
            let (mat, smooth) = (m.face_materials.get(fi).copied(), m.face_smooth.get(fi).copied());
            m.face_materials.extend(mat);
            m.face_smooth.extend(smooth);
            corner += k;
        }
        // Mirror loose edges: remap through map[], skip edges on the plane.
        let base_loose = m.loose_edges.len();
        for i in 0..base_loose {
            let (a, b) = m.loose_edges[i];
            let ma = map[a as usize];
            let mb = map[b as usize];
            if ma == a && mb == b {
                continue; // both endpoints merged onto the mirror plane
            }
            let (lo, hi) = (ma.min(mb), ma.max(mb));
            if lo != hi {
                m.loose_edges.push((lo, hi));
            }
        }
    }
}

/// Blender `BLI_string_flip_side_name` for the common spellings:
/// `L`/`R` (any case) after or before a `.`, `_`, `-` or space separator,
/// and `left`/`right` words. A trailing `.001` number is kept.
pub(crate) fn flip_side(name: &str) -> String {
    let (base, num) = match name.rsplit_once('.') {
        Some((b, n)) if !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()) => (b, format!(".{n}")),
        _ => (name, String::new()),
    };
    let swap = |c: char| match c { 'L' => 'R', 'R' => 'L', 'l' => 'r', 'r' => 'l', o => o };
    let seps = ['.', '_', '-', ' '];
    let b: Vec<char> = base.chars().collect();
    let n = b.len();
    if n >= 2 && seps.contains(&b[n - 2]) && "LRlr".contains(b[n - 1]) {
        return format!("{}{}{num}", &base[..base.len() - 1], swap(b[n - 1]));
    }
    if n >= 2 && seps.contains(&b[1]) && "LRlr".contains(b[0]) {
        return format!("{}{}{num}", swap(b[0]), &base[1..]);
    }
    for (a, z) in [("Left", "Right"), ("left", "right"), ("LEFT", "RIGHT")] {
        if base.contains(a) {
            return format!("{}{num}", base.replacen(a, z, 1));
        }
        if base.contains(z) {
            return format!("{}{num}", base.replacen(z, a, 1));
        }
    }
    name.to_string()
}
