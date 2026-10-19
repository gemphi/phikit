//! `scene_fit` — the figures behind a blank Render Result. Split from the
//! paint path so each file sits under the 240-line read cap
//! (`tests/level_structure.rs`).

use super::*;

/// What this scene asks a device for, and whether it fits — the figures
/// behind a blank Render Result (`phiray::Tracer::why_not`). `Ok` is the
/// sizes, `Err` is the reason the device would decline it.
///
/// Public because the answer belongs to whoever is looking at an empty
/// render: the Info report, the session journal, and `examples/scene_sizes`.
pub fn scene_fit(
    objects: &[crate::v2::scene::BObject3D],
    shading: crate::v2::scene::BShading,
    limit: u64,
    chunks: usize,
) -> Result<String, String> {
    let look = BStillLook::measuring(shading);
    let built = scene::build(objects, &look.palette());
    let mib = |b: u64| format!("{:.1} MiB", b as f64 / 1048576.0);
    let sizes = format!(
        "{} objects, {} vertices ({}), {} faces ({}), {} materials, atlas {}; largest array {}",
        objects.len(),
        built.pos.len(),
        mib(built.pos.len() as u64 * 32),
        built.faces.len(),
        mib(built.faces.len() as u64 * 16),
        built.mats.len(),
        mib(built.atlas.words().len() as u64 * 4),
        mib(built.biggest()),
    );
    match phiray::Tracer::why_not(&built, limit, chunks) {
        Some(why) => Err(format!("{sizes}\n  declined: {why}")),
        None => Ok(sizes),
    }
}
