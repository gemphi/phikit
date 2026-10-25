//! Which size the tracer draws at while the viewport is being resized.
//!
//! Every new size restarts the trace from its first sample and reads back
//! whole frames until it refines. Dragging a window edge produces a new size
//! every frame, so the tracer spent the entire drag restarting at sizes that
//! were replaced before they finished — and because the view never settled,
//! the UI kept redrawing flat out the whole time.
//!
//! So a size change is sorted by how it arrives:
//!
//! - **On its own** (a space switch, maximise, a panel toggling) it is taken
//!   at once. Delaying it would only show a stretched picture for nothing.
//! - **In a burst** (a drag: another change inside [`SETTLE`] of the last, or
//!   on the very next frame) the tracer holds the size it already has, and
//!   the painter stretches that image over the new bounds. When the size has
//!   held still for [`SETTLE`] *and* two frames, it is taken and traced once.
//!
//! Frames as well as time, because a slow build is exactly where this
//! matters: at a few frames a second every drag step lands more than
//! [`SETTLE`] after the last, and a clock alone would call each one a lone
//! change and restart the trace for all of them.
//!
//! The camera is untouched by any of this: an orbit or a zoom still posts at
//! once. Only the size waits.

use std::time::Duration;
use web_time::Instant;

/// How long the size must hold still before a burst is over.
pub(super) const SETTLE: Duration = Duration::from_millis(150);

pub(super) type BSize = (u32, u32);

#[derive(Default)]
pub(super) struct BSettle {
    /// The size the tracer is drawing at.
    used: Option<BSize>,
    /// The last live size seen.
    seen: Option<BSize>,
    /// When, and on which frame, the live size last changed.
    changed: Option<(Instant, u64)>,
    /// Whether the change being waited on arrived as part of a burst.
    burst: bool,
    /// Frames seen so far (one call per painted frame).
    frame: u64,
}

impl BSettle {
    /// The size to trace at this frame, and whether a newer size is waiting —
    /// in which case the caller must keep frames coming, or the wait would
    /// never be seen to end.
    pub(super) fn size(&mut self, live: BSize, now: Instant) -> (BSize, bool) {
        self.frame += 1;
        if self.seen != Some(live) {
            if self.seen.is_some() {
                let frame = self.frame;
                self.burst = self.changed.is_some_and(|(at, on)| {
                    now.saturating_duration_since(at) < SETTLE || frame - on <= 1
                });
                self.changed = Some((now, frame));
            }
            self.seen = Some(live);
        }
        let used = *self.used.get_or_insert(live);
        if used == live {
            return (used, false);
        }
        let frame = self.frame;
        let quiet = self.changed.is_some_and(|(at, on)| {
            now.saturating_duration_since(at) >= SETTLE && frame - on >= 2
        });
        if !self.burst || quiet {
            self.used = Some(live);
            self.burst = false;
            return (live, false);
        }
        (used, true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ms(t0: Instant, n: u64) -> Instant {
        t0 + Duration::from_millis(n)
    }

    #[test]
    fn a_lone_change_is_taken_at_once() {
        let t0 = Instant::now();
        let mut s = BSettle::default();
        assert_eq!(s.size((800, 600), t0), ((800, 600), false));
        // A space switch a second later: one change, nothing after it.
        assert_eq!(s.size((1000, 600), ms(t0, 1000)), ((1000, 600), false));
    }

    #[test]
    fn a_drag_holds_the_size_until_it_stops() {
        let t0 = Instant::now();
        let mut s = BSettle::default();
        s.size((800, 600), t0);
        // The first step of a drag is a lone change as far as anyone can tell.
        assert_eq!(s.size((810, 600), ms(t0, 1000)).0, (810, 600));
        // Every step after it lands inside SETTLE of the last: hold 810.
        let mut t = 1000;
        for w in [820, 840, 870, 900, 930] {
            t += 16;
            assert_eq!(s.size((w, 600), ms(t0, t)), ((810, 600), true), "restarted at {w}");
        }
        // The pointer stops. Frames keep coming; the size is still waiting.
        assert_eq!(s.size((930, 600), ms(t0, t + 60)), ((810, 600), true));
        // Quiet for SETTLE: the final size is taken, once.
        assert_eq!(s.size((930, 600), ms(t0, t + 151)), ((930, 600), false));
        assert_eq!(s.size((930, 600), ms(t0, t + 300)), ((930, 600), false));
    }

    #[test]
    fn a_slow_drag_is_still_a_drag() {
        // A debug build at two frames a second: every step is further apart
        // than SETTLE, but each lands on the frame after the last.
        let t0 = Instant::now();
        let mut s = BSettle::default();
        s.size((800, 600), t0);
        assert_eq!(s.size((810, 600), ms(t0, 1000)).0, (810, 600));
        let mut t = 1000;
        for w in [830, 860, 900] {
            t += 500;
            assert_eq!(s.size((w, 600), ms(t0, t)), ((810, 600), true), "restarted at {w}");
        }
        // One quiet frame is not yet a stop, however long it took.
        assert_eq!(s.size((900, 600), ms(t0, t + 500)), ((810, 600), true));
        // Two quiet frames and SETTLE elapsed: taken.
        assert_eq!(s.size((900, 600), ms(t0, t + 1000)), ((900, 600), false));
    }

    #[test]
    fn a_drag_back_to_the_traced_size_needs_no_retrace() {
        let t0 = Instant::now();
        let mut s = BSettle::default();
        s.size((800, 600), t0);
        s.size((810, 600), ms(t0, 1000));
        s.size((830, 600), ms(t0, 1016));
        // Back to where the tracer already is: nothing is waiting.
        assert_eq!(s.size((810, 600), ms(t0, 1032)), ((810, 600), false));
    }
}
