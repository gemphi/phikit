//! The session journal: one file, from the moment the window opens, with
//! every line of the Info log at its time, every slow frame, the frame
//! summaries and the tracer's timings — so a slow session can be read
//! after the fact, action by action, with the gaps between them.
//!
//! It is written next to the executable (`target/release/phiber-session.log`
//! under a `cargo run`), where the workspace can see it, and falls back to
//! the temp directory. A new run starts it afresh. Only the running app
//! writes it (`app::run` arms it); a test's window writes nothing.

use crate::v2::state::log;
use std::io::Write;
use std::sync::Mutex;
use web_time::Instant;

/// Forwards `log` crate records into the journal (`BJournal::bridge_log`), at
/// Info and above so the GPU adapter line is kept without the debug/trace flood.
struct BLogBridge;
impl ::log::Log for BLogBridge {
    fn enabled(&self, meta: &::log::Metadata) -> bool {
        meta.level() <= ::log::Level::Info
    }
    fn log(&self, record: &::log::Record) {
        if self.enabled(record.metadata()) {
            BJournal::note(format!("[{}] {}: {}", record.level(), record.target(), record.args()));
        }
    }
    fn flush(&self) {}
}

/// The open journal.
struct BOpen {
    file: std::fs::File,
    /// When the session began: every line's time is counted from it.
    began: Instant,
    /// When the last line was written: every line says how long since.
    last: Instant,
    /// The newest Info line written so far.
    seq: u64,
}

static OPEN: Mutex<Option<BOpen>> = Mutex::new(None);

pub struct BJournal;

impl BJournal {
    /// Where the journal is written: beside the executable, else in the
    /// temp directory.
    pub fn path() -> std::path::PathBuf {
        std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(std::path::Path::to_path_buf))
            .unwrap_or_else(std::env::temp_dir)
            .join("phiber-session.log")
    }

    /// The running app: start the journal afresh at `path`.
    pub fn arm() {
        Self::arm_at(&Self::path());
    }

    fn arm_at(path: &std::path::Path) {
        let Ok(file) = std::fs::File::create(path) else { return };
        let now = Instant::now();
        *OPEN.lock().unwrap() = Some(BOpen { file, began: now, last: now, seq: 0 });
        Self::note(format!(
            "Phiber session journal — {} build, {}",
            if cfg!(debug_assertions) { "debug" } else { "release" },
            std::env::current_exe().map_or("?".to_string(), |exe| exe.display().to_string())
        ));
        Self::note_panics();
    }

    /// A panic on any thread writes itself here before the process's own
    /// hook runs: a crash leaves its reason in the journal, not only on a
    /// console nobody had open.
    fn note_panics() {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let thread = std::thread::current().name().unwrap_or("?").to_string();
            let where_ = info.location().map_or(String::new(), |l| format!(" at {}:{}", l.file(), l.line()));
            let what = info.payload().downcast_ref::<&str>().map(|s| s.to_string()).or_else(|| info.payload().downcast_ref::<String>().cloned()).unwrap_or_default();
            // The lock may be held by the thread that panicked: try, do not wait.
            if let Ok(mut guard) = OPEN.try_lock()
                && let Some(open) = guard.as_mut()
            {
                Self::write(open, Instant::now(), &format!("PANIC on thread '{thread}'{where_}: {what}"));
            }
            previous(info);
        }));
    }

    /// Route the `log` crate — wgpu, gpui and the rest of the vendored stack —
    /// into the journal, so a diagnostic like the chosen GPU adapter and
    /// backend lands on disk (phiber installs no other logger, so these records
    /// were dropped before). Best effort: if a logger is already set, it is
    /// left in place. `::log` is the crate; `log` in this file is the in-app
    /// one (`state::log`).
    pub fn bridge_log() {
        static LOGGER: BLogBridge = BLogBridge;
        if ::log::set_logger(&LOGGER).is_ok() {
            ::log::set_max_level(::log::LevelFilter::Info);
        }
    }

    /// A line of its own — a slow frame, a frame summary, the tracer
    /// settling — at this moment. Nothing when the journal is not armed.
    pub fn note(text: impl AsRef<str>) {
        let at = Instant::now();
        if let Some(open) = OPEN.lock().unwrap().as_mut() {
            Self::write(open, at, text.as_ref());
        }
    }

    /// Each frame of the running app: the Info lines written since the
    /// last, each at its own time.
    pub fn each_frame() {
        let mut guard = OPEN.lock().unwrap();
        let Some(open) = guard.as_mut() else { return };
        let entries = log::entries();
        let written = open.seq;
        for entry in entries.iter().rev().filter(|e| e.seq > written) {
            Self::write(open, entry.at, &entry.text);
            open.seq = entry.seq;
        }
    }

    fn write(open: &mut BOpen, at: Instant, text: &str) {
        let _ = writeln!(open.file, "{}", Self::line(at.saturating_duration_since(open.began), at.saturating_duration_since(open.last), text));
        open.last = at.max(open.last);
    }

    /// One line: the time since the session began, the gap since the last
    /// line, the text.
    fn line(since_start: std::time::Duration, gap: std::time::Duration, text: &str) -> String {
        format!("{:9.3}s  +{:8.1}ms  {text}", since_start.as_secs_f64(), gap.as_secs_f64() * 1000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_line_carries_the_time_from_the_start_and_the_gap_since_the_last() {
        let line = BJournal::line(std::time::Duration::from_millis(12_345), std::time::Duration::from_micros(2_500), "Opened Cube.blend");
        assert_eq!(line, "   12.345s  +     2.5ms  Opened Cube.blend");
    }

    #[test]
    fn an_armed_journal_files_the_info_lines_once_each_in_order() {
        let dir = std::env::temp_dir().join(format!("phiber-journal-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("phiber-session.log");
        BJournal::arm_at(&path);
        // Names no other test writes: the journal is one file for the
        // process, and the other tests' lines may land in it meanwhile.
        log::push("Opening Journal_Probe.blend");
        log::push("Opened Journal_Probe.blend: 3 objects");
        BJournal::each_frame();
        BJournal::each_frame();
        BJournal::note("slow frame 61.0ms (journal probe)");
        *OPEN.lock().unwrap() = None;
        let text = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert!(lines[0].contains("session journal"), "{text}");
        let ours: Vec<_> = lines.iter().filter(|l| l.contains("Journal_Probe") || l.contains("journal probe")).collect();
        assert_eq!(ours.len(), 3, "each Info line once, then the note: {text}");
        assert!(ours[0].ends_with("Opening Journal_Probe.blend") && ours[1].ends_with("Opened Journal_Probe.blend: 3 objects"));
        assert!(ours[2].ends_with("slow frame 61.0ms (journal probe)"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
