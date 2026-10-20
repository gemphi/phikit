//! Handing the scene to the render thread, and why the GPU may not be
//! drawing this frame.

use super::super::mesh::BMeshPalette;
use super::key::BRayKey;
use super::{BRayJob, BRayWorker, STATE};
use crate::v2::scene::BObject3D;

/// Why the GPU is not drawing this frame, when it is not.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::v2::view::draw::canvas) enum BRayDeclined {
    /// No adapter at all: the CPU painter is the only renderer.
    NoGpu,
    /// The scene exceeds what this device can bind, even chunked.
    TooLarge,
}

/// Hand `scene` to the render thread when it changed. `Err` → the GPU is not
/// drawing, and says why, so the fallback can decide what it can honestly do.
pub(in crate::v2::view::draw::canvas::view3d) fn prepare(scene: &std::rc::Rc<Vec<BObject3D>>, palette: &BMeshPalette) -> Result<(), BRayDeclined> {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        // Another render device was picked: the old worker's thread ends with
        // its channel, and the new one builds the scene again.
        if s.worker.as_ref().is_some_and(|w| w.as_ref().is_some_and(|w| w.generation != phiray::Gpu::generation())) {
            (s.worker, s.scene) = (None, 0);
        }
        s.worker.get_or_insert_with(BRayWorker::start);
        let key = BRayKey::of_list(scene, palette);
        let fresh = key != s.scene;
        let Some(Some(w)) = s.worker.as_ref() else { return Err(BRayDeclined::NoGpu) };
        if fresh {
            w.send(BRayJob::Scene(key, BRayKey::shape(scene, palette), scene.to_vec(), *palette));
        }
        let refused = w.refused(key);
        if fresh {
            s.scene = key;
            s.repost();
        }
        if refused { Err(BRayDeclined::TooLarge) } else { Ok(()) }
    })
}
