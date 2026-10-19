//! What one spatial frame costs — the instrument, not an estimate.
//!
//! Every field here is counted, not modelled. There is no
//! milliseconds-per-ray constant in this file and there should not be one:
//! that number belongs to an adapter and is measured (`examples/limits`),
//! and a guessed one would turn an instrument into a fortune teller.
//!
//! What the frame *does* tell you without any constant at all is whether
//! it crosses the bus, and that is the finding worth having: a spatial
//! frame that comes back to the CPU pays for the copy every frame, and
//! `phiray/reference/warp/ANALYSIS.md` §3 says that copy is avoidable for
//! nothing — the compute buffer and the viewport's buffer are on one
//! device, so the kernel can write the one the display reads. Warp needs
//! `RegisteredGLBuffer` to arrange that across CUDA and GL. We do not need
//! anything.

/// Bytes a display pixel occupies on the way back: RGBA8, as
/// `phiray::Tracer::pixels` returns it.
pub const PIXEL_BYTES: u64 = 4;

/// One frame, counted.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BSpatialCost {
    /// Pixels the frame covers, after the tier's divisor.
    pub pixels: u64,
    /// Paths per pixel.
    pub samples: u32,
    /// Primary paths the frame asks for: `pixels × samples`.
    pub rays: u64,
    /// Bytes the frame copies back to the CPU; zero when it stays on the
    /// GPU.
    pub readback: u64,
}

impl BSpatialCost {
    /// The cost of a `w × h` frame at `samples`. `on_gpu` says the frame
    /// is handed to the display as a GPU texture rather than read back.
    pub fn of(w: u32, h: u32, samples: u32, on_gpu: bool) -> Self {
        let pixels = w as u64 * h as u64;
        Self {
            pixels,
            samples,
            rays: pixels.saturating_mul(samples.max(1) as u64),
            readback: if on_gpu { 0 } else { pixels.saturating_mul(PIXEL_BYTES) },
        }
    }

    /// Whether this frame copies itself across the bus.
    ///
    /// True is not a performance *estimate* — it is a fact about the frame,
    /// and the one cost on this list that can be taken to zero without
    /// giving anything up.
    pub fn crosses_bus(&self) -> bool {
        self.readback > 0
    }

    /// Primary paths per millisecond this frame demands of `budget_ms`.
    ///
    /// Compare it against what the adapter actually sustains to know
    /// whether the tier is reachable. This is the number to put next to a
    /// measurement; it is not one itself.
    pub fn rays_per_ms(&self, budget_ms: f32) -> Option<f64> {
        (budget_ms > 0.0).then(|| self.rays as f64 / budget_ms as f64)
    }

    /// The same frame with the readback taken out — what §3 is worth here.
    pub fn on_gpu(self) -> Self {
        Self { readback: 0, ..self }
    }
}
