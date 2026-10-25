//! The render thread's pacing and its account: how a sample is cut into
//! slices so a newer view is taken within one (`BSlicer`), and what the
//! tracer did lately, written to the session journal (`BTally`).

use super::coarse::BCoarse;
use phiray::Mode;
use std::time::Duration;
use web_time::Instant;

/// How a sample is cut (`Tracer::sample_in`): into enough slices that one
/// takes about [`SLICE_MS`], from how long the last sample at this size took
/// — so a view posted while a heavy frame traces waits one slice, not the
/// frame. A coarse sample is quick and goes whole.
pub(super) struct BSlicer {
    /// The last full-size sample's time per pixel, per mode (`Mode::ALL`'s
    /// order): Solid's frame says nothing about Rendered's.
    per_pixel_ns: [f64; 4],
}

fn slot(mode: Mode) -> usize {
    Mode::ALL.iter().position(|m| *m == mode).unwrap_or(0)
}

/// A slice's intended length.
const SLICE_MS: f64 = 25.0;

impl BSlicer {
    pub fn new() -> Self {
        Self { per_pixel_ns: [0.0; 4] }
    }

    /// How long a full-size `w × h` sample of `mode` should take, from the
    /// last one; nothing until one has run.
    pub fn predicted_ms(&self, mode: Mode, w: u32, h: u32) -> Option<f64> {
        let per = self.per_pixel_ns[slot(mode)];
        (per > 0.0).then(|| per * (w as f64 * h as f64) / 1e6)
    }

    pub fn slices(&self, mode: Mode, w: u32, h: u32, coarse: BCoarse) -> u32 {
        match self.predicted_ms(mode, w, h) {
            Some(ms) if coarse.full() => ((ms / SLICE_MS).ceil() as u32).clamp(1, 16),
            _ => 1,
        }
    }

    pub fn took(&mut self, mode: Mode, w: u32, h: u32, coarse: BCoarse, elapsed: Duration) {
        if coarse.full() {
            self.per_pixel_ns[slot(mode)] = elapsed.as_secs_f64() * 1e9 / (w as f64 * h as f64).max(1.0);
        }
    }
}

/// The tracer's own account for the journal, every [`BTally::EVERY`]
/// samples: how many, how long, at what size, how many were abandoned.
pub(super) struct BTally {
    since: Instant,
    samples: u32,
    pub abandoned: u32,
    traced: Duration,
    read: Duration,
    slowest: Duration,
    full: u32,
}

impl BTally {
    const EVERY: u32 = 64;

    pub fn new() -> Self {
        Self { since: Instant::now(), samples: 0, abandoned: 0, traced: Duration::ZERO, read: Duration::ZERO, slowest: Duration::ZERO, full: 0 }
    }

    pub fn sample(&mut self, traced: Duration, read: Duration, full: bool, w: u32, h: u32, mode: Mode) {
        self.samples += 1;
        self.traced += traced;
        self.read += read;
        self.slowest = self.slowest.max(traced);
        self.full += u32::from(full);
        if self.samples >= Self::EVERY {
            let ms = |d: Duration| d.as_secs_f64() * 1000.0;
            crate::v2::app::journal::BJournal::note(format!(
                "Tracer: {} samples in {:.1}s ({mode:?}, {w}×{h} at full size, {} of them full) — trace {:.0}ms average, slowest {:.0}ms, readback {:.0}ms average, {} abandoned for a newer view",
                self.samples,
                self.since.elapsed().as_secs_f64(),
                self.full,
                ms(self.traced) / f64::from(self.samples),
                ms(self.slowest),
                ms(self.read) / f64::from(self.samples),
                self.abandoned
            ));
            *self = Self::new();
        }
    }
}
