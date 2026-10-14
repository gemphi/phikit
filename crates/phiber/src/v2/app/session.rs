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
//! Two speeds of persistence:
//! - **Durable** state (the layout, panels, shading, the browser, the last
//!   file) is written the moment it changes.
//! - **Throttled** state — the camera, the timeline editors' view
//!   ([`THROTTLED`]: `view3d`, `time_view`) and the playhead (`frame`) — is
//!   resumed too, but written at most once per [`THROTTLE`] window, so an
//!   orbit, a scrub or a playing timeline rewrites the file roughly once a
//!   second rather than every frame. The quit path flushes
//!   ([`BSession::flush`]) so the final resting camera is never lost to the
//!   throttle tail.
//! - **Volatile** state ([`VOLATILE`]: the transport flags, the scene's frame
//!   range and fps, marker/key drags) is neither persisted nor resumed — the
//!   range belongs to the file, and reopening restores it.
//!
//! The resumed playhead outlives the file's own saved frame: on reopen the
//! session's frame wins (stashed as [`RESUME_FRAME`], applied in
//! `settle_import`), because the session is newer than the last save. The
//! session reopens paused — `playing` is volatile, never resumed. Deliberate
//! divergence from Blender, which stores the current frame per-scene and always
//! loads the file's.
//!
//! Blender ref: `wm.read_homefile` restoring the UI, and the recent-files
//! reopen — Phiber keeps it to the last file rather than a list, for now.

use crate::v2::state::BState;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Key families that are resumed but change many times a second — the viewport
/// camera, the timeline editors' view (pan/zoom), and the playhead. Persisted
/// like the durable state, but on the [`THROTTLE`] so an orbit, a timeline
/// scrub or a playing clock does not rewrite the file every frame. They are NOT
/// [`is_transient`]: the snapshot carries them and the launch applies them.
/// (`time_view` is pure UI — no file-load path seeds it — so it resumes on the
/// Load-UI-off reopen with no stash, unlike the playhead.)
const THROTTLED: &[&str] = &["view3d", "time_view", "frame"];

/// Key families that are neither persisted nor resumed: the transport flags and
/// the scene's own playback settings (range, fps, markers, key drags). The
/// range and fps belong to the `.blend` and come back when it reopens; the rest
/// are mid-interaction. Left out of the snapshot and the change check both.
const VOLATILE: &[&str] = &[
    "playing", "frame_step", "frame_start", "frame_end", "fps",
    "play_from", "auto_key", "preview_range", "preview_start", "preview_end",
    "markers", "keys",
];

/// How often the throttled families (camera, playhead) may rewrite the session
/// file. A durable change ignores this and writes at once.
const THROTTLE: Duration = Duration::from_millis(1000);

/// File-browser keys that are about the current browse, not a preference: what
/// is selected, an error, the nav intent and its history, the add-bookmark
/// intent, the open filter popover. A resumed browser sits at its folder, not
/// mid-interaction, so these are dropped.
const FILE_EPHEMERAL: &[&str] = &[
    "ctx.file.selected", "ctx.file.error", "ctx.file.nav", "ctx.file.hist_back",
    "ctx.file.hist_fwd", "ctx.file.bookmark_add", "ctx.file.filter_open",
];

/// The one `scene` key that is a durable session fact rather than scene data
/// or a transient flag: the path of the file that was open. Kept so the launch
/// knows what to reopen; applied as a *reopen*, never written straight back
/// (that would claim a file was loaded with no scene behind it).
pub const LOADED_PATH: &str = "ctx.scene.loaded_path";

/// A transient key [`crate::v2::app::BPhiberApp::apply_session`] stashes the
/// resumed playhead into, so the reopen's `settle_import` lands on it rather
/// than on the file's saved frame. Under `ctx.scene.`, so it is transient and
/// never itself persisted.
pub const RESUME_FRAME: &str = "ctx.scene.resume_frame";

/// The coarse family of an encoded key — the segment after `ctx.`.
fn family(encoded: &str) -> &str {
    encoded.strip_prefix("ctx.").unwrap_or(encoded).split('.').next().unwrap_or("")
}

/// True when an encoded key is NOT part of the resumable session: a volatile
/// family, an open menu / header popover / dialog, an area-resize drag, the
/// import machinery or a scene-data edit, or a mid-interaction file-browser
/// key. Everything else — the workspace, panels, shading, overlays, the file
/// browser's folder and preferences, the camera and playhead ([`THROTTLED`]),
/// and [`LOADED_PATH`] — is durable.
pub fn is_transient(encoded: &str) -> bool {
    // The last file is the one durable `scene` key; it overrides the blanket
    // scene exclusion below.
    if encoded == LOADED_PATH {
        return false;
    }
    if VOLATILE.contains(&family(encoded)) {
        return true;
    }
    // THROTTLED families (the camera, the playhead) ARE resumable — they are
    // not transient, they just persist on a throttle (see `persist`).
    if FILE_EPHEMERAL.contains(&encoded) {
        return true;
    }
    // The Image editor's shown name points into the session's image list,
    // which does not persist: resumed, it would dangle and paint nothing.
    matches!(encoded, "ctx.modal" | "ctx.selected" | "ctx.image.shown" | "ctx.area.maximized")
        // A toolbar's expansion (`ctx.<host>.tool.expanded`): Blender's
        // toolbar opens icon-only — labels only once it is dragged wide — so
        // the squares come back at launch whatever was left (`part::strip`).
        || encoded.ends_with(".tool.expanded")
        || encoded.starts_with("ctx.menu.")
        || encoded.starts_with("ctx.header.")
        || encoded.starts_with("ctx.area_resize.")
        // Every other scene key is scene data or a transient flag (loading,
        // error, unsaved, the edit-delta markers, the 3D cursor, the resume
        // stash): reopening the file is what restores the scene, not these.
        || encoded.starts_with("ctx.scene.")
}

/// The durable key/value pairs of `state`, as sorted JSON — the snapshot that
/// is written to disk and read back at launch. Carries the throttled families
/// (camera, playhead) too; only [`is_transient`] keys are dropped.
pub fn snapshot(state: &BState) -> String {
    let map: BTreeMap<&str, &str> = state
        .iter_keys()
        .filter(|k| !is_transient(k))
        .filter_map(|k| state.get_encoded(k).map(|v| (k.as_str(), v)))
        .collect();
    serde_json::to_string_pretty(&map).unwrap_or_default()
}

/// Apply a snapshot back onto `state`, skipping [`LOADED_PATH`] (the file is
/// reopened, not faked loaded) and anything that is transient now (so a stale
/// file from an older build can never seed a key this build treats as
/// ephemeral). The camera and playhead come back here; the reopen keeps the
/// playhead via [`RESUME_FRAME`].
pub fn apply(state: &mut BState, json: &str) {
    let Ok(map) = serde_json::from_str::<BTreeMap<String, String>>(json) else {
        return;
    };
    for (k, v) in map {
        if k == LOADED_PATH || is_transient(&k) {
            continue;
        }
        state.set_encoded(&k, v);
    }
}

/// The file that was open when the snapshot was written, if any.
pub fn last_file(json: &str) -> Option<String> {
    serde_json::from_str::<BTreeMap<String, String>>(json)
        .ok()?
        .remove(LOADED_PATH)
        .filter(|p| !p.is_empty())
}

/// The armed session file. Like the journal, only the running app arms it, so
/// a test's window persists nothing.
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
    use crate::v2::state::{CtxKey, FileKey, SceneKey, View3dKey, BKey as _};

    #[test]
    fn durable_keys_persist_and_transient_ones_do_not() {
        assert!(!is_transient("ctx.space"), "the workspace resumes");
        assert!(!is_transient("ctx.file.path"), "the browser's folder resumes");
        assert!(!is_transient("ctx.file.load_ui"), "a preference resumes");
        assert!(!is_transient(LOADED_PATH), "the last file is remembered");
        assert!(!is_transient("ctx.view3d.yaw"), "the camera now resumes (on a throttle)");
        assert!(!is_transient("ctx.time_view.dopesheet"), "and the timeline view resumes");
        assert!(!is_transient("ctx.frame"), "and the playhead resumes");

        assert!(is_transient("ctx.playing"), "but the session reopens paused");
        assert!(is_transient("ctx.frame_start"), "the scene's range is the file's, not the session's");
        assert!(is_transient("ctx.scene.loading"), "nor the import machinery");
        assert!(is_transient("ctx.scene.unsaved"), "nor a dirty flag");
        assert!(is_transient(RESUME_FRAME), "nor the resume stash");
        assert!(is_transient("ctx.image.shown"), "nor the Image editor's shown name — its list is per-session");
        assert!(is_transient("ctx.image.tool.expanded"), "nor a toolbar's expansion — it opens icon-only, as Blender's does");
        assert!(!is_transient("ctx.prefs.navbar.expanded"), "other strips keep theirs");
        assert!(is_transient("ctx.modal"), "nor an open dialog");
        assert!(is_transient("ctx.menu.view3d.View"), "nor an open menu");
        assert!(is_transient("ctx.file.selected"), "nor a mid-browse selection");
        assert!(is_transient("ctx.selected"), "nor the active selection");
    }

    #[test]
    fn a_snapshot_round_trips_the_resumable_state() {
        let mut s = BState::new();
        s.set(CtxKey::Space, "Shading");
        s.set(&FileKey::Path.path(), "/home/me/scenes");
        s.set(View3dKey::Yaw, "42"); // the camera — now resumed
        s.set_encoded("ctx.time_view.dopesheet", "0,250"); // the timeline view — now resumed
        s.set(CtxKey::Frame, "17"); // the playhead — now resumed
        s.set(CtxKey::Playing, "true"); // volatile — reopen paused
        s.set(&SceneKey::Loading.path(), "true"); // transient — must not travel
        s.set(&SceneKey::LoadedPath.path(), "/home/me/scenes/cube.blend");

        let json = snapshot(&s);
        assert!(json.contains("ctx.space"));
        assert!(json.contains("ctx.view3d.yaw"), "the camera travels: {json}");
        assert!(json.contains("ctx.time_view.dopesheet"), "the timeline view travels: {json}");
        assert!(json.contains("ctx.frame"), "the playhead travels: {json}");
        assert!(!json.contains("ctx.playing"), "but not whether it was playing: {json}");
        assert!(!json.contains("ctx.scene.loading"), "nor the flag: {json}");

        let mut restored = BState::new();
        apply(&mut restored, &json);
        assert_eq!(restored.get(CtxKey::Space), Some("Shading"));
        assert_eq!(restored.get(&FileKey::Path.path()), Some("/home/me/scenes"));
        assert_eq!(restored.get(View3dKey::Yaw), Some("42"), "the camera is restored");
        assert_eq!(restored.get_encoded("ctx.time_view.dopesheet"), Some("0,250"), "the timeline view is restored");
        assert_eq!(restored.get(CtxKey::Frame), Some("17"), "the playhead is restored");
        assert_eq!(restored.get(CtxKey::Playing), None, "but it reopens paused");
        // loaded_path is remembered for the reopen, but never written straight
        // back as if a scene were already behind it.
        assert_eq!(restored.get(&SceneKey::LoadedPath.path()), None);
        assert_eq!(last_file(&json).as_deref(), Some("/home/me/scenes/cube.blend"));
    }

    #[test]
    fn last_file_is_absent_when_nothing_was_open() {
        let s = BState::new();
        assert_eq!(last_file(&snapshot(&s)), None);
    }

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
