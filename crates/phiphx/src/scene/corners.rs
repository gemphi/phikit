//! Shading normals per triangle corner, stored as an id per corner into a
//! table of unique normals (4 bytes a corner instead of 12), and the
//! corners' colours when the mesh has a colour attribute.

use super::geometry::{normalize, smooth_vertex_normals, tri_face_normal};
use std::collections::HashMap;

/// `normal(c)` for corner `c` (`indices[c]`); id 0 is the flat marker (zero normal).
/// Empty when every face is flat.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SceneCorners {
    /// One id per corner (`len == indices.len()`), or none.
    pub ids: Vec<u32>,
    /// Unique unit normals; `normals[0] == [0; 3]` when not empty.
    pub normals: Vec<[f32; 3]>,
    /// The active colour attribute at each corner, scene-linear (`len ==
    /// indices.len()`), or none — Solid's Color ▸ Attribute.
    pub colors: Vec<[f32; 3]>,
}

impl SceneCorners {
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    pub fn len(&self) -> usize {
        self.ids.len()
    }

    /// Normal id of corner `c` (0 = flat).
    pub fn id(&self, c: usize) -> u32 {
        self.ids.get(c).copied().unwrap_or(0)
    }

    /// Normal of corner `c`; `[0; 3]` = flat.
    pub fn normal(&self, c: usize) -> [f32; 3] {
        self.normals.get(self.id(c) as usize).copied().unwrap_or([0.0; 3])
    }

    /// Every corner's normal (tests, small meshes).
    pub fn iter(&self) -> impl Iterator<Item = [f32; 3]> + '_ {
        (0..self.len()).map(|c| self.normal(c))
    }

    /// Split normals when saved (de-duplicated), else smooth vertex normals on
    /// smooth faces (Blender `mesh::normals_calc_verts`); flat faces get id 0.
    pub fn build(
        vertices: &[[f32; 3]],
        indices: &[u32],
        loop_normals: &[[f32; 3]],
        tri_loops: &[[usize; 3]],
        tri_smooth: &[bool],
    ) -> Self {
        if !loop_normals.is_empty() {
            let mut normals = vec![[0.0; 3]];
            let mut seen: HashMap<[u32; 3], u32> = HashMap::from([([0; 3], 0)]);
            let ids = tri_loops
                .iter()
                .flatten()
                .map(|&l| {
                    let n = loop_normals.get(l).copied().unwrap_or([0.0; 3]);
                    *seen.entry(n.map(f32::to_bits)).or_insert_with(|| {
                        normals.push(n);
                        normals.len() as u32 - 1
                    })
                })
                .collect();
            return Self { ids, normals, colors: Vec::new() };
        }
        if !tri_smooth.iter().any(|s| *s) {
            return Self::default();
        }
        let face: Vec<[f32; 3]> = (0..indices.len() / 3).map(|t| tri_face_normal(vertices, indices, t)).collect();
        let accum = smooth_vertex_normals(vertices, indices, &face, tri_smooth);
        let mut normals = vec![[0.0; 3]];
        normals.extend(accum.iter().map(|n| normalize(*n).unwrap_or([0.0; 3])));
        let ids = (0..indices.len())
            .map(|c| match tri_smooth.get(c / 3) {
                Some(true) => indices[c] + 1,
                _ => 0,
            })
            .collect();
        Self { ids, normals, colors: Vec::new() }
    }
}
