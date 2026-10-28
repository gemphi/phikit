//! The Surface Deform's saved bind: `SurfaceDeformModifierData.verts`, an
//! array of `SDefVert { SDefBind *binds; uint binds_num; uint vertex_idx }`,
//! each bind an `SDefBind { uint *vert_inds; uint verts_num; int mode;
//! float *vert_weights; float normal_dist; float influence }` — written by
//! `MOD_surfacedeform.cc`'s `blend_write` as one block per array: the
//! verts, each vert's binds, each bind's indices and its weights (three
//! floats for the corner-triangle and centroid modes, `verts_num` for an
//! n-gon). `blend_read` drops a bind whose arrays are missing, as this
//! does (`verts_num = 0`, no indices, no weights).
//!
//! The DNA carries the members under their **pre-rename** names —
//! `SDefVert.numbinds`, `SDefBind.numverts`, and on the modifier itself
//! `numverts` (bind count), `num_mesh_verts` and `numpoly` — so every
//! count is looked up under the current name and then the saved one. The
//! CRYSTAL-CAVES and BC-15 files are written that way, and reading only
//! the new names left every bind empty, which showed as a Surface Deform
//! that moved nothing.

use super::surface_deform::{MODE_CENTROID, MODE_CORNER_TRIS};
use crate::blend::file::types::*;

/// One `SDefBind`.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BSdefBind {
    pub verts: Vec<u32>,
    pub weights: Vec<f32>,
    pub mode: u32,
    pub normal_dist: f32,
    pub influence: f32,
}

/// One `SDefVert`: a bound vertex and its binds.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BSdefVert {
    pub vertex_idx: u32,
    pub binds: Vec<BSdefBind>,
}

/// A count the DNA may carry under either name: the current one or the
/// saved one (`numverts` and friends).
pub(super) fn count(f: &BlendFile, block: &BlendBlock, st: &str, now: &str, saved: &str, dna: &DnaType) -> u32 {
    f.uint_field(block, st, now, dna).or_else(|| f.uint_field(block, st, saved, dna)).unwrap_or(0)
}

/// A struct array's record `i` out of a block's bytes.
fn record(data: &[u8], size: usize, i: usize) -> Option<&[u8]> {
    data.get(i * size..(i + 1) * size)
}

/// Every bound vertex of the modifier `md`, empty when it was never bound.
pub(super) fn read_binds(f: &BlendFile, md: &BlendBlock, dna: &DnaType) -> Vec<BSdefVert> {
    let st = "SurfaceDeformModifierData";
    let ps = f.header.pointer_size as usize;
    let le = f.header.little_endian;
    let n = count(f, md, st, "bind_verts_num", "numverts", dna) as usize;
    let Some(verts) = f.block_at(f.ptr(md, f.off(st, "*verts", dna))).filter(|_| n > 0) else { return Vec::new() };
    let (Some(vsize), Some(bsize)) = (BlendFile::struct_size("SDefVert", dna, ps), BlendFile::struct_size("SDefBind", dna, ps)) else {
        return Vec::new();
    };
    let binds_num_at = f.off("SDefVert", "binds_num", dna).or_else(|| f.off("SDefVert", "numbinds", dna));
    let (Some(o_binds), Some(o_binds_num), Some(o_vertex)) = (f.off("SDefVert", "*binds", dna), binds_num_at, f.off("SDefVert", "vertex_idx", dna)) else {
        return Vec::new();
    };
    let bind_num_at = f.off("SDefBind", "verts_num", dna).or_else(|| f.off("SDefBind", "numverts", dna));
    let offs = ["*vert_inds", "mode", "*vert_weights", "normal_dist", "influence"].map(|n| f.off("SDefBind", n, dna));
    let [Some(o_inds), Some(o_mode), Some(o_weights), Some(o_dist), Some(o_infl)] = offs else { return Vec::new() };
    let Some(o_num) = bind_num_at else { return Vec::new() };
    let u32_at = |d: &[u8], o: usize| d.get(o..o + 4).map_or(0, |b| f.read_i32(b).max(0) as u32);
    (0..n)
        .filter_map(|i| {
            let rec = record(&verts.data, vsize, i)?;
            let binds_num = u32_at(rec, o_binds_num) as usize;
            let binds_block = f.block_at(BlendFile::read_ptr(rec, o_binds, ps, le)).filter(|_| binds_num > 0);
            let binds = binds_block
                .map(|bb| {
                    (0..binds_num)
                        .filter_map(|j| {
                            let b = record(&bb.data, bsize, j)?;
                            let verts_num = u32_at(b, o_num) as usize;
                            let mode = u32_at(b, o_mode);
                            let inds = f.block_at(BlendFile::read_ptr(b, o_inds, ps, le));
                            let weights = f.block_at(BlendFile::read_ptr(b, o_weights, ps, le));
                            let want = match mode {
                                MODE_CENTROID | MODE_CORNER_TRIS => 3,
                                _ => verts_num,
                            };
                            // `blend_read`: a bind short of its arrays is emptied.
                            let (Some(inds), Some(weights)) = (inds, weights) else {
                                return Some(BSdefBind { verts: Vec::new(), weights: Vec::new(), mode, normal_dist: 0.0, influence: 0.0 });
                            };
                            let verts: Vec<u32> = inds.data.chunks_exact(4).take(verts_num).map(|c| f.read_i32(c).max(0) as u32).collect();
                            let weights: Vec<f32> = weights.data.chunks_exact(4).take(want).map(|c| f.read_f32(c)).collect();
                            let (verts, weights) = match verts.len() == verts_num && weights.len() == want {
                                true => (verts, weights),
                                false => (Vec::new(), Vec::new()),
                            };
                            Some(BSdefBind { verts, weights, mode, normal_dist: f.f32_at(b, o_dist), influence: f.f32_at(b, o_infl) })
                        })
                        .collect()
                })
                .unwrap_or_default();
            Some(BSdefVert { vertex_idx: u32_at(rec, o_vertex), binds })
        })
        .collect()
}
