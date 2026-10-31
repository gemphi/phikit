//! philog — the session journal phikit's crates log through.
//!
//! One file per run, written from the moment the run arms it, carrying every
//! `log` record at its time, the gap since the last line, and the timings a
//! caller marks. It is what makes a session readable after the fact: a
//! window that felt slow, a 280 MB file that took a minute to open, a kernel
//! that took two minutes to compile, a headless RPC server flying a client's
//! drone with nobody watching a terminal.
//!
//! philog is a **sink for the `log` facade**, not a logging API. A crate
//! already calling `log::info!` is journalled by installing it:
//!
//! ```no_run
//! philog::install();                      // the `log` facade's records
//! philog::arm("simkit-session.log");      // beside the executable
//! philog::note(philog::header("SimKit session journal"));
//! philog::note_panics();                  // a crash says why, on disk
//! log::info!("RPC Server listening on 0.0.0.0:41451");
//! let _span = philog::BSpan::open("Flight");   // "Flight took 126.312s"
//! ```
//!
//! Three things it is careful about, because its callers are libraries:
//!
//! - **Nothing unarmed.** Until a run calls [`arm`] or [`arm_at`], every
//!   note and every record is dropped, so tests and benchmarks are silent
//!   and no file appears beside their binary.
//! - **No app required.** The journal is a process-wide static; a library
//!   with no app around it — simkit's server — arms it and writes.
//! - **Readable after a crash.** Records go straight to the file with no
//!   buffer in the way, [`flush`] runs on a schedule, and the journal
//!   flushes again as it closes.
//!
//! Each piece lives in its own module: [`journal`] the file and its lines,
//! [`sink`] the `log` facade's way in, [`span`] the timings.

pub mod journal;
pub mod sink;
pub mod span;

#[cfg(test)]
mod tests;

pub use journal::{Feed, arm, arm_at, armed, beside_exe, disarm, feed, flush, header, line, note, note_at, note_panics};
pub use sink::{install, install_at};
pub use span::{BSpan, note_took, timed, took};
