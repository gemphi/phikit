//! Multiresolution (`MultiresModifierData`), as the viewport evaluates it
//! outside sculpt mode (Blender `MOD_multires.cc`'s `modify_mesh` →
//! `multires_as_mesh`): the mesh subdivided by Catmull-Clark to the viewport
//! level `lvl` on OpenSubdiv's limit surface (`BKE_multires_subdiv_settings_init`:
//! never simple, always adaptive at `quality`, creases and boundary as set),
//! then every vertex displaced by the saved `MDisps` grids in the tangent
//! space of that surface (`subdiv_displacement_multires.cc`).
//!
//! The pieces: `multires_read.rs` (the `CD_MDISPS` layer),
//! `multires_grid.rs` (PTEX faces and corner grids over the refined cage),
//! `multires_tangent.rs` / `multires_limit.rs` (limit derivatives),
//! `multires_disp.rs` (`eval_displacement`), `multires_mesh.rs` (which points
//! each vertex is evaluated at). Not ported: sculpt mode's CCG grids, the
//! Simplify cap on the level, external displacement files, custom normals,
//! and UV smoothing (UVs are interpolated as `subsurf.rs` does).

use super::subsurf::BSubsurf;
use super::topology::Topology;
use super::{BModifier, BlendEvalMesh, EvalCtx};
use crate::blend::file::types::*;
use std::sync::Arc;

#[path = "multires_disp.rs"]
mod disp;
#[path = "multires_grid.rs"]
mod grid;
#[path = "multires_limit.rs"]
mod limit;
#[path = "multires_mesh.rs"]
mod mesh;
#[path = "multires_read.rs"]
mod read;
#[path = "multires_tangent.rs"]
mod tangent;

/// `eMultiresModifierFlag_UseCrease`.
const USE_CREASE: u32 = 1 << 2;
/// `SUBSURF_BOUNDARY_SMOOTH_PRESERVE_CORNERS`.
const PRESERVE_CORNERS: u32 = 1;

pub(crate) struct BMultires {
    /// Viewport level (`lvl`) and the level the grids are saved at (`totlvl`).
    pub lvl: u32,
    pub totlvl: u32,
    pub quality: u32,
    pub use_creases: bool,
    pub keep_corners: bool,
    /// The object's `CD_MDISPS` grids, per base-mesh corner; `None` when the
    /// mesh has no such layer (dynamic topology), which subdivides only.
    pub mdisps: Option<Arc<read::BMDisps>>,
    /// The scene's Simplify cap (`BlendFile::simplify_subsurf`).
    pub simplify: Option<u32>,
}

pub(super) fn read(f: &BlendFile, md: &BlendBlock, ob: &BlendBlock, dna: &DnaType) -> Option<Box<dyn BModifier>> {
    let field = |name: &str| f.int_field(md, "MultiresModifierData", name, dna);
    let mdisps = f.mesh_block_of(ob, dna).and_then(|me| read::read_mdisps(f, me, dna)).map(Arc::new);
    Some(Box::new(BMultires {
        lvl: field("lvl"),
        totlvl: field("totlvl"),
        quality: match field("quality") {
            q if (1..=10).contains(&q) => q,
            _ => 4,
        },
        use_creases: field("flags") & USE_CREASE != 0,
        keep_corners: field("boundary_smooth") == PRESERVE_CORNERS,
        mdisps,
        simplify: f.simplify_subsurf(dna),
    }))
}

impl BMultires {
    /// `multires_get_level` for the viewport, outside sculpt mode:
    /// `get_render_subsurf_level(&scene->r, lvl, false)`, so under the
    /// scene's Simplify cap.
    fn level(&self) -> u32 {
        let lvl = match self.simplify {
            Some(cap) => self.lvl.min(cap),
            None => self.lvl,
        };
        lvl.min(self.totlvl).min(6)
    }

    fn subsurf(&self, limit: bool) -> BSubsurf {
        let mut s = BSubsurf::new(self.level(), false, limit, self.keep_corners);
        s.use_creases = self.use_creases;
        s.quality = self.quality;
        s
    }
}

impl BModifier for BMultires {
    fn clone_box(&self) -> Box<dyn BModifier> {
        Box::new(Self { mdisps: self.mdisps.clone(), ..*self })
    }

    /// `modify_mesh` returns the mesh untouched at level 0
    /// (`mesh_settings.resolution < 3`).
    fn disabled(&self) -> bool {
        self.level() == 0
    }

    fn apply(&self, mesh: &mut BlendEvalMesh, ctx: &EvalCtx) {
        let level = self.level();
        if level == 0 {
            return;
        }
        let coarse = mesh.faces.clone();
        let mut cage = BlendEvalMesh {
            vertices: mesh.vertices.clone(),
            faces: mesh.faces.clone(),
            edge_creases: mesh.edge_creases.clone(),
            vert_creases: mesh.vert_creases.clone(),
            loose_edges: mesh.loose_edges.clone(),
            ..Default::default()
        };
        self.subsurf(false).apply(&mut cage, ctx);
        self.subsurf(true).apply(mesh, ctx);
        // `displacement_attach_from_multires`: nothing to add without the
        // layer, or with one that no longer matches the corners it reaches.
        let Some(mdisps) = self.mdisps.as_deref().filter(|m| m.len() == coarse.corners()) else { return };
        let Some(grids) = grid::BGrids::build(&coarse, &cage.faces, level) else { return };
        if cage.vertices.len() != mesh.vertices.len() {
            return;
        }
        let topology = Topology::of(&cage);
        let limit = limit::BLimit {
            grids: &grids,
            cage: &cage,
            topology: &topology,
            limit: &mesh.vertices,
            level,
            quality: self.quality,
            keep_corners: self.keep_corners,
        };
        let disp = disp::BDisplacement {
            grids: &grids,
            limit: &limit,
            mdisps,
            grid_size: (1usize << self.totlvl.max(1).saturating_sub(1).min(30)) + 1,
        };
        let displaced = mesh::displace(&disp, &grids.face_size, level, &mesh.vertices);
        mesh.vertices = displaced;
    }
}
