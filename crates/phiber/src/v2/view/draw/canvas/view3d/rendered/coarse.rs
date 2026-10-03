//! The sizes a navigated view is traced at, small first (`worker.rs`).

use phiray::Mode;

/// How much smaller than the view the next sample is traced: a navigated
/// view starts at an eighth (a quarter for Workbench, one dispatch) and
/// doubles each sample until full size, where refinement goes on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct BCoarse(u32);

/// A full-size sample predicted to take no longer than this starts at full
/// size: there is nothing to gain from a smaller first image, and a
/// navigated view of a deterministic mode then never pixelates.
pub(super) const FULL_AT_ONCE_MS: f64 = 33.0;

impl BCoarse {
    pub fn start(mode: Mode) -> Self {
        Self(if mode.deterministic() { 4 } else { 8 })
    }

    /// `start`, unless the last full-size sample of this mode says a full
    /// one is quick (`predicted_ms`, from `BSlicer`): then full at once.
    pub fn start_for(mode: Mode, predicted_ms: Option<f64>) -> Self {
        match predicted_ms {
            Some(ms) if ms <= FULL_AT_ONCE_MS => Self(1),
            _ => Self::start(mode),
        }
    }

    pub fn pixels(self, w: u32, h: u32) -> (u32, u32) {
        ((w / self.0).max(1), (h / self.0).max(1))
    }

    pub fn full(self) -> bool {
        self.0 == 1
    }

    pub fn next(&mut self) {
        self.0 = (self.0 / 2).max(1);
    }
}
