//! The Subdivision Surface's creases, as OpenSubdiv's Catmark scheme
//! applies them (`sdc/crease.cpp`, `Scheme::ComputeEdgeVertexMask` /
//! `ComputeVertexVertexMask`), with the options Blender gives it:
//! uniform creasing (`CREASE_UNIFORM`, `topology_refiner_factory.cc`) and a
//! crease `c` read as sharpness `c² · 10` (`BKE_subdiv_crease_to_sharpness`).
//!
//! An edge's sharpness decides its edge point — the smooth average, the
//! midpoint once it is 1 or more, a blend between for a fraction. A
//! vertex's rule comes from its own sharpness and how many sharp edges meet
//! it (smooth, dart, crease, corner). Each level takes one off every
//! sharpness, and where that changes a vertex's rule the two masks are
//! blended by how sharp the edges and vertex that went smooth were.
//! Boundary and non-manifold edges are infinitely sharp, and so, when the
//! modifier keeps corners, is a boundary vertex with one face.

use super::BlendEvalMesh;
use super::topology::Topology;

/// OpenSubdiv's `SHARPNESS_INFINITE`.
const INFINITE: f32 = 10.0;

type V3 = [f32; 3];

/// `Crease::Rule`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BRule {
    Smooth,
    Dart,
    Crease,
    Corner,
}

/// Creases carried from one level to the next: each creased edge by its
/// ends, with the crease the file gave it and its sharpness at this level.
#[derive(Clone, Debug, Default)]
pub(crate) struct BCreaseLevel {
    pub edge: Vec<(u32, u32, f32, f32)>,
    /// Vertex sharpness, one per vertex, or empty.
    pub vert: Vec<f32>,
}

/// One level's sharpness, laid out on its topology.
pub(crate) struct BSharpness {
    edge: Vec<f32>,
    vert: Vec<f32>,
}

/// How a vertex is refined: its rule now, the rule its child takes, and
/// the sharp neighbours each crease mask reads.
struct BVertexRule {
    parent: (BRule, [u32; 2]),
    child: (BRule, [u32; 2]),
    /// How much of the parent's mask is kept when the rule changes.
    weight: f32,
}

impl BCreaseLevel {
    /// The mesh's creases at the first level; with `use_creases` off they
    /// are carried along but sharpen nothing.
    pub fn read(m: &BlendEvalMesh, use_creases: bool) -> Self {
        let sharp = |c: f32| if use_creases { c * c * INFINITE } else { 0.0 };
        let vert = match use_creases && m.vert_creases.len() == m.vertices.len() {
            true => m.vert_creases.iter().map(|&c| sharp(c)).collect(),
            false => Vec::new(),
        };
        Self { edge: m.edge_creases.iter().map(|&(a, b, c)| (a, b, c, sharp(c))).collect(), vert }
    }

    /// `SubdivideUniformSharpness`: one less, nothing under 1, infinite kept.
    pub fn decrement(s: f32) -> f32 {
        if s >= INFINITE {
            INFINITE
        } else if s > 1.0 {
            s - 1.0
        } else {
            0.0
        }
    }

    /// The creases of the level refined from this one: each creased edge's
    /// two halves, meeting at its edge point, one step less sharp; each
    /// vertex one step less sharp, the new vertices smooth.
    pub fn refined(&self, t: &Topology, ebase: u32, verts: usize) -> Self {
        let mut edge = Vec::new();
        for &(a, b, c, s) in &self.edge {
            let child = Self::decrement(s);
            let Some(e) = t.edge_of(a, b) else { continue };
            let mid = ebase + e as u32;
            edge.push((a.min(mid), a.max(mid), c, child));
            edge.push((b.min(mid), b.max(mid), c, child));
        }
        let vert = match self.vert.iter().any(|&s| Self::decrement(s) > 0.0) {
            true => self.vert.iter().map(|&s| Self::decrement(s)).chain(std::iter::repeat(0.0)).take(verts).collect(),
            false => Vec::new(),
        };
        Self { edge, vert }
    }

    /// Any sharpness above nothing and short of infinite?
    pub fn has_finite(&self) -> bool {
        self.edge.iter().map(|e| e.3).chain(self.vert.iter().copied()).any(|s| s > 0.0 && s < INFINITE)
    }

    /// Only the infinitely sharp kept: what OpenSubdiv's patches honour
    /// past the level they isolate features to.
    pub fn isolated(&self) -> Self {
        let keep = |s: f32| if s >= INFINITE { s } else { 0.0 };
        let vert = if self.vert.iter().any(|&s| s >= INFINITE) { self.vert.iter().map(|&s| keep(s)).collect() } else { Vec::new() };
        Self { edge: self.edge.iter().map(|&(a, b, c, s)| (a, b, c, keep(s))).collect(), vert }
    }

}

impl BSharpness {
    pub fn of(t: &Topology, level: &BCreaseLevel, keep_corners: bool) -> Self {
        let mut edge: Vec<f32> = t.edge_nfaces.iter().map(|&n| if n != 2 { INFINITE } else { 0.0 }).collect();
        for &(a, b, _, s) in &level.edge {
            if let Some(e) = t.edge_of(a, b) {
                edge[e] = edge[e].max(s);
            }
        }
        let n = t.vert_count();
        let mut vert = if level.vert.len() == n { level.vert.clone() } else { vec![0.0; n] };
        if keep_corners {
            for (v, s) in vert.iter_mut().enumerate() {
                if t.vert_corners(v).len() == 1 {
                    *s = INFINITE;
                }
            }
        }
        Self { edge, vert }
    }

    /// Edge `e`'s point from its smooth average and its midpoint.
    pub fn edge_point(&self, e: usize, smooth: V3, mid: V3) -> V3 {
        let s = self.edge[e];
        if s <= 0.0 {
            smooth
        } else if s >= 1.0 {
            mid
        } else {
            Self::lerp(smooth, mid, s)
        }
    }

    /// `DetermineVertexVertexRule` over the given sharpnesses, and the other
    /// ends of the first two sharp edges.
    fn rule(t: &Topology, v: usize, vertex: f32, edge: impl Fn(u32) -> f32) -> (BRule, [u32; 2]) {
        let mut pair = [v as u32; 2];
        let mut count = 0;
        for &e in t.vert_edges(v) {
            if edge(e) > 0.0 {
                if count < 2 {
                    pair[count] = t.other(e, v);
                }
                count += 1;
            }
        }
        let rule = match (vertex > 0.0, count) {
            (true, _) => BRule::Corner,
            (_, 0) => BRule::Smooth,
            (_, 1) => BRule::Dart,
            (_, 2) => BRule::Crease,
            _ => BRule::Corner,
        };
        (rule, pair)
    }

    /// The rule at this level, for the limit.
    pub fn limit_rule(&self, t: &Topology, v: usize) -> (BRule, [u32; 2]) {
        Self::rule(t, v, self.vert[v], |e| self.edge[e as usize])
    }

    fn vertex_rule(&self, t: &Topology, v: usize) -> BVertexRule {
        let parent = self.limit_rule(t, v);
        if parent.0 == BRule::Smooth {
            return BVertexRule { parent, child: parent, weight: 0.0 };
        }
        let down = BCreaseLevel::decrement;
        let child = Self::rule(t, v, down(self.vert[v]), |e| down(self.edge[e as usize]));
        // `ComputeFractionalWeightAtVertex`: the mean sharpness of what went
        // from sharp to smooth, at most 1.
        let mut went = Vec::new();
        if self.vert[v] > 0.0 && down(self.vert[v]) <= 0.0 {
            went.push(self.vert[v]);
        }
        went.extend(t.vert_edges(v).iter().map(|&e| self.edge[e as usize]).filter(|&s| s > 0.0 && down(s) <= 0.0));
        let weight = if went.is_empty() { 0.0 } else { (went.iter().sum::<f32>() / went.len() as f32).min(1.0) };
        BVertexRule { parent, child, weight }
    }

    /// Vertex `v`'s refined position, given its smooth Catmull-Clark point.
    pub fn vertex_point(&self, t: &Topology, m: &BlendEvalMesh, v: usize, smooth: impl Fn() -> V3) -> V3 {
        let r = self.vertex_rule(t, v);
        let mask = |(rule, [a, b]): (BRule, [u32; 2])| match rule {
            BRule::Smooth | BRule::Dart => smooth(),
            BRule::Crease => Self::weigh(m.vertices[v], 0.75, m.vertices[a as usize], m.vertices[b as usize], 0.125),
            BRule::Corner => m.vertices[v],
        };
        if r.parent.0 == r.child.0 {
            return mask(r.parent);
        }
        let child = mask(r.child);
        if r.weight > 0.0 { Self::lerp(child, mask(r.parent), r.weight) } else { child }
    }

    /// `w_p · p + w_e · (a + b)`.
    pub fn weigh(p: V3, w_p: f32, a: V3, b: V3, w_e: f32) -> V3 {
        std::array::from_fn(|i| p[i] * w_p + (a[i] + b[i]) * w_e)
    }

    fn lerp(from: V3, to: V3, w: f32) -> V3 {
        std::array::from_fn(|i| from[i] + (to[i] - from[i]) * w)
    }
}
