//! Edge and vertex creases — the `crease_edge` (edge domain) and
//! `crease_vert` (point domain) float attributes the Subdivision Surface
//! sharpens by (`subdiv_converter_mesh.cc`), with `.edge_verts` naming each
//! edge's two vertices. Blender 4.x keeps them as named `CustomData` layers
//! (`edge_data`, `vert_data`), 5.x in `attribute_storage`, and 3.x as
//! unnamed `CD_CREASE` float layers beside the `MEdge` array
//! (`BKE_mesh_legacy_crease_to_generic`); before those, in
//! `MEdge.crease_legacy` as a byte (`BKE_mesh_legacy_edge_crease_to_layers`).

use crate::blend::file::types::*;
use std::borrow::Cow;

const DOMAIN_POINT: i8 = 0;
const DOMAIN_EDGE: i8 = 1;
/// `CD_MEDGE`, `CD_CREASE`.
const CD_MEDGE: i32 = 3;
const CD_CREASE: i32 = 30;

/// A mesh's creases as the file has them, `0..1`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlendCreases {
    /// `(a, b, crease)` with `a < b`, for every edge with a crease above 0.
    pub edge: Vec<(u32, u32, f32)>,
    /// One per vertex, or empty when no vertex is creased.
    pub vert: Vec<f32>,
}

impl BlendFile {
    /// `me`'s creases (none when the mesh has no crease attributes).
    pub(crate) fn mesh_creases(&self, me: &BlendBlock, dna: &DnaType, verts: usize) -> BlendCreases {
        self.scoped(me, || {
            let edges = self.crease_edges(me, dna);
            let crease: Vec<f32> = match self.crease_layer(me, dna, "crease_edge", DOMAIN_EDGE, 4) {
                Some(bytes) => bytes.chunks_exact(4).map(|c| self.read_f32(c)).collect(),
                None => edges.iter().map(|e| e.2).collect(),
            };
            let edge = edges
                .iter()
                .zip(crease)
                .filter(|&(&(a, b, _), c)| c > 0.0 && a != b)
                .map(|(&(a, b, _), c)| (a.min(b), a.max(b), c.min(1.0)))
                .collect();
            let vert = self
                .crease_layer(me, dna, "crease_vert", DOMAIN_POINT, 4)
                .map(|b| b.chunks_exact(4).take(verts).map(|c| self.read_f32(c).clamp(0.0, 1.0)).collect::<Vec<_>>())
                .filter(|v| v.len() == verts && v.iter().any(|&c| c > 0.0))
                .unwrap_or_default();
            BlendCreases { edge, vert }
        })
    }

    /// Each edge's two vertices (`.edge_verts`, else the legacy `MEdge`
    /// array) — what a bone's custom shape draws (`rig/display.rs`).
    pub(crate) fn mesh_edges(&self, me: &BlendBlock, dna: &DnaType) -> Vec<(u32, u32)> {
        self.crease_edges(me, dna).into_iter().map(|(a, b, _)| (a, b)).collect()
    }

    /// Each edge's two vertices (`.edge_verts`, else the legacy `MEdge`
    /// array), with the crease a legacy `MEdge` carries itself.
    fn crease_edges(&self, me: &BlendBlock, dna: &DnaType) -> Vec<(u32, u32, f32)> {
        if let Some(bytes) = self.crease_layer(me, dna, ".edge_verts", DOMAIN_EDGE, 8) {
            return bytes
                .chunks_exact(8)
                .map(|p| (self.read_i32(&p[0..4]).max(0) as u32, self.read_i32(&p[4..8]).max(0) as u32, 0.0))
                .collect();
        }
        let ps = self.header.pointer_size as usize;
        let field = |f: &str| Self::struct_field_offset_exact("MEdge", f, dna, ps);
        let (Some(cd), Some(size), Some(v1), Some(v2)) =
            (self.cd_offset(dna, ["edge_data", "edata"]), Self::struct_size("MEdge", dna, ps), field("v1"), field("v2"))
        else {
            return Vec::new();
        };
        let crease = field("crease_legacy").or_else(|| field("crease"));
        let Some(bytes) = self.customdata_layer(me, cd, dna, &|t, _| t == CD_MEDGE) else { return Vec::new() };
        bytes
            .chunks_exact(size)
            .map(|r| {
                let c = crease.and_then(|o| r.get(o)).map_or(0.0, |&b| f32::from(b) / 255.0);
                (self.read_i32(&r[v1..v1 + 4]) as u32, self.read_i32(&r[v2..v2 + 4]) as u32, c)
            })
            .collect()
    }

    fn cd_offset(&self, dna: &DnaType, fields: [&str; 2]) -> Option<usize> {
        let ps = self.header.pointer_size as usize;
        fields.iter().find_map(|f| Self::struct_field_offset_exact("Mesh", f, dna, ps))
    }

    /// The layer `name` of `domain`: a named `CustomData` layer (4.x), an
    /// attribute of that domain whose elements are `width` bytes (5.x), or
    /// for the creases the unnamed `CD_CREASE` layer (3.x).
    fn crease_layer(&self, me: &BlendBlock, dna: &DnaType, name: &str, domain: i8, width: usize) -> Option<Cow<'_, [u8]>> {
        let cd = self.cd_offset(dna, if domain == DOMAIN_POINT { ["vert_data", "vdata"] } else { ["edge_data", "edata"] });
        if let Some(bytes) = cd.and_then(|cd| self.read_customdata_named(me, cd, dna, name)) {
            return Some(Cow::Borrowed(bytes));
        }
        if let Some((_, bytes)) = self.mesh_attribute(me, dna, &|n, _, d| n == name && d == domain) {
            return (bytes.len() % width == 0).then_some(bytes);
        }
        let legacy = name.starts_with("crease_").then_some(cd).flatten();
        legacy.and_then(|cd| self.customdata_layer(me, cd, dna, &|t, _| t == CD_CREASE)).map(Cow::Borrowed)
    }
}

/// `find_edges_bad_verts` (`blenkernel/intern/mesh_validate.cc`): the two
/// rules Blender drops an edge by when it validates a mesh — "Edge {} has
/// equal vertex indices {}" and "Edge {} has out of range vertex ({}, {})".
/// True for an edge worth keeping, over a mesh of `verts` vertices.
pub(crate) fn edge_verts_are_good(a: u32, b: u32, verts: usize) -> bool {
    a != b && (a as usize) < verts && (b as usize) < verts
}

impl BlendFile {
    /// Edges that belong to no face — wire-only geometry.
    ///
    /// Reads every edge from the mesh block (`mesh_edges`), collects all
    /// edges the faces reference, and returns the difference: edges present
    /// in the file but absent from any face's corner loop. Each pair has
    /// `a < b`.
    ///
    /// Validated as Blender validates a mesh it loads (`edge_verts_are_good`,
    /// above): an edge whose ends are the same vertex, or whose ends are not
    /// vertices of this mesh, is dropped rather than handed on. Blender drops
    /// them at load, so nothing downstream of it is written to expect one —
    /// and nothing here was either: an edge naming vertex 24 of a 24-vertex
    /// mesh panicked the Mirror modifier building its vertex map
    /// (`modifier/mirror.rs`) and Subsurf blending the group weights of its
    /// midpoint (`modifier/subsurf.rs`). `verts` is the mesh's vertex count,
    /// as `mesh_creases` already takes it.
    pub(crate) fn mesh_loose_edges(
        &self,
        me: &BlendBlock,
        dna: &DnaType,
        faces: &crate::blend::BFaces,
        verts: usize,
    ) -> Vec<(u32, u32)> {
        let all = self.mesh_edges(me, dna);
        if all.is_empty() {
            return Vec::new();
        }
        let mut face_edges: std::collections::HashSet<(u32, u32)> =
            std::collections::HashSet::new();
        for face in faces.iter() {
            for i in 0..face.len() {
                let (a, b) = (face[i], face[(i + 1) % face.len()]);
                face_edges.insert((a.min(b), a.max(b)));
            }
        }
        all.into_iter()
            .filter(|&(a, b)| edge_verts_are_good(a, b, verts))
            .map(|(a, b)| (a.min(b), a.max(b)))
            .filter(|e| !face_edges.contains(e))
            .collect()
    }
}

#[cfg(test)]
mod loose_edge_tests {
    use super::edge_verts_are_good;

    /// Blender's `find_edges_bad_verts`, rule for rule. Before this, an edge
    /// naming vertex 24 of a 24-vertex mesh reached the modifier stack and
    /// panicked the Mirror modifier building its vertex map
    /// (`modifier/mirror.rs`, `the len is 24 but the index is 24`) and
    /// Subsurf blending its midpoint's group weights (`modifier/subsurf.rs`).
    #[test]
    fn an_edge_is_kept_unless_blender_would_have_dropped_it() {
        // In range and distinct: kept, either way round.
        assert!(edge_verts_are_good(0, 23, 24));
        assert!(edge_verts_are_good(23, 0, 24));
        // "Edge {} has out of range vertex": one past the last vertex is the
        // case that panicked, and so is anything beyond it.
        assert!(!edge_verts_are_good(0, 24, 24));
        assert!(!edge_verts_are_good(24, 0, 24));
        assert!(!edge_verts_are_good(0, 3_187_833_393, 24));
        // "Edge {} has equal vertex indices": a loop on one vertex.
        assert!(!edge_verts_are_good(7, 7, 24));
        // A mesh with no vertices keeps no edges.
        assert!(!edge_verts_are_good(0, 0, 0));
    }
}
