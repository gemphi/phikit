//! A render job that never stays "running" after its thread died. (A frame
//! larger than the device binds as one image is rendered in bands,
//! `rendered/still.rs::BBands`.)

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
            if let Ok(mut slot) = self.0.shot.lock()
                && let Some(shot) = slot.as_mut()
            {
                shot.done = true;
            }
            self.0.seq.fetch_add(1, Ordering::Relaxed);
        }
        self.0.running.store(false, Ordering::Relaxed);
    }
}
