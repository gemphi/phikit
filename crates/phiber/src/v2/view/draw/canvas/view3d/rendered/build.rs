//! The tracer built for a scene, and what the build cost, for the journal.

use super::super::mesh::BMeshPalette;
use super::scene;
use crate::v2::scene::BObject3D;
use phiray::{Gpu, Tracer};
use std::sync::Mutex;
use std::time::Instant;

/// The tracer for `objects`, built afresh; a scene the device cannot hold
/// is listed as refused (the painter falls back). The journal says what it
/// cost: the triangles and the atlas, then the tree and its upload — and
/// whether Solid and Wireframe rasterize their primary hits (Workbench's
/// prepass, on hardware) or trace them (a software adapter).
pub(super) fn built(gpu: &Gpu, key: u64, objects: &[BObject3D], palette: &BMeshPalette, refused: &Mutex<Vec<u64>>) -> Option<Tracer> {
    let at = Instant::now();
    let scene = scene::build(objects, palette);
    let (faces, atlas_mb, built_ms) = (scene.len(), scene.atlas.bytes() as f64 / 1e6, at.elapsed().as_secs_f64() * 1000.0);
    let tracer = Tracer::from_scene(gpu, scene);
    crate::v2::app::journal::BJournal::note(format!(
        "Tracer: scene built — {faces} triangles, {atlas_mb:.0} MB of images: triangles {built_ms:.0}ms, tree and upload {:.0}ms{}",
        at.elapsed().as_secs_f64() * 1000.0 - built_ms,
        match &tracer {
            None => " — refused: more than the device holds",
            Some(t) if t.raster() => "; Solid and Wireframe rasterize their primary hits",
            Some(_) => "; Solid and Wireframe trace their primary hits (software adapter)",
        }
    ));
    if tracer.is_none()
        && let Ok(mut r) = refused.lock()
    {
        r.push(key);
    }
    tracer
}
