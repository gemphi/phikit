//! The kernels compiled off the render thread.
//!
//! A mode's kernel compiles the first time it is dispatched, and on a
//! driver with no shader cache that can take seconds — so switching from
//! Rendered to Solid used to stall on Workbench's compile with the old
//! image still on screen. The first frame up, the thread hands the rest to
//! a thread of its own (`Tracer::warm`), which holds the device's pipeline
//! lock one kernel at a time and is done long before anyone changes
//! shading.

use phiray::{Gpu, Tracer};
use std::time::Instant;

/// Whether this render thread has already started the compiles.
pub(super) struct BWarmed(bool);

impl BWarmed {
    pub(super) fn new() -> Self {
        Self(false)
    }

    /// Compile the other kernels once, on a thread of its own. Does
    /// nothing on every call after the first.
    pub(super) fn once(&mut self, gpu: &Gpu) {
        if self.0 {
            return;
        }
        self.0 = true;
        let gpu = gpu.clone();
        let _ = std::thread::Builder::new().name("phiray.warm".into()).spawn(move || {
            let at = Instant::now();
            let each: Vec<String> = Tracer::warm(&gpu).into_iter().map(|(name, ms)| format!("{name} {ms:.0}")).collect();
            crate::v2::app::journal::BJournal::note(format!("Tracer: kernels compiled in {:.0}ms ({})", at.elapsed().as_secs_f64() * 1000.0, each.join(", ")));
        });
    }
}
