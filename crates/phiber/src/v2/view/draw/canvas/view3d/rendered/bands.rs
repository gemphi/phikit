//! The horizontal bands a still larger than the device binds is rendered
//! in (`still.rs`): each `(first row, rows)`, as tall as one binding
//! allows less the two rows of overlap into each neighbour, and the
//! progress count that steps over them together.

/// The horizontal bands a frame is rendered in: each `(first row, rows)`,
/// as tall as the device binds, the last one the rest.
pub struct BBandList {
    pub list: Vec<(u32, u32)>,
}

#[cfg(test)]
thread_local! {
    /// A test's stand-in for the device's binding limit, in pixels.
    static FORCED: std::cell::Cell<Option<u64>> = const { std::cell::Cell::new(None) };
}

/// A test's binding limit, in pixels; `None` lifts it (`still.rs`'s tests).
#[cfg(test)]
pub(super) fn force(most: Option<u64>) {
    FORCED.with(|f| f.set(most));
}

impl BBandList {
    /// The pixels `gpu` binds as one image (a test may force fewer).
    pub(super) fn most(gpu: &phiray::Gpu) -> u64 {
        #[cfg(test)]
        if let Some(forced) = FORCED.with(|f| f.get()) {
            return forced;
        }
        phiray::Tracer::max_pixels(gpu)
    }

    pub fn of(w: u32, h: u32, most: u64) -> Self {
        // Four rows of each band are its overlap into its neighbours (`still_stepwise`).
        let rows = ((most / u64::from(w.max(1))).saturating_sub(4) as u32).clamp(1, h.max(1));
        let list = (0..h.max(1)).step_by(rows as usize).map(|y0| (y0, rows.min(h.max(1) - y0))).collect();
        Self { list }
    }

    /// Samples done over every band so far, as a share of `samples`.
    pub(super) fn progress(&self, band: usize, taken: u32, samples: u32) -> u32 {
        let n = self.list.len() as u64;
        ((band as u64 * u64::from(samples) + u64::from(taken)) / n.max(1)) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::BBandList;

    #[test]
    fn a_frame_too_large_for_one_binding_is_cut_into_bands_that_cover_it() {
        let bands = BBandList::of(1920, 1080, 1_398_101);
        assert_eq!(bands.list, [(0, 724), (724, 356)], "728 rows bind; four are each band's overlap");
        assert_eq!(bands.progress(1, 10, 64), 37, "half way through the second band of two: 37 of 64");
        assert_eq!(BBandList::of(64, 32, 1_398_101).list, [(0, 32)], "a small frame is one band");
        assert_eq!(BBandList::of(4096, 4096, 1_398_101).list.len(), 13);
    }
}
