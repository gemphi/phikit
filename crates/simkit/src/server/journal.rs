//! The session journal of a serving run.
//!
//! The RPC server is the one part of simkit that runs with nobody watching:
//! headless, on a machine somewhere, flying a client's drone. When a flight
//! misbehaves there is no terminal to scroll back through, so the serving
//! process leaves a file of its own — `simkit-session.log`, beside the
//! executable — with every `log` record at its time, the gap since the last,
//! how long each client was connected and every request that took too long.
//!
//! philog is the journal; this module is simkit's arming of it. Nothing is
//! written until [`arm`] is called, and only [`RpcServer::start`] calls it,
//! so a test, a benchmark or an example that builds a server and never
//! serves leaves no file — exactly as phiber's window does in a test.
//!
//! [`RpcServer::start`]: crate::server::RpcServer::start

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// The journal's file name, beside the executable.
pub const FILE: &str = "simkit-session.log";

/// A request slower than this gets a line of its own: at 100 ms a control
/// call has already missed several of a drone's control periods.
pub const SLOW_MS: f64 = 100.0;

/// Whether this process has already armed the journal.
static ARMED: AtomicBool = AtomicBool::new(false);

/// Where the journal is written: beside the executable, else in the temp
/// directory.
pub fn path() -> std::path::PathBuf {
    philog::beside_exe(FILE)
}

/// The serving process: start the journal afresh, route the `log` facade's
/// records into it and have a panic write its reason there.
///
/// Once per process: a second server in the same process joins the first's
/// journal rather than truncating it. Best effort throughout — a read-only
/// install writes no journal and the server serves anyway.
pub fn arm() {
    if ARMED.swap(true, Ordering::AcqRel) {
        return;
    }
    philog::install();
    if !philog::arm(FILE) {
        return;
    }
    philog::note(philog::header("SimKit session journal"));
    philog::note_panics();
}

/// One finished request: a line of its own when it was slow enough to be
/// worth one, nothing when it was not.
pub fn slow_request(method: &str, msg_id: u32, took: Duration) {
    if let Some(line) = slow_line(method, msg_id, took) {
        philog::note(line);
    }
}

/// The line a slow request leaves — the method, which call it was, how long
/// it took — and [`None`] when it was quick enough to say nothing about.
fn slow_line(method: &str, msg_id: u32, took: Duration) -> Option<String> {
    match took.as_secs_f64() * 1000.0 >= SLOW_MS {
        true => Some(format!("slow request {method} (id={msg_id}) took {}", philog::took(took))),
        false => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_quick_request_says_nothing_and_a_slow_one_names_itself_with_its_time() {
        assert_eq!(slow_line("moveToPosition", 7, Duration::from_millis(1)), None);
        assert_eq!(
            slow_line("moveToPosition", 7, Duration::from_millis(250)).unwrap(),
            "slow request moveToPosition (id=7) took 250.0ms"
        );
    }

    #[test]
    fn the_journal_goes_beside_the_executable() {
        assert_eq!(path().file_name().unwrap(), FILE);
    }
}
