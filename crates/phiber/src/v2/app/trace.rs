//! Frame timing.
//!
//! Always on, cheaply: every frame is counted (`tick`), and every
//! [`SUMMARY_EVERY`] frames a summary goes to the session journal
//! (`journal.rs`) — the average, the tree/draw split, the worst, how many
//! were slow. Any frame slower than [`SLOW_MS`] is written there on its own
//! with where the time went — tree build, draw, what the viewport held and
//! who drew it (GPU or the CPU painter). Send the journal when the app
//! feels slow; it is the answer to "why".
//!
//! `set PHIBER_TRACE=1` additionally prints EVERY frame to stderr:
//!   `frame 14.2ms | tree 1.1 | draw 12.8 | scene 305 objects 8.8M tris | gpu`

use super::journal::BJournal;
use std::cell::RefCell;
use std::time::{Duration, Instant};

thread_local! {
    static LOG: RefCell<BTrace> = const { RefCell::new(BTrace::new()) };
}

/// A frame slower than this is worth keeping: it is under 20 fps.
pub const SLOW_MS: f32 = 50.0;
/// Frames between two summary lines in the journal.
pub const SUMMARY_EVERY: u32 = 120;

/// Rolling frame statistics. The journal lines are always written; the
/// per-frame stderr stream only with `PHIBER_TRACE`.
pub struct BTrace {
    on: Option<bool>,
    frames: u32,
    slow: u32,
    total: Duration,
    worst: Duration,
    /// This summary's span: frames, their tree and draw time, since when.
    span: (u32, Duration, Duration, Option<Instant>),
}

impl BTrace {
    const fn new() -> Self {
        Self { on: None, frames: 0, slow: 0, total: Duration::ZERO, worst: Duration::ZERO, span: (0, Duration::ZERO, Duration::ZERO, None) }
    }

    fn enabled(&mut self) -> bool {
        *self.on.get_or_insert_with(|| std::env::var("PHIBER_TRACE").is_ok_and(|v| v != "0"))
    }

    /// Count one frame; the summary line when this one completes a span.
    fn tick(&mut self, tree: Duration, draw: Duration) -> Option<String> {
        let total = tree + draw;
        self.frames += 1;
        self.total += total;
        self.worst = self.worst.max(total);
        if total.as_secs_f32() * 1000.0 >= SLOW_MS {
            self.slow += 1;
        }
        let (n, t, d, since) = &mut self.span;
        let began = *since.get_or_insert_with(Instant::now);
        *n += 1;
        *t += tree;
        *d += draw;
        if *n < SUMMARY_EVERY {
            return None;
        }
        let line = Self::summary(*n, *t, *d, began.elapsed(), self.worst, self.slow, self.frames);
        self.span = (0, Duration::ZERO, Duration::ZERO, None);
        self.worst = Duration::ZERO;
        Some(line)
    }

    /// `n` frames over `wall`: their average and split, the worst, how many
    /// of the session's frames so far were slow.
    fn summary(n: u32, tree: Duration, draw: Duration, wall: Duration, worst: Duration, slow: u32, frames: u32) -> String {
        let ms = |d: Duration| d.as_secs_f32() * 1000.0;
        let avg = ms(tree + draw) / n as f32;
        format!(
            "{n} frames in {:.1}s: {avg:.1}ms average ({:.0} fps when drawing) | tree {:.1} | draw {:.1} | worst {:.1}ms | {slow} slow of {frames} so far",
            wall.as_secs_f32(),
            1000.0 / avg.max(0.001),
            ms(tree) / n as f32,
            ms(draw) / n as f32,
            ms(worst)
        )
    }
}

/// Where the frame lines go: the session journal.
pub fn log_path() -> std::path::PathBuf {
    BJournal::path()
}

/// Count one finished frame — every frame, slow or not; a summary line
/// reaches the journal every `SUMMARY_EVERY`.
pub fn tick(tree: Duration, draw: Duration) {
    if let Some(line) = LOG.with(|l| l.borrow_mut().tick(tree, draw)) {
        BJournal::note(line);
    }
}

/// Is the per-frame stderr stream on? Frame accounting itself always runs;
/// it costs two `Instant::now()` calls.
pub fn on() -> bool {
    LOG.with(|l| l.borrow_mut().enabled())
}

/// Describe one finished frame that was slow, or that the stderr stream
/// wants: the slow one goes to the journal with what the viewport held.
pub fn frame(tree: Duration, draw: Duration, objects: usize, tris: usize, gpu: bool) {
    // Which device drew it: the card's name and backend, so a log from a
    // machine with two GPUs says which one was used.
    let drew = || match phiray::Gpu::in_use().filter(|_| gpu) {
        Some(a) => format!("gpu {} ({})", a.name, a.backend_label()),
        None if gpu => "gpu".to_string(),
        None => "cpu painter".to_string(),
    };
    LOG.with(|l| {
        let mut l = l.borrow_mut();
        let total = tree + draw;
        let ms = |d: Duration| d.as_secs_f32() * 1000.0;
        if ms(total) >= SLOW_MS {
            BJournal::note(format!(
                "slow frame {:6.1}ms | tree {:5.1} | draw {:6.1} | {objects} objects {:.1}M tris | {}",
                ms(total), ms(tree), ms(draw), tris as f32 / 1e6,
                drew()
            ));
        }
        if !l.enabled() {
            return;
        }
        eprintln!(
            "frame {:6.1}ms | tree {:5.1} | draw {:6.1} | {objects} objects {:.1}M tris | {}",
            ms(total),
            ms(tree),
            ms(draw),
            tris as f32 / 1e6,
            drew()
        );
        if l.frames % 60 == 0 {
            eprintln!(
                "  ── {} frames: {:.1}ms average ({:.0} fps), worst {:.1}ms",
                l.frames,
                ms(l.total) / l.frames.max(1) as f32,
                1000.0 / (ms(l.total) / l.frames.max(1) as f32),
                ms(l.worst)
            );
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_span_of_frames_sums_to_one_line_and_starts_the_next() {
        let mut t = BTrace::new();
        let (tree, draw) = (Duration::from_millis(2), Duration::from_millis(14));
        for _ in 0..SUMMARY_EVERY - 1 {
            assert!(t.tick(tree, draw).is_none());
        }
        let line = t.tick(Duration::from_millis(10), Duration::from_millis(50)).expect("the span's line");
        assert!(line.starts_with("120 frames in"), "{line}");
        assert!(line.contains("worst 60.0ms") && line.contains("1 slow of 120 so far"), "{line}");
        assert!(t.tick(tree, draw).is_none(), "a new span");
        assert_eq!(t.frames, SUMMARY_EVERY + 1);
    }
}

/// Time `f`, returning its value and how long it took.
pub fn timed<T>(f: impl FnOnce() -> T) -> (T, Duration) {
    let at = Instant::now();
    let out = f();
    (out, at.elapsed())
}
