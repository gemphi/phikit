//! Frame timing.
//!
//! Always on, cheaply: any frame slower than [`SLOW_MS`] is appended to
//! `phiber-frames.log` in the temp directory (`%TEMP%` on Windows) with where
//! the time went — tree build, draw, what the viewport held and who drew it
//! (GPU or the CPU painter). Send that file when the app feels slow; it is
//! the answer to "why". Every 60 slow frames a summary line is written too.
//!
//! `set PHIBER_TRACE=1` additionally prints EVERY frame to stderr:
//!   `frame 14.2ms | tree 1.1 | draw 12.8 | scene 305 objects 8.8M tris | gpu`

use std::cell::RefCell;
use std::time::{Duration, Instant};

thread_local! {
    static LOG: RefCell<BTrace> = const { RefCell::new(BTrace::new()) };
}

/// A frame slower than this is worth keeping: it is under 20 fps.
pub const SLOW_MS: f32 = 50.0;

/// Rolling frame statistics. The slow-frame log is always kept; the
/// per-frame stderr stream only with `PHIBER_TRACE`.
pub struct BTrace {
    on: Option<bool>,
    frames: u32,
    slow: u32,
    total: Duration,
    worst: Duration,
    file: Option<Option<std::fs::File>>,
}

impl BTrace {
    const fn new() -> Self {
        Self { on: None, frames: 0, slow: 0, total: Duration::ZERO, worst: Duration::ZERO, file: None }
    }

    fn enabled(&mut self) -> bool {
        *self.on.get_or_insert_with(|| std::env::var("PHIBER_TRACE").is_ok_and(|v| v != "0"))
    }

    /// The slow-frame log, opened once, appended to.
    fn log(&mut self) -> Option<&mut std::fs::File> {
        self.file
            .get_or_insert_with(|| {
                std::fs::OpenOptions::new().create(true).append(true).open(log_path()).ok()
            })
            .as_mut()
    }
}

/// Where the slow-frame log lives: `<temp>/phiber-frames.log`.
pub fn log_path() -> std::path::PathBuf {
    std::env::temp_dir().join("phiber-frames.log")
}

/// Is the per-frame stderr stream on? Frame accounting itself always runs;
/// it costs two `Instant::now()` calls.
pub fn on() -> bool {
    LOG.with(|l| l.borrow_mut().enabled())
}

/// Record one finished frame.
pub fn frame(tree: Duration, draw: Duration, objects: usize, tris: usize, gpu: bool) {
    use std::io::Write;
    LOG.with(|l| {
        let mut l = l.borrow_mut();
        let total = tree + draw;
        let ms = |d: Duration| d.as_secs_f32() * 1000.0;
        l.frames += 1;
        l.total += total;
        l.worst = l.worst.max(total);
        if ms(total) >= SLOW_MS {
            l.slow += 1;
            let (slow, frames, avg) = (l.slow, l.frames, ms(l.total) / l.frames as f32);
            if let Some(f) = l.log() {
                let stamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_or(0, |d| d.as_secs());
                let _ = writeln!(
                    f,
                    "{stamp} slow frame {:6.1}ms | tree {:5.1} | draw {:6.1} | {objects} objects {:.1}M tris | {}",
                    ms(total), ms(tree), ms(draw), tris as f32 / 1e6,
                    if gpu { "gpu" } else { "cpu painter" }
                );
                if slow % 60 == 0 {
                    let _ = writeln!(f, "{stamp}   ── {slow} slow of {frames} frames, {avg:.1}ms average");
                }
            }
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
            if gpu { "gpu" } else { "cpu painter" }
        );
        if l.frames % 60 == 0 {
            eprintln!(
                "  ── {} frames: {:.1}ms average ({:.0} fps), worst {:.1}ms",
                l.frames,
                ms(l.total) / l.frames as f32,
                1000.0 / (ms(l.total) / l.frames as f32),
                ms(l.worst)
            );
            l.worst = Duration::ZERO;
        }
    });
}

/// Time `f`, returning its value and how long it took.
pub fn timed<T>(f: impl FnOnce() -> T) -> (T, Duration) {
    let at = Instant::now();
    let out = f();
    (out, at.elapsed())
}
