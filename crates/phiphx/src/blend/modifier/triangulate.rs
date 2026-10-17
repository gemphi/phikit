//! Triangulate modifier (`TriangulateModifierData`, Blender `MOD_triangulate.cc`):
//! replace every face of more than `min_vertices` corners with triangles.
//!
//! The split follows `mesh_calc_tessellation_for_face`, the same routine the
//! viewport already uses to draw n-gons, so a triangulated mesh and an
//! untriangulated one are shaded identically — which is what Blender shows.

use super::{BModifier, BlendEvalMesh, EvalCtx};
use super::driver::BModValue;
use crate::blend::file::types::*;

pub(crate) struct BTriangulate {
    /// Faces with fewer corners than this are left alone (Blender's default
    /// is 4, so quads are kept).
    pub min_vertices: u32,
}

pub(super) fn read(
    f: &BlendFile,
    md: &BlendBlock,
    _ob: &BlendBlock,
    dna: &DnaType,
) -> Option<Box<dyn BModifier>> {
    let declared = f.int_field(md, "TriangulateModifierData", "min_vertices", dna);
    Some(Box::new(BTriangulate {
        // Older files have no `min_vertices`; Blender treats those as 4.
        min_vertices: if declared < 4 { 4 } else { declared },
    }))
}

impl BTriangulate {
    fn driven(&mut self, prop: &str, value: BModValue) -> bool {
        match prop {
            "min_vertices" => self.min_vertices = value.as_f32().max(4.0) as u32,
            _ => return false,
        }
        true
    }
}
impl BModifier for BTriangulate {
    fn clone_box(&self) -> Box<dyn BModifier> {
        Box::new(Self { min_vertices: self.min_vertices })
    }

    fn set_driven(&mut self, prop: &str, _component: usize, value: BModValue) -> bool {
        self.driven(prop, value)
    }

    fn apply(&self, mesh: &mut BlendEvalMesh, _ctx: &EvalCtx) {
        let faces = mesh.faces.len();
        if faces == 0 {
            return;
        }
        let mut out = crate::blend::BFaces::default();
        let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(mesh.loop_uvs.len());
        let mut colors: Vec<[f32; 3]> = Vec::with_capacity(mesh.loop_colors.len());
        let mut materials: Vec<u16> = Vec::with_capacity(faces);
        let mut smooth: Vec<bool> = Vec::with_capacity(faces);
        let has_uvs = !mesh.loop_uvs.is_empty();
        let has_colors = !mesh.loop_colors.is_empty();

        for face in 0..faces {
            let corners = mesh.faces.get(face).to_vec();
            let start = mesh.faces.start(face);
            let material = mesh.face_materials.get(face).copied().unwrap_or(0);
            let flat = mesh.face_smooth.get(face).copied().unwrap_or(true);
            if corners.len() < self.min_vertices as usize {
                out.push(&corners);
                if has_uvs {
                    for n in 0..corners.len() {
                        uvs.push(uv_at(mesh, start + n));
                    }
                }
                if has_colors {
                    colors.extend((0..corners.len()).map(|n| mesh.loop_colors.get(start + n).copied().unwrap_or([0.0; 3])));
                }
                materials.push(material);
                smooth.push(flat);
                continue;
            }
            // The same split the renderer uses, so the shape does not change.
            let ring: Vec<[f32; 3]> = corners
                .iter()
                .map(|c| mesh.vertices.get(*c as usize).copied().unwrap_or_default())
                .collect();
            let tris = crate::scene::polyfill::triangulate(&ring);
            for tri in tris {
                out.push(&[corners[tri[0]], corners[tri[1]], corners[tri[2]]]);
                if has_uvs {
                    for n in tri {
                        uvs.push(uv_at(mesh, start + n));
                    }
                }
                if has_colors {
                    colors.extend(tri.iter().map(|&n| mesh.loop_colors.get(start + n).copied().unwrap_or([0.0; 3])));
                }
                materials.push(material);
                smooth.push(flat);
            }
        }
        mesh.faces = out;
        if has_uvs {
            mesh.loop_uvs = uvs;
        }
        if has_colors {
            mesh.loop_colors = colors;
        }
        mesh.face_materials = materials;
        mesh.face_smooth = smooth;
    }
}

fn uv_at(mesh: &BlendEvalMesh, corner: usize) -> [f32; 2] {
    mesh.loop_uvs.get(corner).copied().unwrap_or([0.0, 0.0])
}
