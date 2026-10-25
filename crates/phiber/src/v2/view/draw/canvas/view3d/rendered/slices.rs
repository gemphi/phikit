//! The stepwise render's helpers: how many slices a sample is cut into,
//! the compositor pass over a finished band, and the note a refused render
//! leaves.

/// Why a render produced nothing, said once: the Info editor's report and
/// the session journal both carry it, so a blank Render Result is never the
/// only evidence that something went wrong.
pub(super) fn note_render_refused(why: String) {
    let line = format!("Render failed: {why}");
    crate::v2::state::log::push(line.clone());
    crate::v2::app::journal::BJournal::note(line);
}

/// A slice's intended length in the final render. Short enough that no
/// single dispatch approaches a GPU watchdog (two seconds on Windows),
/// long enough that the drain between slices stays a small share of the
/// work — `Tracer::sample_in` waits for the device after each one.
const RENDER_SLICE_MS: f64 = 120.0;

/// Slices for the very first sample, before anything has been timed.
pub(super) const FIRST_SLICES: u32 = 32;

/// The most a sample is cut into; a slice costs one device drain.
const MOST_SLICES: u32 = 64;

/// How many slices a sample takes, from how long one took.
///
/// Settled from the **second** sample, never the first: `Tracer::warm`
/// compiles the other shading modes on another thread and holds the
/// device's pipeline lock while it does, so the first sample can spend
/// minutes blocked on a lock rather than on the GPU.
pub(super) struct BSliceCut;

impl BSliceCut {
    pub(super) fn from(took: std::time::Duration) -> u32 {
        let ms = took.as_secs_f64() * 1000.0;
        if ms <= 0.0 {
            return FIRST_SLICES;
        }
        ((ms / RENDER_SLICE_MS).ceil() as u32).clamp(1, MOST_SLICES)
    }
}

/// The scene's compositor over the finished render, as Blender's render
/// pipeline runs it: on the scene-linear pixels, before the view transform
/// (`scene/composite.rs`). Nothing happens when the file has no tree, it is
/// switched off, or nothing reaches its output. A band renders as its own
/// image, so each band is composited on its own — which is sound only
/// because every node that runs is per-pixel; a node that read a
/// neighbourhood would see a seam, and those are exactly the ones
/// `BCompTree::declined` refuses.
pub(super) fn composite(gpu: &phiray::Gpu, tracer: &mut phiray::Tracer) {
    use crate::v2::scene::composite::BComposite;
    if let Some(tree) = crate::v2::scene::composite::tree() {
        let run = tracer.composite(gpu, &tree);
        if run.any() {
            crate::v2::app::journal::BJournal::note(run.line());
        }
    }
}
