//! The JSON format: `BState` → sorted map of durable (encoded, value) pairs,
//! and back. The format is a `BTreeMap<String, String>` so a snapshot sorts
//! by key for a stable diff on disk — if two launches resume to the same
//! state the file is byte-identical.

use super::transient::{is_transient, loaded_path};
use crate::v2::state::BKey as _;
use crate::v2::state::BState;
use std::collections::BTreeMap;

/// The durable key/value pairs of `state`, as sorted JSON. Carries the
/// throttled families (camera, playhead) too; only [`is_transient`] keys are
/// dropped.
pub fn snapshot(state: &BState) -> String {
    let map: BTreeMap<&str, &str> = state
        .iter_keys()
        .filter(|k| !is_transient(k))
        .filter_map(|k| state.get_encoded(k).map(|v| (k.as_str(), v)))
        .collect();
    serde_json::to_string_pretty(&map).unwrap_or_default()
}

/// Apply a snapshot back onto `state`, skipping [`loaded_path`] (the file is
/// reopened, not faked loaded) and anything that is transient now (so a stale
/// file from an older build can never seed a key this build treats as
/// ephemeral). The camera and playhead come back here; the reopen keeps the
/// playhead via [`super::resume_frame`].
pub fn apply(state: &mut BState, json: &str) {
    let Ok(map) = serde_json::from_str::<BTreeMap<String, String>>(json) else {
        return;
    };
    // A session from before the current look keeps everything but its
    // theme: once, every window opens on the default theme (Phiber), and the
    // look it is on is written, so the user's next choice sticks.
    let stale_look = map.get(LOOK_KEY).map(String::as_str) != Some(LOOK);
    for (k, v) in map {
        if k == loaded_path() || is_transient(&k) || (stale_look && LOOK_KEYS.iter().any(|key| key.path().encode() == k)) {
            continue;
        }
        state.set_encoded(&k, v);
    }
    state.set_encoded(LOOK_KEY, LOOK);
}

/// Which look a session was saved under (`apply`). Raised when the default
/// theme changes, so every user is moved onto it once.
pub const LOOK_KEY: &str = "ctx.look";
/// The current look: 2, the Phiber theme as the default.
pub const LOOK: &str = "2";
/// What a stale look does not carry over: the theme, its mode and its coat.
const LOOK_KEYS: [crate::v2::state::AppKey; 3] = [crate::v2::state::AppKey::Theme, crate::v2::state::AppKey::ThemeMode, crate::v2::state::AppKey::Flavour];

/// The file that was open when the snapshot was written, if any.
pub fn last_file(json: &str) -> Option<String> {
    serde_json::from_str::<BTreeMap<String, String>>(json)
        .ok()?
        .remove(loaded_path().as_str())
        .filter(|p| !p.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::resume_frame;
    use crate::v2::state::{CtxKey, FileKey, SceneKey, View3dKey};

    #[test]
    fn a_session_from_an_older_look_opens_on_the_default_theme_once() {
        let mut old = BState::new();
        old.set_encoded("ctx.theme", "Silver");
        old.set(CtxKey::Space, "Shading");
        let mut s = BState::new();
        apply(&mut s, &snapshot(&old));
        assert_eq!(s.get_encoded("ctx.theme"), None, "the old theme is not restored");
        assert_eq!(s.get(CtxKey::Space), Some("Shading"), "everything else is");
        // Chosen again under the current look, it sticks.
        s.set_encoded("ctx.theme", "Silver");
        let mut again = BState::new();
        apply(&mut again, &snapshot(&s));
        assert_eq!(again.get_encoded("ctx.theme"), Some("Silver"));
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
        // `resume_frame` is still transient — a stashed frame never travels in the snapshot.
        let _ = resume_frame();
    }

    #[test]
    fn last_file_is_absent_when_nothing_was_open() {
        let s = BState::new();
        assert_eq!(last_file(&snapshot(&s)), None);
    }
}
