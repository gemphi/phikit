//! Resume the last session. The durable part of `BState` — the workspace and
//! its panels, shading, overlays, the file browser's folder and preferences,
//! which file was open, and (on a throttle) the viewport camera and the
//! playhead — is written to disk whenever it changes, and read back at launch
//! so the window opens where it was left and reopens the last file with Load UI
//! off (the file's own saved layout does not overwrite the one just restored).
//!
//! Only the running app persists (`app::run` arms it); a test's window writes
//! nothing, as the journal does.
//!
//! Three concerns, each one file: `transient` says what resumes and what
//! doesn't — the durable / throttled / volatile split; `snapshot` is the
//! on-disk JSON format; and this file holds [`BSession`], the open-file
//! handle that reads [`is_transient`] through `transient`, writes the
//! [`snapshot`] under a throttle, and flushes at quit so a resting camera is
//! never lost to the throttle tail.
//!
//! The resumed playhead outlives the file's own saved frame: on reopen the
//! session's frame wins (stashed as [`resume_frame`], applied in
//! `settle_import`), because the session is newer than the last save. The
//! session reopens paused — `playing` is volatile, never resumed. Deliberate
//! divergence from Blender, which stores the current frame per-scene and
//! always loads the file's.
//!
//! Blender ref: `wm.read_homefile` restoring the UI, and the recent-files
//! reopen — Phiber keeps it to the last file rather than a list, for now.

pub mod snapshot;
mod transient;

pub use snapshot::{apply, last_file, snapshot};
pub use transient::{is_transient, loaded_path, resume_frame};

use crate::v2::state::BState;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use web_time::Instant;
use transient::{THROTTLE, THROTTLED, VOLATILE};

struct BOpen {
    path: PathBuf,
    /// The revision fingerprint over the *durable* families at the last check —
    /// a change here writes at once.
    last_durable_rev: u64,
    /// The fingerprint over durable *and* throttled families — a change here
    /// with no durable change writes only once the throttle has elapsed.
    last_full_rev: u64,
    /// When the file was last written, for the throttle. `None` until the
    /// first write.
    last_write: Option<Instant>,
    /// The snapshot last written — skips the write when the bytes are unchanged.
    last: String,
}

static OPEN: Mutex<Option<BOpen>> = Mutex::new(None);

/// The families whose change forces an immediate write: everything but the
/// volatile and the throttled.
fn durable_skip() -> Vec<&'static str> {
    let mut skip: Vec<&'static str> = VOLATILE.to_vec();
    skip.extend_from_slice(THROTTLED);
    skip
}

/// The armed session file. Like the journal, only the running app arms it, so
/// a test's window persists nothing.
pub struct BSession;

impl BSession {
    /// Where the snapshot lives: beside the executable, as the journal is, so
    /// a `cargo run` from the workspace finds it; else the temp directory.
    pub fn path() -> PathBuf {
        std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(Path::to_path_buf))
            .unwrap_or_else(std::env::temp_dir)
            .join("phiber-session.json")
    }

    /// The running app: begin persisting to [`Self::path`].
    pub fn arm() {
        Self::arm_at(&Self::path());
    }

    fn arm_at(path: &Path) {
        *OPEN.lock().unwrap() = Some(BOpen {
            path: path.to_path_buf(),
            last_durable_rev: 0,
            last_full_rev: 0,
            last_write: None,
            last: String::new(),
        });
    }

    /// Write the durable snapshot if it has changed. A no-op unless armed.
    /// Cheap when nothing resumable moved. A durable change (layout, the last
    /// file) is written at once; a throttled-only change (the camera, the
    /// playhead) waits out [`THROTTLE`], so an orbit rewrites the file at most
    /// once a second.
    pub fn persist(state: &BState) {
        Self::persist_at(state, Instant::now());
    }

    /// The clock-free core of [`Self::persist`] — the caller supplies `now`, so
    /// the throttle is deterministic under test.
    fn persist_at(state: &BState, now: Instant) {
        let mut guard = OPEN.lock().unwrap();
        let Some(open) = guard.as_mut() else { return };
        let durable_rev = state.revision_except(&durable_skip());
        let full_rev = state.revision_except(VOLATILE);
        let durable_changed = durable_rev != open.last_durable_rev;
        let throttled_changed = full_rev != open.last_full_rev;
        if !durable_changed && !throttled_changed {
            return; // nothing resumable moved (a volatile family doesn't count)
        }
        // A throttled-only change waits out the window; a durable change never
        // does.
        if !durable_changed {
            let due = open.last_write.map_or(true, |t| now.duration_since(t) >= THROTTLE);
            if !due {
                return;
            }
        }
        open.last_durable_rev = durable_rev;
        open.last_full_rev = full_rev;
        let snap = snapshot(state);
        if snap != open.last && std::fs::write(&open.path, &snap).is_ok() {
            open.last = snap;
            open.last_write = Some(now);
        }
    }

    /// Write the snapshot now, ignoring the throttle — the quit path calls this
    /// so the camera and playhead as they rest at exit are never lost to the
    /// throttle tail. A no-op unless armed, and when the bytes are unchanged.
    pub fn flush(state: &BState) {
        let mut guard = OPEN.lock().unwrap();
        let Some(open) = guard.as_mut() else { return };
        let snap = snapshot(state);
        if snap != open.last && std::fs::write(&open.path, &snap).is_ok() {
            open.last = snap;
            open.last_durable_rev = state.revision_except(&durable_skip());
            open.last_full_rev = state.revision_except(VOLATILE);
            open.last_write = Some(Instant::now());
        }
    }

    /// The persisted snapshot, if one exists and is not empty.
    pub fn restore_text() -> Option<String> {
        std::fs::read_to_string(Self::path()).ok().filter(|s| !s.trim().is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::state::{CtxKey, View3dKey};
    use std::time::Duration;

    #[test]
    fn a_durable_change_writes_at_once_the_camera_waits_the_throttle() {
        let dir = std::env::temp_dir().join(format!("phiber-session-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("phiber-session.json");

        BSession::arm_at(&path);
        let t0 = Instant::now();

        // A durable change is written immediately.
        let mut s = BState::new();
        s.set(CtxKey::Space, "Shading");
        BSession::persist_at(&s, t0);
        let written = std::fs::read_to_string(&path).unwrap();
        assert!(written.contains("Shading"), "the durable change was written: {written}");
        assert!(!written.contains("ctx.view3d.yaw"), "no camera yet: {written}");

        // An orbit within the throttle window does not rewrite the file.
        s.set(View3dKey::Yaw, "90");
        BSession::persist_at(&s, t0 + Duration::from_millis(100));
        let written = std::fs::read_to_string(&path).unwrap();
        assert!(!written.contains("90"), "the orbit waited out the throttle: {written}");

        // Once the throttle elapses, the camera is written.
        BSession::persist_at(&s, t0 + THROTTLE + Duration::from_millis(50));
        let written = std::fs::read_to_string(&path).unwrap();
        assert!(written.contains("90"), "the camera was written after the throttle: {written}");

        // A volatile-only change (the clock playing) never rewrites the file.
        s.set(CtxKey::Playing, "true");
        BSession::persist_at(&s, t0 + Duration::from_millis(5000));
        let written = std::fs::read_to_string(&path).unwrap();
        assert!(!written.contains("ctx.playing"), "playing stayed off disk: {written}");

        // flush writes the resting camera at once, ignoring the throttle.
        s.set(View3dKey::Yaw, "123");
        BSession::flush(&s);
        let written = std::fs::read_to_string(&path).unwrap();
        assert!(written.contains("123"), "flush wrote the resting camera: {written}");

        *OPEN.lock().unwrap() = None;
        let _ = std::fs::remove_dir_all(&dir);
    }
}
