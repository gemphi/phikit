//! One rung of the ladder, and how the ladder is climbed.

use super::set::{CLIMB, TIER};

/// A resolution-and-sample pair a spatial frame can be asked for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BSpatialTier {
    /// How this tier reads in the UI.
    pub name: &'static str,
    /// Blender's pixel size: the divisor applied to each axis.
    pub pixel: u32,
    /// Paths per pixel at this tier.
    pub samples: u32,
}

impl BSpatialTier {
    /// Every tier, finest first.
    pub fn every() -> &'static [BSpatialTier] {
        TIER
    }

    /// `n` pixels along an axis, at this tier. Never zero: a one-pixel
    /// viewport still has to render something.
    pub fn span(&self, n: u32) -> u32 {
        (n / self.pixel.max(1)).max(1)
    }

    /// Pixels in a `w × h` frame at this tier.
    pub fn pixels(&self, w: u32, h: u32) -> u64 {
        self.span(w) as u64 * self.span(h) as u64
    }

    /// The tier to use for the next frame, given that the frame at `at`
    /// took `took_ms` against `budget_ms`.
    ///
    /// Down one rung when over budget, up one when the frame fits in
    /// [`CLIMB`] of it, and nowhere otherwise. One rung at a time in both
    /// directions: a view that drops two tiers on one slow frame — a
    /// shader compiling, a file loading — spends the next second climbing
    /// back through a resolution the user can see change.
    pub fn step(at: usize, took_ms: f32, budget_ms: f32) -> usize {
        let last = TIER.len() - 1;
        let at = at.min(last);
        if budget_ms <= 0.0 || !took_ms.is_finite() {
            return at;
        }
        if took_ms > budget_ms {
            return (at + 1).min(last);
        }
        if took_ms <= budget_ms * CLIMB {
            return at.saturating_sub(1);
        }
        at
    }
}
