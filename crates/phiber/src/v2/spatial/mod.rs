//! Spatial — the depth view, and the frame budget that keeps it moving.
//!
//! A *spatial* view is one you fly through: the node graph in Spatial mode
//! (`view/draw/canvas/node/spatial`), the welcome window's terrain
//! (`modal/splash/terrain`), and a scene seen from inside rather than
//! through a camera. What they share is not what they draw — it is that
//! the frame must land in time, every time, while the thing being drawn
//! does not change between frames.
//!
//! That is the whole of it, and it is why this module is small. The
//! machinery is already in phiray and came from NVIDIA Warp
//! (`phiray/reference/warp/ANALYSIS.md`), whose own conclusion is worth
//! quoting because it is the opposite of what the hardware vendor sells:
//!
//! > Warp's speed is *not* exotic. It is (1) no per-call allocation, (2) no
//! > per-call argument marshalling, (3) replaying a recorded launch list.
//!
//! None of those three needs CUDA. `phiray::Pass` records a launch list
//! once and replays it with a params write (`Pass::set`, `Pass::run`);
//! `GpuArray` grows by doubling and is never reallocated per frame;
//! `GpuBvh::update` refits a posed frame instead of rebuilding it. So the
//! vendor-neutral path is not a compromise here — it is the same three
//! wins, on any adapter wgpu opens, and in a browser through WebGPU, which
//! is what makes a spatial view shareable at all.
//!
//! What is left, then, is the one decision phiray cannot make for itself:
//! **how much frame to ask for.** A path tracer will happily spend a
//! second on one frame. A spatial view has about sixteen milliseconds. So
//! this module holds the ladder ([`tier`]) and the instrument that says
//! where a frame went ([`cost`]) — nothing else, because nothing else is
//! missing.
//!
//! ## Where this parts from Blender
//!
//! Blender has the same dial and holds it still: `preview_pixel_size`
//! (`rna_scene.cc:6922`) offers AUTO, 1×, 2×, 4×, 8× — "Render at full
//! resolution" down to "Render at 12.5% resolution" — and it is a user
//! preference, set once. [`set::TIER`] is those same four divisors, for
//! the reason that they are the right ones: powers of two resample
//! cleanly. The difference is that a spatial view **moves on the ladder**
//! while it flies, because a budget held by hand is a budget missed on the
//! frame the scene gets heavy.

pub mod cost;
pub mod set;
pub mod tier;

#[cfg(test)]
mod tests;

pub use cost::BSpatialCost;
pub use set::BUDGET_MS;
pub use set::TIER;
pub use tier::BSpatialTier;

/// A spatial view's running state: where it sits on the ladder, and what
/// the last frames cost.
///
/// One of these per spatial host. It owns no GPU object — the recorded
/// `phiray::Pass` lives with the tracer — so it is cheap to keep in state
/// and cheap to copy for a test.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BSpatial {
    /// Index into [`TIER`].
    pub tier: usize,
    /// How long the last frame took, in milliseconds.
    pub last_ms: f32,
    /// What a frame is allowed to take.
    pub budget_ms: f32,
    /// Frames served by replaying the recorded pass.
    pub replays: u32,
    /// Frames that had to record the pass again (a scene change).
    pub records: u32,
}

impl Default for BSpatial {
    fn default() -> Self {
        Self { tier: 0, last_ms: 0.0, budget_ms: BUDGET_MS, replays: 0, records: 0 }
    }
}

impl BSpatial {
    /// The tier in force.
    pub fn at(&self) -> &'static BSpatialTier {
        &TIER[self.tier.min(TIER.len() - 1)]
    }

    /// A frame landed in `took_ms`: record it and step the ladder.
    pub fn frame(&mut self, took_ms: f32) {
        self.last_ms = took_ms;
        self.replays = self.replays.saturating_add(1);
        self.tier = BSpatialTier::step(self.tier, took_ms, self.budget_ms);
    }

    /// The pass had to be recorded again — the scene changed under us.
    ///
    /// Counted separately from [`BSpatial::frame`] because the ratio is the
    /// diagnosis: a spatial view that records as often as it replays is not
    /// getting the recorded-launch win at all, whatever its frame time says.
    pub fn record(&mut self) {
        self.records = self.records.saturating_add(1);
    }

    /// Replays per record, or `None` before the first record.
    ///
    /// Warp's third win is worth exactly this number. One means it is worth
    /// nothing.
    pub fn reuse(&self) -> Option<f32> {
        (self.records > 0).then(|| self.replays as f32 / self.records as f32)
    }

    /// What the next frame will cost at the tier in force.
    pub fn cost(&self, w: u32, h: u32, on_gpu: bool) -> BSpatialCost {
        let t = self.at();
        BSpatialCost::of(t.span(w), t.span(h), t.samples, on_gpu)
    }

    /// Whether the view is holding its budget.
    pub fn in_budget(&self) -> bool {
        self.last_ms <= self.budget_ms
    }
}
