//! A render job that never stays "running" after its thread died. (A frame
//! larger than the device binds as one image is rendered in bands,
//! `rendered/bands.rs::BBandList`.)

use super::render_job::BJob;
use std::sync::Arc;
use std::sync::atomic::Ordering;

/// Clears the job's `running` flag when the render thread ends, however it
/// ends — a finished frame, a cancel, or a panic in the GPU layer.
pub(crate) struct BRunning(pub(crate) Arc<BJob>);

impl Drop for BRunning {
    fn drop(&mut self) {
        if std::thread::panicking() {
            crate::v2::state::log::push("The render failed on the GPU; see the console".to_string());
            // The slot it was rendering into is finished as it stands.
            let into = self.0.last.load(Ordering::Relaxed);
            if let Some(mut shot) = self.0.slot(into) {
                shot.done = true;
                self.0.put(into, Some(shot));
            }
        }
        self.0.running.store(false, Ordering::Relaxed);
    }
}
