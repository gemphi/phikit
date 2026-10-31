//! The session journal: one file, from the moment the run begins, with every
//! record at its time and the gap since the last — so a slow or a misbehaving
//! session can be read after the fact, action by action, with the gaps
//! between them.
//!
//! Nothing is written until a run arms it ([`arm`], [`arm_at`]), so a test,
//! a benchmark or a library linked into someone else's binary is silent.
//! A new run starts the file afresh.
//!
//! Records go straight to the file: there is no buffer between a line and
//! the disk, because the sessions worth reading are the ones that ended in a
//! crash. [`flush`] is called on a schedule and when the journal closes, so
//! whatever the OS holds back is handed over too.

use std::io::Write;
use std::sync::Mutex;
use std::sync::MutexGuard;
use web_time::Instant;

/// Lines between two flushes; the journal also flushes when it closes.
const FLUSH_EVERY: u32 = 16;

/// The open journal.
struct BOpen {
    file: std::fs::File,
    /// When the session began: every line's time is counted from it.
    began: Instant,
    /// When the last line was written: every line says how long since.
    last: Instant,
    /// The newest record a [`Feed`] has filed; see [`Feed::cursor`].
    cursor: u64,
    /// Lines written since the last flush.
    since_flush: u32,
}

impl Drop for BOpen {
    /// A closing journal hands over what the OS still holds, so the file on
    /// disk is readable even when the run ended badly.
    fn drop(&mut self) {
        let _ = self.file.flush();
    }
}

static OPEN: Mutex<Option<BOpen>> = Mutex::new(None);

/// Where a journal called `name` goes: beside the executable, where the
/// workspace can see it (`target/release/<name>` under a `cargo run`), else
/// in the temp directory.
pub fn beside_exe(name: &str) -> std::path::PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(std::path::Path::to_path_buf))
        .unwrap_or_else(std::env::temp_dir)
        .join(name)
}

/// This run: start the journal afresh beside the executable, as `name`.
/// `false` when the file could not be opened — a read-only install, a
/// browser build — and the journal stays closed.
pub fn arm(name: &str) -> bool {
    arm_at(&beside_exe(name))
}

/// This run: start the journal afresh at `path`.
pub fn arm_at(path: &std::path::Path) -> bool {
    let Ok(file) = std::fs::File::create(path) else { return false };
    let now = Instant::now();
    *OPEN.lock().unwrap() = Some(BOpen { file, began: now, last: now, cursor: 0, since_flush: 0 });
    true
}

/// Close the journal: the file is flushed and dropped, and later records go
/// nowhere. A run that ends cleanly does not need this; a test that wants to
/// read what it wrote does.
pub fn disarm() {
    *OPEN.lock().unwrap() = None;
}

/// Is a journal open? Callers that would build an expensive line ask first.
pub fn armed() -> bool {
    OPEN.lock().unwrap().is_some()
}

/// The line a session opens with: what was built and which executable is
/// running, so a journal says which binary left it.
pub fn header(title: &str) -> String {
    format!(
        "{title} — {} build, {}",
        match cfg!(debug_assertions) {
            true => "debug",
            false => "release",
        },
        std::env::current_exe().map_or("?".to_string(), |exe| exe.display().to_string())
    )
}

/// A line of its own — a slow frame, a stage finishing, a span's timing — at
/// this moment. Nothing when the journal is not armed.
pub fn note(text: impl AsRef<str>) {
    note_at(Instant::now(), text.as_ref());
}

/// A line that happened at `at` rather than now: a caller with its own
/// timestamped records files them at the time they were stamped.
pub fn note_at(at: Instant, text: &str) {
    if let Some(open) = OPEN.lock().unwrap().as_mut() {
        write(open, at, text);
    }
}

/// Hand over whatever the OS still holds.
pub fn flush() {
    if let Some(open) = OPEN.lock().unwrap().as_mut() {
        let _ = open.file.flush();
        open.since_flush = 0;
    }
}

/// A panic on any thread writes itself here before the process's own hook
/// runs: a crash leaves its reason in the journal, not only on a console
/// nobody had open.
pub fn note_panics() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let thread = std::thread::current().name().unwrap_or("?").to_string();
        let where_ = info.location().map_or(String::new(), |l| format!(" at {}:{}", l.file(), l.line()));
        let what = info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_default();
        // The lock may be held by the thread that panicked: try, do not wait.
        if let Ok(mut guard) = OPEN.try_lock()
            && let Some(open) = guard.as_mut()
        {
            write(open, Instant::now(), &format!("PANIC on thread '{thread}'{where_}: {what}"));
        }
        previous(info);
    }));
}

/// A feeder's turn at the open journal: several records under one lock, in
/// order, so another thread's note cannot land between two of them.
///
/// [`None`] when the journal is not armed. Do not call [`note`] while a
/// `Feed` is held — it waits on the same lock.
pub fn feed() -> Option<Feed> {
    let guard = OPEN.lock().unwrap();
    match guard.is_some() {
        true => Some(Feed(guard)),
        false => None,
    }
}

/// The open journal, held for a run of records; see [`feed`].
pub struct Feed(MutexGuard<'static, Option<BOpen>>);

impl Feed {
    /// The newest record filed so far, as the feeder numbered it: a caller
    /// draining its own list writes only what is past this. Zero on a journal
    /// nothing has fed yet, and zero again after a re-arm.
    pub fn cursor(&self) -> u64 {
        self.0.as_ref().map_or(0, |open| open.cursor)
    }

    /// Remember that everything up to `seq` has been filed.
    pub fn set_cursor(&mut self, seq: u64) {
        if let Some(open) = self.0.as_mut() {
            open.cursor = seq;
        }
    }

    /// File one record at the time it was stamped.
    pub fn note_at(&mut self, at: Instant, text: &str) {
        if let Some(open) = self.0.as_mut() {
            write(open, at, text);
        }
    }
}

fn write(open: &mut BOpen, at: Instant, text: &str) {
    let _ = writeln!(
        open.file,
        "{}",
        line(at.saturating_duration_since(open.began), at.saturating_duration_since(open.last), text)
    );
    open.last = at.max(open.last);
    open.since_flush += 1;
    if open.since_flush >= FLUSH_EVERY {
        let _ = open.file.flush();
        open.since_flush = 0;
    }
}

/// One line: the time since the session began, the gap since the last line,
/// the text. Public so a caller that shows the journal's own format — or
/// tests that pin it — need not spell it again.
pub fn line(since_start: std::time::Duration, gap: std::time::Duration, text: &str) -> String {
    format!("{:9.3}s  +{:8.1}ms  {text}", since_start.as_secs_f64(), gap.as_secs_f64() * 1000.0)
}
