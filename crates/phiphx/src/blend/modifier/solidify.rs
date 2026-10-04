//! Solidify modifier, simple mode (`SolidifyModifierData`, Blender
//! `MOD_solidify_extrude.cc`): give a surface thickness by adding an inner
//! shell offset along the vertex normals and a rim joining the two along the
//! open boundary.
//!
//! `offset` places the shell relative to the original surface: −1 puts it
//! entirely inside, +1 entirely outside, 0 centres it. Not yet: the complex
//! (non-manifold) mode, per-vertex-group thickness, rim material offsets.

use super::geom::{boundary, vertex_normals};
use super::{BModifier, BlendEvalMesh, EvalCtx};
use crate::blend::file::types::*;
use glam::Vec3;

/// `MOD_SOLIDIFY_RIM` — close the open boundary between the two shells
/// (`DNA_modifier_types.h`: bit 0; bit 1 is `EVEN`, bit 2 `NORMAL_CALC`).
const RIM: u32 = 1 << 0;
/// `MOD_SOLIDIFY_FLIP` — reverse the normals of the result (bit 5).
const FLIP: u32 = 1 << 5;

pub(crate) struct BSolidify {
    pub thickness: f32,
    pub offset: f32,
    pub flag: u32,
}

pub(super) fn read(
    f: &BlendFile,
    md: &BlendBlock,
    _ob: &BlendBlock,
    dna: &DnaType,
) -> Option<Box<dyn BModifier>> {
    Some(Box::new(BSolidify {
        thickness: f.float_field(md, "SolidifyModifierData", "offset", dna, 0.01),
        offset: f.float_field(md, "SolidifyModifierData", "offset_fac", dna, -1.0),
        flag: f.int_field(md, "SolidifyModifierData", "flag", dna),
    }))
}

impl BModifier for BSolidify {
    fn apply(&self, mesh: &mut BlendEvalMesh, _ctx: &EvalCtx) {
        if mesh.vertices.is_empty() || mesh.faces.is_empty() {
            return;
        }
        let normals = vertex_normals(mesh);
        let base_verts = mesh.vertices.len() as u32;
        let base_faces = mesh.faces.len();
        let uvs = mesh.loop_uvs.len() >= mesh.faces.corners();
        let colors = mesh.loop_colors.len() >= mesh.faces.corners();
        // First corner of every face, so a face's UVs can be read back.
        let mut first = Vec::with_capacity(base_faces);
        let mut at = 0usize;
        for f in 0..base_faces {
            first.push(at);
            at += mesh.faces.get(f).len();
        }
        let rim_edges = if self.flag & RIM != 0 { rim(mesh) } else { Vec::new() };

        // `MOD_solidify_extrude.cc`: `ofs_orig = -((1 - offset_fac) / 2) *
        // thickness`, `ofs_new = ofs_orig + thickness`. Which shell takes which
        // is decided by `INIT_VERT_ARRAY_OFFSETS`, not by name: without FLIP
        // the original vertices take the larger offset and the copies the
        // smaller, whatever the sign of the thickness. A negative thickness
        // (a garment offset outward from its base surface) used to leave the
        // originals at the base and push the reversed copies out, so the
        // correctly-wound surface sat exactly on the body underneath it.
        let ofs_orig = -((1.0 - self.offset) * 0.5) * self.thickness;
        let ofs_new = ofs_orig + self.thickness;
        let flip = self.flag & FLIP != 0;
        let (outer, inner) = if (ofs_new >= ofs_orig) != flip {
            (ofs_new, ofs_orig)
        } else {
            (ofs_orig, ofs_new)
        };
        for i in 0..base_verts as usize {
            let v = Vec3::from(mesh.vertices[i]);
            mesh.vertices[i] = (v + normals[i] * outer).into();
        }
        for i in 0..base_verts as usize {
            let v = Vec3::from(mesh.vertices[i]) - normals[i] * outer;
            mesh.vertices.push((v + normals[i] * inner).into());
            let weights = mesh.dverts.get(i).cloned().unwrap_or_default();
            mesh.dverts.push(weights);
        }

        // The copied shell faces the other way, and carries the same UVs
        // (Blender copies the loop data of each face to its twin).
        let mut shell_uvs: Vec<[f32; 2]> = Vec::new();
        let mut shell_colors: Vec<[f32; 3]> = Vec::new();
        for face in 0..base_faces {
            let mut corners: Vec<u32> = mesh.faces.get(face).iter().map(|c| c + base_verts).collect();
            let mut face_uvs: Vec<[f32; 2]> = if uvs {
                mesh.loop_uvs[first[face]..first[face] + corners.len()].to_vec()
            } else {
                Vec::new()
            };
            let mut face_colors: Vec<[f32; 3]> = if colors { mesh.loop_colors[first[face]..first[face] + corners.len()].to_vec() } else { Vec::new() };
            if !flip {
                corners.reverse();
                face_uvs.reverse();
                face_colors.reverse();
            }
            mesh.faces.push(&corners);
            shell_uvs.extend(face_uvs);
            shell_colors.extend(face_colors);
            let material = mesh.face_materials.get(face).copied().unwrap_or(0);
            mesh.face_materials.push(material);
            let smooth = mesh.face_smooth.get(face).copied().unwrap_or(true);
            mesh.face_smooth.push(smooth);
        }
        // The rim: one quad per open edge, wound against the face that owns
        // the edge so it faces out of the gap, taking that face's material,
        // smoothing and the UVs of the edge's two corners.
        for &(face, ca, cb) in &rim_edges {
            let owner = mesh.faces.get(face);
            let (a, b) = (owner[ca], owner[cb]);
            let mut quad = [b, a, a + base_verts, b + base_verts];
            let mut quad_uvs = if uvs {
                let (ua, ub) = (mesh.loop_uvs[first[face] + ca], mesh.loop_uvs[first[face] + cb]);
                vec![ub, ua, ua, ub]
            } else {
                Vec::new()
            };
            let mut quad_colors = if colors {
                let (ca_col, cb_col) = (mesh.loop_colors[first[face] + ca], mesh.loop_colors[first[face] + cb]);
                vec![cb_col, ca_col, ca_col, cb_col]
            } else {
                Vec::new()
            };
            if flip {
                quad.reverse();
                quad_uvs.reverse();
                quad_colors.reverse();
            }
            mesh.faces.push(&quad);
            shell_uvs.extend(quad_uvs);
            shell_colors.extend(quad_colors);
            let material = mesh.face_materials.get(face).copied().unwrap_or(0);
            mesh.face_materials.push(material);
            let smooth = mesh.face_smooth.get(face).copied().unwrap_or(true);
            mesh.face_smooth.push(smooth);
        }
        if flip {
            // FLIP turns the original shell over as well.
            let mut faces = crate::blend::BFaces::default();
            let mut flipped_uvs = Vec::with_capacity(mesh.loop_uvs.len());
            let mut flipped_colors = Vec::with_capacity(mesh.loop_colors.len());
            for f in 0..mesh.faces.len() {
                let mut c = mesh.faces.get(f).to_vec();
                if f < base_faces {
                    c.reverse();
                    if uvs {
                        let mut u = mesh.loop_uvs[first[f]..first[f] + c.len()].to_vec();
                        u.reverse();
                        flipped_uvs.extend(u);
                    }
                    if colors {
                        let mut col = mesh.loop_colors[first[f]..first[f] + c.len()].to_vec();
                        col.reverse();
                        flipped_colors.extend(col);
                    }
                }
                faces.push(&c);
            }
            mesh.faces = faces;
            if uvs {
                mesh.loop_uvs = flipped_uvs;
            }
            if colors {
                mesh.loop_colors = flipped_colors;
            }
        }
        let base_corners = first.last().map_or(0, |f| f + mesh.faces.get(base_faces - 1).len());
        if uvs {
            mesh.loop_uvs.truncate(base_corners);
            mesh.loop_uvs.extend(shell_uvs);
        } else if !mesh.loop_uvs.is_empty() {
            mesh.loop_uvs.clear();
        }
        if colors {
            mesh.loop_colors.truncate(base_corners);
            mesh.loop_colors.extend(shell_colors);
        } else if !mesh.loop_colors.is_empty() {
            mesh.loop_colors.clear();
        }
    }
}

/// Open edges as `(face, corner, next corner)` in the owning face's winding.
fn rim(mesh: &BlendEvalMesh) -> Vec<(usize, usize, usize)> {
    let open: std::collections::HashSet<(u32, u32)> = boundary(mesh).into_iter().collect();
    let mut out = Vec::new();
    for face in 0..mesh.faces.len() {
        let c = mesh.faces.get(face);
        for n in 0..c.len() {
            let (a, b) = (c[n], c[(n + 1) % c.len()]);
            if open.contains(&if a < b { (a, b) } else { (b, a) }) {
                out.push((face, n, (n + 1) % c.len()));
            }
        }
    }
    out
}
