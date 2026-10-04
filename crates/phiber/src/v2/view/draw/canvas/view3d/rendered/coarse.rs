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

    /// A tile of a larger frame (`Camera::tile`), at this size: the frame
    /// and the corner shrink with the image, so the tile's rays stay the
    /// region's (the viewport's own render region, `BRayView::border`).
    pub fn tile(self, tile: [u32; 4]) -> [u32; 4] {
        if tile == [0; 4] {
            return tile;
        }
        [tile[0] / self.0, tile[1] / self.0, (tile[2] / self.0).max(1), (tile[3] / self.0).max(1)]
    }

    pub fn full(self) -> bool {
        self.0 == 1
    }

    pub fn next(&mut self) {
        self.0 = (self.0 / 2).max(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tile_shrinks_with_the_coarse_image() {
        let mut c = BCoarse::start(Mode::Rendered);
        assert_eq!(c.pixels(800, 400), (100, 50));
        // The right half's lower half of an 800 × 400 frame, at an eighth.
        assert_eq!(c.tile([400, 200, 800, 400]), [50, 25, 100, 50]);
        assert_eq!(c.tile([0; 4]), [0; 4], "the frame itself stays the frame");
        while !c.full() {
            c.next();
        }
        assert_eq!(c.tile([400, 200, 800, 400]), [400, 200, 800, 400]);
    }
}
