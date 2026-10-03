//! The newest view and scene the UI posted, as the render thread reads
//! them between the slices of a sample (`Tracer::sample_in`): a newer one
//! waiting ends the sample early, so a drag is answered within a slice.

/// What was posted last: the view's key and the scene's.
#[derive(Default)]
pub(super) struct BPosted {
    pub view: std::sync::atomic::AtomicU64,
    pub scene: std::sync::atomic::AtomicU64,
}

impl BPosted {
    /// Is a newer view or scene than `view` / `scene` waiting?
    pub fn newer_than(&self, view: u64, scene: u64) -> bool {
        use std::sync::atomic::Ordering::Relaxed;
        self.view.load(Relaxed) != view || self.scene.load(Relaxed) != scene
    }
}
