//! Weld modifier (`WeldModifierData`, Blender `MOD_weld.cc`): merge vertices
//! that are closer than `merge_dist`, then drop the faces and corners the
//! merge collapsed.
//!
//! `ALL` considers every pair in range; `CONNECTED` only pairs that already
//! share an edge. Both feed the same union-find, so the result is
//! order-independent the way Blender's is. Not yet: the vertex-group filter
//! and the `loose_edges` option.

use super::{BModifier, BlendEvalMesh, EvalCtx};
use crate::blend::file::types::*;
use glam::Vec3;

/// `MOD_WELD_MODE_CONNECTED`.
const MODE_CONNECTED: u32 = 1;

pub(crate) struct BWeld {
    pub merge_dist: f32,
    pub mode: u32,
}

pub(super) fn read(
    f: &BlendFile,
    md: &BlendBlock,
    _ob: &BlendBlock,
    dna: &DnaType,
) -> Option<Box<dyn BModifier>> {
    Some(Box::new(BWeld {
        merge_dist: f.float_field(md, "WeldModifierData", "merge_dist", dna, 0.001),
        mode: f.int_field(md, "WeldModifierData", "mode", dna),
    }))
}

/// Disjoint sets over vertex indices, path-halving.
struct Groups(Vec<u32>);

impl Groups {
    fn new(n: usize) -> Self {
        Self((0..n as u32).collect())
    }

    fn root(&mut self, mut v: u32) -> u32 {
        while self.0[v as usize] != v {
            self.0[v as usize] = self.0[self.0[v as usize] as usize];
            v = self.0[v as usize];
        }
        v
    }

    fn join(&mut self, a: u32, b: u32) {
        let (a, b) = (self.root(a), self.root(b));
        // The lower index wins, so the survivor is the one Blender keeps.
        if a != b {
            self.0[a.max(b) as usize] = a.min(b);
        }
    }
}

impl BWeld {
    /// Vertex pairs close enough to merge.
    fn pairs(&self, mesh: &BlendEvalMesh) -> Vec<(u32, u32)> {
        let d = self.merge_dist;
        let near = |a: u32, b: u32| {
            Vec3::from(mesh.vertices[a as usize]).distance(Vec3::from(mesh.vertices[b as usize])) <= d
        };
        if self.mode == MODE_CONNECTED {
            let mut out = Vec::new();
            for face in 0..mesh.faces.len() {
                let corners = mesh.faces.get(face);
                for n in 0..corners.len() {
                    let (a, b) = (corners[n], corners[(n + 1) % corners.len()]);
                    if a != b && near(a, b) {
                        out.push((a, b));
                    }
                }
            }
            return out;
        }
        // All pairs, bucketed by a grid of the merge distance so only
        // neighbouring cells are compared.
        let cell = |v: [f32; 3]| {
            let s = |x: f32| (x / d).floor() as i64;
            (s(v[0]), s(v[1]), s(v[2]))
        };
        let mut grid: std::collections::HashMap<(i64, i64, i64), Vec<u32>> = Default::default();
        let mut out = Vec::new();
        for (i, v) in mesh.vertices.iter().enumerate() {
            let (x, y, z) = cell(*v);
            for dx in -1..=1 {
                for dy in -1..=1 {
                    for dz in -1..=1 {
                        for &j in grid.get(&(x + dx, y + dy, z + dz)).into_iter().flatten() {
                            if near(i as u32, j) {
                                out.push((i as u32, j));
                            }
                        }
                    }
                }
            }
            grid.entry((x, y, z)).or_default().push(i as u32);
        }
        out
    }
}

impl BModifier for BWeld {
    fn apply(&self, mesh: &mut BlendEvalMesh, _ctx: &EvalCtx) {
        if self.merge_dist <= 0.0 || mesh.vertices.is_empty() {
            return;
        }
        let mut groups = Groups::new(mesh.vertices.len());
        for (a, b) in self.pairs(mesh) {
            groups.join(a, b);
        }
        // Surviving vertices keep their order; everything else maps onto one.
        let mut moved = vec![u32::MAX; mesh.vertices.len()];
        let (mut verts, mut dverts) = (Vec::new(), Vec::new());
        for i in 0..mesh.vertices.len() {
            let root = groups.root(i as u32) as usize;
            if moved[root] == u32::MAX {
                moved[root] = verts.len() as u32;
                verts.push(mesh.vertices[root]);
                dverts.push(mesh.dverts.get(root).cloned().unwrap_or_default());
            }
            moved[i] = moved[root];
        }
        if verts.len() == mesh.vertices.len() {
            return;
        }
        let has_uvs = !mesh.loop_uvs.is_empty();
        let mut faces = crate::blend::BFaces::default();
        let (mut uvs, mut materials, mut smooth) = (Vec::new(), Vec::new(), Vec::new());
        for face in 0..mesh.faces.len() {
            let start = mesh.faces.start(face);
            let mut kept: Vec<(u32, usize)> = Vec::new();
            for (n, corner) in mesh.faces.get(face).iter().enumerate() {
                let v = moved[*corner as usize];
                // A corner that landed on its neighbour disappears.
                if kept.last().map(|(p, _)| *p) != Some(v) {
                    kept.push((v, start + n));
                }
            }
            if kept.first().map(|(v, _)| *v) == kept.last().map(|(v, _)| *v) && kept.len() > 1 {
                kept.pop();
            }
            if kept.len() < 3 {
                continue;
            }
            faces.push(&kept.iter().map(|(v, _)| *v).collect::<Vec<_>>());
            if has_uvs {
                uvs.extend(kept.iter().map(|(_, c)| mesh.loop_uvs.get(*c).copied().unwrap_or_default()));
            }
            materials.push(mesh.face_materials.get(face).copied().unwrap_or(0));
            smooth.push(mesh.face_smooth.get(face).copied().unwrap_or(true));
        }
        mesh.vertices = verts;
        mesh.dverts = dverts;
        mesh.faces = faces;
        mesh.face_materials = materials;
        mesh.face_smooth = smooth;
        if has_uvs {
            mesh.loop_uvs = uvs;
        }
    }
}
