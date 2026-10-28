//! Cloth (`eModifierType_Cloth` = 22) as the viewport shows it at the
//! file's frame — Blender `MOD_cloth.cc` → `clothModifier_do`
//! (`blenkernel/intern/cloth.cc`) with the point cache read of
//! `BKE_ptcache_read` (`pointcache.cc`), and no solver.
//!
//! `clothModifier_do` at frame `f` with the cache's `startframe..endframe`:
//! before `startframe` the mesh passes through; `f` is clamped to
//! `endframe`; **at** `startframe` the cloth is initialised from the mesh
//! and the mesh passes through; otherwise the cache is read — the exact
//! frame, an interpolation between the cached frames around it, or the
//! cached frame before (or, failing that, after) it as `PTCACHE_READ_OLD`
//! — and `cloth_to_object` writes the cached world-space positions back
//! through the object's inverse world. With nothing cached the solver
//! would step, which is not ported: the mesh passes through. The cache is
//! the file's own (`PointCache.mem_cache`, `cloth_cache.rs`) or the
//! `blendcache_<file>/` folder beside it (`cloth_disk.rs`).
//!
//! `can_simulate` — `framenr == last_frame + 1`, with `last_frame` set to
//! `MINFRAME - 1` by `do_init_cloth` on a fresh open — is false for every
//! frame at or after 1, so an old cached frame is shown rather than
//! simulated past.

use super::cloth_cache::{BClothRead, frame_read};
use super::{BModifier, BlendEvalMesh, EvalCtx};
use crate::blend::file::types::*;
use glam::Vec3;
use std::sync::Arc;

/// `ePointCache_Flag` (`PTCACHE_BAKED` changes nothing here: a baked
/// cache with no frame leaves the mesh, as an unbaked one does without a
/// solver).
pub(super) const PTCACHE_DISK_CACHE: u32 = 1 << 6;
pub(super) const PTCACHE_EXTERNAL: u32 = 1 << 9;

/// What the cache holds for the frame, read once at open.
#[derive(Clone)]
pub(crate) struct BClothMod {
    /// `None` when the mesh passes through (before or at the start frame,
    /// or nothing cached); otherwise the frame to show.
    pub read: Option<Arc<BClothRead>>,
}

/// `Scene.r.cfra` of the file's current scene — the depsgraph's
/// `DEG_get_ctime` at open.
pub(super) fn scene_frame(f: &BlendFile, dna: &DnaType) -> i32 {
    let ps = f.header.pointer_size as usize;
    let sc = f.file_global_curscene(dna).or_else(|| f.blocks.iter().find(|b| b.code_str == "SC"));
    let off = BlendFile::struct_field_offset_exact("Scene", "r", dna, ps)
        .zip(BlendFile::struct_field_offset_exact("RenderData", "cfra", dna, ps))
        .map(|(a, b)| a + b);
    match (sc, off) {
        (Some(sc), Some(o)) if o + 4 <= sc.data.len() => f.read_i32(&sc.data[o..o + 4]),
        _ => 1,
    }
}

pub(super) fn read(f: &BlendFile, md: &BlendBlock, ob: &BlendBlock, dna: &DnaType) -> Option<Box<dyn BModifier>> {
    let Some(pc) = f.block_at(f.ptr(md, f.off("ClothModifierData", "*point_cache", dna))) else {
        return Some(Box::new(BClothMod { read: None }));
    };
    let i = |n: &str| f.uint_field(pc, "PointCache", n, dna).unwrap_or(0) as i32;
    let (start, end) = (i("startframe"), i("endframe"));
    let framenr = scene_frame(f, dna);
    // "simulation is only active during a specific period"
    if framenr < start {
        return Some(Box::new(BClothMod { read: None }));
    }
    let framenr = framenr.min(end);
    // `framenr == startframe`: the cloth is (re)initialised from the mesh
    // and the mesh is left as it is.
    if framenr == start {
        return Some(Box::new(BClothMod { read: None }));
    }
    let read = frame_read(f, pc, ob, dna, framenr, start, end).map(Arc::new);
    Some(Box::new(BClothMod { read }))
}

impl BModifier for BClothMod {
    fn clone_box(&self) -> Box<dyn BModifier> {
        Box::new(self.clone())
    }

    /// `cloth_to_object`: the cached positions are world space.
    fn apply(&self, mesh: &mut BlendEvalMesh, ctx: &EvalCtx) {
        let Some(read) = &self.read else { return };
        let positions = read.positions_at();
        // "Number of points in cache does not match mesh": the shorter
        // count is used, the rest left.
        let n = positions.len().min(mesh.vertices.len());
        let inv = ctx.world.inverse();
        for (v, p) in mesh.vertices.iter_mut().zip(positions.iter()).take(n) {
            *v = inv.transform_point3(Vec3::from(*p)).to_array();
        }
    }
}
