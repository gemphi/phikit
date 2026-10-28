//! The Mask modifier's new mesh (`MOD_mask.cc`, from `compute_masked_verts`
//! to `add_interpolated_faces_to_new_mesh`): the kept vertices in order, the
//! faces whose every corner is kept, and — with `use_smooth` — a vertex on
//! every edge that crosses the threshold, with the straddling faces cut there.
//!
//! The mesh here has no edge array, so the edges are the face sides in
//! first-seen order and then the loose edges — the order the smooth cut's
//! new vertices are appended in. An edge both of whose vertices survive but
//! whose faces do not stays, as Blender keeps it: a loose edge now.

use super::super::BlendEvalMesh;
use std::collections::{HashMap, HashSet};

/// `use_smooth`'s inputs: each vertex's weight in the mask group and the
/// threshold the cut is placed at.
pub(super) struct Cut<'a> {
    pub weights: &'a [f32],
    pub threshold: f32,
}

impl Cut<'_> {
    /// `get_interp_factor_from_vgroup`: where between `v1` and `v2` the
    /// weight reaches the threshold.
    fn factor(&self, v1: u32, v2: u32) -> f32 {
        let (a, b) = (self.weights[v1 as usize], self.weights[v2 as usize]);
        (self.threshold - a) / (b - a)
    }
}

fn key(a: u32, b: u32) -> (u32, u32) {
    (a.min(b), a.max(b))
}

/// `mesh->edges()` and `corner_edges()`: every face side once, then the
/// loose edges; and per corner, the edge from it to the next corner.
fn edges(mesh: &BlendEvalMesh) -> (Vec<(u32, u32)>, Vec<usize>) {
    let mut index: HashMap<(u32, u32), usize> = HashMap::new();
    let mut edges = Vec::new();
    let mut corner_edge = Vec::with_capacity(mesh.faces.corners());
    for face in mesh.faces.iter() {
        for (k, &a) in face.iter().enumerate() {
            let b = face[(k + 1) % face.len()];
            let e = *index.entry(key(a, b)).or_insert_with(|| {
                edges.push((a, b));
                edges.len() - 1
            });
            corner_edge.push(e);
        }
    }
    for &(a, b) in &mesh.loose_edges {
        index.entry(key(a, b)).or_insert_with(|| {
            edges.push((a, b));
            edges.len() - 1
        });
    }
    (edges, corner_edge)
}

fn lerp<const N: usize>(a: [f32; N], b: [f32; N], fac: f32) -> [f32; N] {
    std::array::from_fn(|i| a[i] * (1.0 - fac) + b[i] * fac)
}

/// `layerInterp_mdeformvert` of two vertices: every group of either,
/// weighted, contributions of zero skipped.
fn mix_dvert(a: &[(u32, f32)], b: &[(u32, f32)], fac: f32) -> Vec<(u32, f32)> {
    let mut out: Vec<(u32, f32)> = Vec::new();
    for (src, t) in [(a, 1.0 - fac), (b, fac)] {
        for &(g, w) in src.iter().filter(|(_, w)| w * t != 0.0) {
            match out.iter_mut().find(|(o, _)| *o == g) {
                Some(slot) => slot.1 += w * t,
                None => out.push((g, w * t)),
            }
        }
    }
    out
}

/// The masked copy of `mesh`: `mask` per vertex, `cut` for `use_smooth`.
pub(super) fn masked(mesh: &BlendEvalMesh, mask: &[bool], cut: Option<&Cut>) -> BlendEvalMesh {
    let n = mesh.vertices.len();
    let (with_dverts, with_vcrease) = (mesh.dverts.len() == n, mesh.vert_creases.len() == n);
    let mut out = BlendEvalMesh::default();
    // `compute_masked_verts` + `copy_masked_verts_to_new_mesh`.
    let mut vmap = vec![u32::MAX; n];
    for i in (0..n).filter(|&i| mask[i]) {
        vmap[i] = out.vertices.len() as u32;
        out.vertices.push(mesh.vertices[i]);
        if with_dverts {
            out.dverts.push(mesh.dverts[i].clone());
        }
        if with_vcrease {
            out.vert_creases.push(mesh.vert_creases[i]);
        }
    }
    let (edges, corner_edge) = edges(mesh);
    // `add_interp_verts_copy_edges_to_new_mesh`: one new vertex per edge with
    // one end in the mask, in edge order, after the kept ones.
    let mut cut_vert = vec![u32::MAX; edges.len()];
    if let Some(c) = cut {
        for (e, &(a, b)) in edges.iter().enumerate().filter(|&(_, &(a, b))| mask[a as usize] != mask[b as usize]) {
            let (ai, bi) = (a as usize, b as usize);
            let fac = c.factor(a, b);
            cut_vert[e] = out.vertices.len() as u32;
            out.vertices.push(lerp(mesh.vertices[ai], mesh.vertices[bi], fac));
            if with_dverts {
                out.dverts.push(mix_dvert(&mesh.dverts[ai], &mesh.dverts[bi], fac));
            }
            if with_vcrease {
                out.vert_creases.push(lerp([mesh.vert_creases[ai]], [mesh.vert_creases[bi]], fac)[0]);
            }
        }
    }
    // The edge each source edge becomes: both ends kept, or cut at its new vertex.
    let dst_edge = |e: usize| -> Option<(u32, u32)> {
        let (a, b) = edges[e];
        match (mask[a as usize], mask[b as usize]) {
            (true, true) => Some(key(vmap[a as usize], vmap[b as usize])),
            (true, false) if cut_vert[e] != u32::MAX => Some(key(cut_vert[e], vmap[a as usize])),
            (false, true) if cut_vert[e] != u32::MAX => Some(key(cut_vert[e], vmap[b as usize])),
            _ => None,
        }
    };
    let corners = mesh.faces.corners();
    let (with_uvs, with_colors) = (mesh.loop_uvs.len() == corners, mesh.loop_colors.len() == corners);
    let mut faces = FaceOut { out: &mut out, with_uvs, with_colors };
    // `computed_masked_faces` + `copy_masked_faces_to_new_mesh`.
    for fi in 0..mesh.faces.len() {
        let (face, s) = (mesh.faces.get(fi), mesh.faces.start(fi));
        if face.iter().all(|&v| mask[v as usize]) {
            let corners: Vec<Corner> = (0..face.len()).map(|k| Corner::copy(mesh, vmap[face[k] as usize], s + k)).collect();
            faces.push(mesh, fi, &corners);
        }
    }
    // `compute_interpolated_faces` + `add_interpolated_faces_to_new_mesh`:
    // each run of kept corners of a straddling face, closed by the two cuts,
    // in ring order from the face's first unkept corner.
    if let Some(c) = cut {
        for fi in 0..mesh.faces.len() {
            let (face, s) = (mesh.faces.get(fi), mesh.faces.start(fi));
            let k = face.len();
            let Some(start) = face.iter().position(|&v| !mask[v as usize]) else { continue };
            let mut sub: Vec<Corner> = Vec::new();
            for j in 0..k {
                let (last, index) = ((start + j) % k, (start + 1 + j) % k);
                let (vl, vi) = (face[last], face[index]);
                let edge_cut = |fac| Corner::mix(mesh, cut_vert[corner_edge[s + last]], s + last, s + index, fac);
                match (mask[vl as usize], mask[vi as usize]) {
                    (false, true) => {
                        sub.push(edge_cut(c.factor(vl, vi)));
                        sub.push(Corner::copy(mesh, vmap[vi as usize], s + index));
                    }
                    (true, false) => {
                        sub.push(edge_cut(c.factor(vl, vi)));
                        faces.push(mesh, fi, &sub);
                        sub.clear();
                    }
                    (true, true) => sub.push(Corner::copy(mesh, vmap[vi as usize], s + index)),
                    (false, false) => {}
                }
            }
        }
    }
    // Edge data: creases ride with the edge they were on (`edge_interp.copy`);
    // the closing edges of a cut are new and have none.
    let index: HashMap<(u32, u32), usize> = edges.iter().enumerate().map(|(e, &(a, b))| (key(a, b), e)).collect();
    out.edge_creases = mesh
        .edge_creases
        .iter()
        .filter_map(|&(a, b, w)| index.get(&key(a, b)).and_then(|&e| dst_edge(e)).map(|(a, b)| (a, b, w)))
        .collect();
    // Loose: every kept or cut edge no face of the result uses.
    let sides: HashSet<(u32, u32)> = out
        .faces
        .iter()
        .flat_map(|f| (0..f.len()).map(move |k| key(f[k], f[(k + 1) % f.len()])))
        .collect();
    let kept = (0..edges.len()).filter(|&e| mask[edges[e].0 as usize] && mask[edges[e].1 as usize]);
    let crossed = (0..edges.len()).filter(|&e| cut_vert[e] != u32::MAX);
    out.loose_edges = kept.chain(crossed).filter_map(dst_edge).filter(|e| !sides.contains(e)).collect();
    out
}

/// One corner of the new mesh: its vertex and its corner data.
struct Corner {
    vert: u32,
    uv: Option<[f32; 2]>,
    color: Option<[f32; 3]>,
}

impl Corner {
    /// `corner_interp.copy`.
    fn copy(mesh: &BlendEvalMesh, vert: u32, c: usize) -> Self {
        Self { vert, uv: mesh.loop_uvs.get(c).copied(), color: mesh.loop_colors.get(c).copied() }
    }

    /// `corner_interp.mix({a, b}, {1 - fac, fac})`.
    fn mix(mesh: &BlendEvalMesh, vert: u32, a: usize, b: usize, fac: f32) -> Self {
        let uv = mesh.loop_uvs.get(a).zip(mesh.loop_uvs.get(b)).map(|(p, q)| lerp(*p, *q, fac));
        let color = mesh.loop_colors.get(a).zip(mesh.loop_colors.get(b)).map(|(p, q)| lerp(*p, *q, fac));
        Self { vert, uv, color }
    }
}

/// The face half of the new mesh, written face by face.
struct FaceOut<'a> {
    out: &'a mut BlendEvalMesh,
    with_uvs: bool,
    with_colors: bool,
}

impl FaceOut<'_> {
    /// One face from source face `fi` (`face_interp.copy`: its material and
    /// smooth flag) with these corners.
    fn push(&mut self, mesh: &BlendEvalMesh, fi: usize, corners: &[Corner]) {
        self.out.faces.push(&corners.iter().map(|c| c.vert).collect::<Vec<_>>());
        if self.with_uvs {
            self.out.loop_uvs.extend(corners.iter().map(|c| c.uv.unwrap_or_default()));
        }
        if self.with_colors {
            self.out.loop_colors.extend(corners.iter().map(|c| c.color.unwrap_or_default()));
        }
        self.out.face_materials.extend(mesh.face_materials.get(fi).copied());
        self.out.face_smooth.extend(mesh.face_smooth.get(fi).copied());
    }
}
