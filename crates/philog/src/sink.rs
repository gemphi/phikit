//! The `log` facade's way into the journal.
//!
//! philog adds no logging calls of its own: a crate that already writes
//! `log::info!` is journalled by installing this sink, with no call site
//! changed. Records at or above the installed level are filed by
//! [`journal::note`]; everything else is dropped before it is formatted, so
//! the debug/trace flood of a vendored stack costs nothing.

use crate::journal;
use std::sync::atomic::{AtomicUsize, Ordering};

/// The installed level, as `LevelFilter as usize`. An atomic rather than a
/// field, because `log::set_logger` wants a `&'static` logger.
static LEVEL: AtomicUsize = AtomicUsize::new(log::LevelFilter::Info as usize);

/// Forwards `log` records into the journal.
pub struct BSink;

static SINK: BSink = BSink;

impl log::Log for BSink {
    fn enabled(&self, meta: &log::Metadata) -> bool {
        meta.level() as usize <= LEVEL.load(Ordering::Relaxed)
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            journal::note(format!("[{}] {}: {}", record.level(), record.target(), record.args()));
        }
    }

    fn flush(&self) {
        journal::flush();
    }
}

/// Route the `log` facade into the journal at Info and above — the level that
/// keeps a diagnostic like the chosen GPU adapter without the debug flood.
///
/// Best effort: `false` when a logger is already set, and that one is left in
/// place. Installing the sink does not open a journal; nothing is written
/// until a run arms one.
pub fn install() -> bool {
    install_at(log::LevelFilter::Info)
}

/// As [`install`], at a level of the caller's choosing.
pub fn install_at(level: log::LevelFilter) -> bool {
    LEVEL.store(level as usize, Ordering::Relaxed);
    match log::set_logger(&SINK).is_ok() {
        true => {
            log::set_max_level(level);
            true
        }
        false => false,
    }
}
