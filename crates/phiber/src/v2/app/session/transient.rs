//! Which keys resume on the next launch, and which don't. The two speeds are
//! families, not individual keys:
//!
//! - **Durable** state (the layout, panels, shading, the browser, the last
//!   file) persists on every change.
//! - **Throttled** families — the viewport camera and the timeline editors'
//!   view (`view3d`, `time_view`), and the playhead (`frame`) — change many
//!   times a second but still resume; `BSession::persist` writes them at most
//!   once per [`THROTTLE`] window, so an orbit, a scrub or a playing clock
//!   does not rewrite the file every frame. They are not transient — the
//!   snapshot carries them and the launch applies them.
//! - **Volatile** families — the transport flags and the scene's own
//!   playback settings (range, fps, markers, key drags) — are neither
//!   persisted nor resumed. The range and fps belong to the `.blend` and come
//!   back when it reopens; the rest are mid-interaction.
//!
//! `time_view` is pure UI — no file-load path seeds it — so it resumes on the
//! Load-UI-off reopen with no stash, unlike the playhead (which does need a
//! stash because the file carries its own saved frame; see `resume_frame`).

use std::time::Duration;

pub(super) const THROTTLED: &[&str] = &["view3d", "time_view", "frame"];

pub(super) const VOLATILE: &[&str] = &[
    "playing", "frame_step", "frame_start", "frame_end", "fps",
    "play_from", "auto_key", "preview_range", "preview_start", "preview_end",
    "markers", "keys",
];

/// How often the throttled families may rewrite the session file. A durable
/// change ignores this and writes at once.
pub(super) const THROTTLE: Duration = Duration::from_millis(1000);

/// The one `scene` key that is a durable session fact rather than scene data
/// or a transient flag: the path of the file that was open. Kept so the launch
/// knows what to reopen; applied as a *reopen*, never written straight back
/// (that would claim a file was loaded with no scene behind it).
pub fn loaded_path() -> String {
    crate::v2::scene::loaded_path_key().encode()
}

/// Where a resumed session stashes its playhead, so the reopen's
/// `settle_import` lands on it rather than on the file's saved frame. Under
/// `ctx.scene.`, so it is transient and never itself persisted.
pub fn resume_frame() -> String {
    use crate::v2::state::{BKey as _, SceneKey};
    SceneKey::ResumeFrame.path().encode()
}

/// The coarse family of an encoded key — the segment after `ctx.`.
fn family(encoded: &str) -> &str {
    encoded.strip_prefix("ctx.").unwrap_or(encoded).split('.').next().unwrap_or("")
}

/// True when `encoded` is NOT part of the resumable session: a volatile
/// family, an open menu / header popover / dialog, an area-resize drag, the
/// import machinery or a scene-data edit, or a mid-interaction file-browser
/// key. Everything else — the workspace, panels, shading, overlays, the file
/// browser's folder and preferences, the camera and playhead ([`THROTTLED`]),
/// and [`loaded_path`] — is durable.
pub fn is_transient(encoded: &str) -> bool {
    // The last file is the one durable `scene` key; it overrides the blanket
    // scene exclusion below.
    if encoded == loaded_path() {
        return false;
    }
    if VOLATILE.contains(&family(encoded)) {
        return true;
    }
    // THROTTLED families (the camera, the playhead) ARE resumable — they are
    // not transient, they just persist on a throttle (see `persist`).
    super::super::local_keys::never_resumed(encoded)
        // A toolbar's expansion (`ctx.<host>.tool.expanded`): Blender's
        // toolbar opens icon-only — labels only once it is dragged wide — so
        // the squares come back at launch whatever was left (`part::strip`).
        || encoded.ends_with(".tool.expanded")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn durable_keys_persist_and_transient_ones_do_not() {
        assert!(!is_transient("ctx.space"), "the workspace resumes");
        assert!(!is_transient("ctx.file.path"), "the browser's folder resumes");
        assert!(!is_transient("ctx.file.load_ui"), "a preference resumes");
        assert!(!is_transient(&loaded_path()), "the last file is remembered");
        assert!(!is_transient("ctx.view3d.yaw"), "the camera now resumes (on a throttle)");
        assert!(!is_transient("ctx.time_view.dopesheet"), "and the timeline view resumes");
        assert!(!is_transient("ctx.frame"), "and the playhead resumes");

        assert!(is_transient("ctx.playing"), "but the session reopens paused");
        assert!(is_transient("ctx.frame_start"), "the scene's range is the file's, not the session's");
        assert!(is_transient("ctx.scene.loading"), "nor the import machinery");
        assert!(is_transient("ctx.scene.unsaved"), "nor a dirty flag");
        assert!(is_transient(&resume_frame()), "nor the resume stash");
        assert!(is_transient("ctx.image.shown"), "nor the Image editor's shown name — its list is per-session");
        assert!(is_transient("ctx.image.tool.expanded"), "nor a toolbar's expansion — it opens icon-only, as Blender's does");
        assert!(!is_transient("ctx.prefs.navbar.expanded"), "other strips keep theirs");
        assert!(is_transient("ctx.modal"), "nor an open dialog");
        assert!(is_transient("ctx.menu.view3d.View"), "nor an open menu");
        assert!(is_transient("ctx.file.selected"), "nor a mid-browse selection");
        assert!(is_transient("ctx.selected"), "nor the active selection");
    }
}
