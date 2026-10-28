//! The mesh's edges as a Geometry Nodes field sees them.
//!
//! A Blender mesh stores its edges; an evaluated [`BlendEvalMesh`] stores
//! faces and the loose edges beside them, so the edge domain has to be
//! rebuilt before anything can be evaluated on it. Every face corner
//! contributes its edge (`corner_edges`), loose edges follow, and each
//! edge keeps the faces that use it exactly as
//! `node_geo_input_mesh_edge_angle.cc`'s `create_edge_map` does: `-1` for
//! none, the two faces for a manifold edge, `-2` once a third face arrives
//! (which makes the edge's angle 0, like a boundary edge).
//!
//! The order matters: it is the order the Set Shade Smooth node writes
//! `sharp_edge` in and the order a field's values line up with.

use super::BlendEvalMesh;
use std::collections::HashMap;

/// The mesh's edges, as the tree sees them: every face edge, then the
/// loose ones, with each edge's faces (`create_edge_map`: `-1` none, `-2`
/// more than two).
pub(crate) struct BTopo {
    pub edges: Vec<(u32, u32)>,
    pub index: HashMap<(u32, u32), usize>,
    pub edge_faces: Vec<[i32; 2]>,
}

impl BTopo {
    pub fn of(m: &BlendEvalMesh) -> Self {
        let mut t = Self { edges: Vec::new(), index: HashMap::new(), edge_faces: Vec::new() };
        for (f, face) in m.faces.iter().enumerate() {
            for i in 0..face.len() {
                let e = t.edge(face[i], face[(i + 1) % face.len()]);
                let slot = &mut t.edge_faces[e];
                match slot {
                    [-1, _] => slot[0] = f as i32,
                    [_, -1] => slot[1] = f as i32,
                    _ => *slot = [-2, -2],
                }
            }
        }
        for &(a, b) in &m.loose_edges {
            t.edge(a, b);
        }
        t
    }

    fn edge(&mut self, a: u32, b: u32) -> usize {
        let key = (a.min(b), a.max(b));
        *self.index.entry(key).or_insert_with(|| {
            self.edges.push(key);
            self.edge_faces.push([-1, -1]);
            self.edges.len() - 1
        })
    }

    pub(super) fn face_edges<'a>(&'a self, face: &'a [u32]) -> impl Iterator<Item = usize> + 'a {
        (0..face.len()).filter_map(move |i| {
            let (a, b) = (face[i], face[(i + 1) % face.len()]);
            self.index.get(&(a.min(b), a.max(b))).copied()
        })
    }
}
