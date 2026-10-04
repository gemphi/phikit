//! Subdivision Surface (`SubsurfModifierData`), evaluated like Blender's
//! OpenSubdiv path for the viewport `levels`: Catmull-Clark (or Simple)
//! refinement, boundaries smoothed (`SUBSURF_BOUNDARY_SMOOTH_ALL`) or with
//! corners kept, then — unless "Use Limit Surface" is off
//! (`eSubsurfModifierFlag_UseRecursiveSubdivision`) — every vertex moved to
//! its limit position. Edge and vertex creases sharpen it as OpenSubdiv's
//! rules do (`crease.rs`), unless "Use Creases" is off. Face materials /
//! smooth flags are inherited; UVs are interpolated linearly. Not yet: UV
//! smoothing.

use crate::blend::file::types::*;
use super::{BModifier, BlendEvalMesh, EvalCtx};
use super::crease::{BCreaseLevel, BRule, BSharpness};
use super::topology::Topology;
use rayon::prelude::*;

const RECURSIVE: u32 = 1 << 6;
/// `eSubsurfModifierFlag_UseCrease`.
const USE_CREASE: u32 = 1 << 4;
/// Guard against runaway levels on huge meshes (faces after refinement).
const MAX_FACES: usize = 8_000_000;

pub(crate) struct BSubsurf {
    pub levels: u32,
    pub simple: bool,
    pub limit: bool,
    pub keep_corners: bool,
    /// "Use Creases": the mesh's edge and vertex creases sharpen it.
    pub use_creases: bool,
    /// `quality`: the level OpenSubdiv isolates sharp features to when it
    /// evaluates the limit surface.
    pub quality: u32,
    /// Each level's adjacency, by the mesh's vertex and face counts: a mesh
    /// subdivided again in another pose (`scene/rig.rs`, every frame) has
    /// the same faces, and building the tables was most of the work.
    topologies: std::sync::Mutex<Vec<((usize, usize), std::sync::Arc<Topology>)>>,
}

impl BSubsurf {
    #[cfg(test)]
    pub(crate) fn new(levels: u32, simple: bool, limit: bool, keep_corners: bool) -> Self {
        Self { levels, simple, limit, keep_corners, use_creases: true, quality: 3, topologies: Default::default() }
    }

    /// One level of refinement under `creases`; the creases of the next.
    fn refine_level(&self, mesh: &mut BlendEvalMesh, creases: &BCreaseLevel) -> BCreaseLevel {
        let t = self.topology(mesh);
        let sharp = BSharpness::of(&t, creases, self.keep_corners);
        let ebase = (mesh.vertices.len() + mesh.faces.len()) as u32;
        refine(mesh, self, &t, &sharp);
        creases.refined(&t, ebase, mesh.vertices.len())
    }

    /// Every vertex's limit position. Blender evaluates the limit surface
    /// from OpenSubdiv's patches, isolated to `quality` levels: a
    /// semi-sharp crease still sharp at the grid's level is refined further,
    /// up to that level, and whatever sharpness it has left there is
    /// dropped (only the infinitely sharp stay). A vertex keeps its index
    /// from level to level, so the grid's vertices are the finer mesh's
    /// first ones.
    fn limit_positions(&self, mesh: &BlendEvalMesh, creases: BCreaseLevel) -> Vec<[f32; 3]> {
        let n = mesh.vertices.len();
        let mut fine: Option<BlendEvalMesh> = None;
        let mut creases = creases;
        let mut level = self.levels;
        while level < self.quality && creases.has_finite() {
            let m = fine.get_or_insert_with(|| BlendEvalMesh { vertices: mesh.vertices.clone(), faces: mesh.faces.clone(), ..Default::default() });
            creases = self.refine_level(m, &creases);
            level += 1;
        }
        if level >= self.quality {
            creases = creases.isolated();
        }
        let m = fine.as_ref().unwrap_or(mesh);
        let t = self.topology(m);
        let sharp = BSharpness::of(&t, &creases, self.keep_corners);
        (0..n).into_par_iter().map(|v| limit(&t, m, v, &sharp)).collect()
    }

    fn topology(&self, m: &BlendEvalMesh) -> std::sync::Arc<Topology> {
        let key = (m.vertices.len(), m.faces.len());
        let mut cache = self.topologies.lock().unwrap_or_else(|e| e.into_inner());
        if let Some((_, t)) = cache.iter().find(|(k, _)| *k == key) {
            return t.clone();
        }
        let t = std::sync::Arc::new(Topology::of(m));
        cache.push((key, t.clone()));
        t
    }
}

pub(super) fn read(f: &BlendFile, md: &BlendBlock, _ob: &BlendBlock, dna: &DnaType) -> Option<Box<dyn BModifier>> {
    let levels = f.int_field(md, "SubsurfModifierData", "levels", dna);
    (levels > 0).then(|| {
        Box::new(BSubsurf {
            levels: levels.min(6),
            simple: f.int_field(md, "SubsurfModifierData", "subdivType", dna) == 1,
            limit: f.int_field(md, "SubsurfModifierData", "flags", dna) & RECURSIVE == 0,
            keep_corners: f.int_field(md, "SubsurfModifierData", "boundary_smooth", dna) == 0,
            use_creases: f.int_field(md, "SubsurfModifierData", "flags", dna) & USE_CREASE != 0,
            quality: match f.int_field(md, "SubsurfModifierData", "quality", dna) { q if (1..=10).contains(&q) => q, _ => 3 },
            topologies: Default::default(),
        }) as Box<dyn BModifier>
    })
}

impl BModifier for BSubsurf {
    fn apply(&self, mesh: &mut BlendEvalMesh, _ctx: &EvalCtx) {
        let adaptive = self.limit && !self.simple;
        let mut creases = BCreaseLevel::read(mesh, self.use_creases);
        let carried = creases.clone();
        for level in 0..self.levels {
            if mesh.faces.len() * 4 > MAX_FACES {
                break;
            }
            if adaptive && level >= self.quality {
                creases = creases.isolated();
            }
            creases = self.refine_level(mesh, &creases);
        }
        if adaptive {
            mesh.vertices = self.limit_positions(mesh, creases);
        }
        let _ = carried;
    }
}

type V3 = [f32; 3];
fn add(a: V3, b: V3) -> V3 { [a[0] + b[0], a[1] + b[1], a[2] + b[2]] }
fn scale(a: V3, s: f32) -> V3 { [a[0] * s, a[1] * s, a[2] * s] }

/// Catmull-Clark limit position (all faces are quads after one level): a
/// corner stays, a crease (a boundary is one) runs to its curve's limit
/// (`1/6, 4/6, 1/6`), anything else to the smooth limit.
fn limit(t: &Topology, m: &BlendEvalMesh, v: usize, sharp: &BSharpness) -> V3 {
    let p = m.vertices[v];
    if t.vert_corners(v).is_empty() {
        return p;
    }
    match sharp.limit_rule(t, v) {
        (BRule::Corner, _) => return p,
        (BRule::Crease, [a, b]) => return BSharpness::weigh(p, 4.0 / 6.0, m.vertices[a as usize], m.vertices[b as usize], 1.0 / 6.0),
        _ => {}
    }
    let n = t.vert_edges(v).len() as f32;
    let e = t.vert_edges(v).iter().fold([0.0; 3], |s, &x| add(s, m.vertices[t.other(x, v) as usize]));
    let f = t.vert_corners(v).iter().fold([0.0; 3], |s, &(fi, c)| {
        let face = &m.faces[fi as usize];
        add(s, m.vertices[face[(c as usize + face.len() / 2) % face.len()] as usize])
    });
    scale(add(add(scale(p, n * n), scale(e, 4.0)), f), 1.0 / (n * (n + 5.0)))
}

fn vertex_point(m: &BlendEvalMesh, t: &Topology, face_pts: &[V3], v: usize, s: &BSubsurf, sharp: &BSharpness) -> V3 {
    let p = m.vertices[v];
    if s.simple || t.vert_corners(v).is_empty() {
        return p;
    }
    sharp.vertex_point(t, m, v, || {
        let n = t.vert_edges(v).len() as f32;
        let q = t.vert_corners(v).iter().fold([0.0; 3], |a, &(f, _)| add(a, face_pts[f as usize]));
        let r = t.vert_edges(v).iter().fold([0.0; 3], |a, &e| add(a, add(p, m.vertices[t.other(e, v) as usize])));
        let (q, r) = (scale(q, 1.0 / t.vert_corners(v).len() as f32), scale(r, 0.5 / n));
        scale(add(add(q, scale(r, 2.0)), scale(p, n - 3.0)), 1.0 / n)
    })
}

/// Equal-weight blend of the listed vertices' group weights.
fn mix(dverts: &[Vec<(u32, f32)>], of: &[u32]) -> Vec<(u32, f32)> {
    let mut acc: Vec<(u32, f32)> = Vec::new();
    let share = 1.0 / of.len().max(1) as f32;
    for &v in of {
        for &(g, w) in &dverts[v as usize] {
            match acc.iter_mut().find(|e| e.0 == g) {
                Some(e) => e.1 += w * share,
                None => acc.push((g, w * share)),
            }
        }
    }
    acc
}

fn refine(m: &mut BlendEvalMesh, s: &BSubsurf, t: &Topology, sharp: &BSharpness) {
    let nv = m.vertices.len();
    let face_pts: Vec<V3> = (0..m.faces.len())
        .into_par_iter()
        .map(|fi| {
            let f = m.faces.get(fi);
            scale(f.iter().fold([0.0; 3], |a, &i| add(a, m.vertices[i as usize])), 1.0 / f.len() as f32)
        })
        .collect();
    let edge_pts: Vec<V3> = (0..t.edges.len())
        .into_par_iter()
        .map(|e| {
            let (a, b) = t.edges[e];
            let mid = scale(add(m.vertices[a as usize], m.vertices[b as usize]), 0.5);
            if s.simple || t.edge_nfaces[e] != 2 {
                return mid;
            }
            let [f0, f1] = t.edge_faces[e];
            let smooth = scale(add(mid, scale(add(face_pts[f0 as usize], face_pts[f1 as usize]), 0.5)), 0.5);
            sharp.edge_point(e, smooth, mid)
        })
        .collect();
    let mut verts: Vec<V3> = (0..nv).into_par_iter().map(|v| vertex_point(m, t, &face_pts, v, s, sharp)).collect();
    // Vertex data follows the same stencil shape: corners keep theirs, face
    // points average the face, edge points the two ends.
    let mut dverts = Vec::new();
    if m.dverts.len() == nv {
        dverts = std::mem::take(&mut m.dverts);
        let src = dverts.clone();
        dverts.extend((0..m.faces.len()).map(|fi| mix(&src, m.faces.get(fi))));
        dverts.extend(t.edges.iter().map(|&(a, b)| mix(&src, &[a, b])));
    }
    let (fbase, ebase) = (nv as u32, (nv + face_pts.len()) as u32);
    verts.extend(face_pts);
    verts.extend(edge_pts);
    let total: usize = m.faces.corners();
    let uvs = m.loop_uvs.len() >= total;
    let colors = m.loop_colors.len() >= total;
    let mut out = BlendEvalMesh { vertices: verts, dverts, ..Default::default() };
    out.faces.reserve(total, 4 * total);
    for (fi, f) in m.faces.iter().enumerate() {
        let (k, c0) = (f.len(), t.face_start[fi]);
        let uv = |i: usize| m.loop_uvs.get(c0 + i % k).copied().unwrap_or([0.0; 2]);
        let center = (0..k).fold([0.0, 0.0], |a, i| [a[0] + uv(i)[0] / k as f32, a[1] + uv(i)[1] / k as f32]);
        let mid = |i: usize| { let (a, b) = (uv(i), uv(i + 1)); [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5] };
        // The colour attribute interpolates as the UVs do (a corner-domain
        // attribute in `BKE_subdiv_mesh`'s loop interpolation).
        let col = |i: usize| m.loop_colors.get(c0 + i % k).copied().unwrap_or([0.0; 3]);
        let col_center = (0..k).fold([0.0; 3], |a, i| std::array::from_fn(|x| a[x] + col(i)[x] / k as f32));
        let col_mid = |i: usize| -> [f32; 3] { let (a, b) = (col(i), col(i + 1)); std::array::from_fn(|x| (a[x] + b[x]) * 0.5) };
        for i in 0..k {
            let (e_next, e_prev) = (t.corner_edge[c0 + i], t.corner_edge[c0 + (i + k - 1) % k]);
            out.faces.push(&[f[i], ebase + e_next, fbase + fi as u32, ebase + e_prev]);
            if uvs {
                out.loop_uvs.extend([uv(i), mid(i), center, mid(i + k - 1)]);
            }
            if colors {
                out.loop_colors.extend([col(i), col_mid(i), col_center, col_mid(i + k - 1)]);
            }
            out.face_materials.extend(m.face_materials.get(fi).copied());
            out.face_smooth.extend(m.face_smooth.get(fi).copied());
        }
    }
    *m = out;
}
