//! Spans and timings: how long something took, beside the lines it took it in.
//!
//! A journal of lines alone says what happened; a journal with timings says
//! where the session went. [`BSpan`] opens at the start of a piece of work
//! and files `"... took 126.312s"` when it closes — on the way out of the
//! scope, so a `?` or a panic still leaves the timing — and [`timed`] is the
//! same thing around a closure.

use crate::journal;
use std::time::Duration;
use web_time::Instant;

/// A piece of work being timed: its name, from when it began.
///
/// The line is filed when the span is dropped. [`BSpan::close`] drops it
/// early, and [`BSpan::elapsed`] reads the clock without closing it.
pub struct BSpan {
    what: String,
    began: Instant,
}

impl BSpan {
    /// Open a span: `what` names the work as the journal line will.
    pub fn open(what: impl Into<String>) -> Self {
        Self { what: what.into(), began: Instant::now() }
    }

    /// How long the span has been open.
    pub fn elapsed(&self) -> Duration {
        self.began.elapsed()
    }

    /// File the timing now rather than at the end of the scope.
    pub fn close(self) {}

    /// Note a line of this span's own, with how far into it we are: a stage
    /// of a long piece of work, inside the one timing.
    pub fn note(&self, text: impl AsRef<str>) {
        journal::note(format!("{}: {} (at {})", self.what, text.as_ref(), took(self.elapsed())));
    }
}

impl Drop for BSpan {
    fn drop(&mut self) {
        journal::note(format!("{} took {}", self.what, took(self.began.elapsed())));
    }
}

/// Run `work`, and file how long it took as `what`. The value is returned
/// unchanged, so this wraps an expression where it stands.
pub fn timed<T>(what: impl Into<String>, work: impl FnOnce() -> T) -> T {
    let span = BSpan::open(what);
    let out = work();
    drop(span);
    out
}

/// File a duration a caller measured itself, under `what`.
pub fn note_took(what: &str, how_long: Duration) {
    journal::note(format!("{what} took {}", took(how_long)));
}

/// A duration as the journal says it: milliseconds under a second, seconds
/// above, so a 126-second kernel build reads as one number.
pub fn took(how_long: Duration) -> String {
    match how_long.as_secs_f64() {
        s if s < 1.0 => format!("{:.1}ms", s * 1000.0),
        s => format!("{s:.3}s"),
    }
}
