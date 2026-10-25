//! Seed Phiber's UI state from a loaded file's view layer, so the Outliner
//! and viewport open the way Blender does: the file's active object is the
//! active row, and objects hidden in the view layer (`BASE_HIDDEN`) or living
//! in a hidden / excluded collection (`LAYER_COLLECTION_HIDE` / `_EXCLUDE`)
//! start with their eye closed; the monitor, arrow and camera toggles
//! (`Object.visibility_flag`, `Collection.flag`) start as the file has them.

use crate::v2::state::CtxKey;
use crate::phiphx::scene::{SceneCollection, SceneModel};
use crate::v2::part::outliner::tree::k_restrict;
use crate::v2::part::text::info::{BReportLevel, push};
use crate::v2::state::{
    BState,
    val::V_TRUE,
};
use crate::v2::state::BPath;

mod props;
pub use props::{seed_output_props, seed_render_props};

/// A hidden or excluded collection closes its own eye, and the viewport
/// inherits that down the tree (`outliner::layer::hidden_by_collections`).
/// Marking every object under it individually would leave them hidden after
/// the collection's eye was opened again.
fn seed_collections(state: &mut BState, c: &SceneCollection, path: &str) {
    let id = crate::v2::part::outliner::layer::collection_id(path);
    if c.hide_viewport {
        state.set(&k_restrict(id, "hide_get"), V_TRUE);
    }
    // The checkbox: `LAYER_COLLECTION_EXCLUDE`, its own key so it saves back
    // as itself rather than as a closed eye.
    if c.exclude {
        state.set(&k_restrict(id, "exclude"), V_TRUE);
    }
    seed_visibility(state, id, c.visibility);
    for k in &c.children {
        seed_collections(state, k, &format!("{path}/{}", k.name));
    }
}

/// The monitor, arrow and camera of one row, as the file has them.
fn seed_visibility(state: &mut BState, id: &str, v: crate::phiphx::blend::BlendVisibility) {
    for (on, field) in [(v.hide_viewport, "hide_viewport"), (v.hide_select, "hide_select"), (v.hide_render, "hide_render")] {
        if on {
            state.set(&k_restrict(id, field), V_TRUE);
        }
    }
}


pub fn seed_view_layer(state: &mut BState, model: &SceneModel) {
    clear_restrictions(state);
    if let Some(active) = &model.selection.active {
        state.set(CtxKey::Object, active.clone());
        // A plain `set` runs no rule, so the Object and Material tabs are
        // filled here; `BActiveObjectRule` keeps them following the active
        // object afterwards.
        crate::v2::part::props::active::seed(state, model, active);
    }
    for name in &model.selection.hidden {
        state.set(&k_restrict(name, "hide_get"), V_TRUE);
    }
    for name in &model.selection.viewport_disabled {
        state.set(&k_restrict(name, "hide_viewport"), V_TRUE);
    }
    for name in &model.selection.unselectable {
        state.set(&k_restrict(name, "hide_select"), V_TRUE);
    }
    for name in &model.selection.render_disabled {
        state.set(&k_restrict(name, "hide_render"), V_TRUE);
    }
    for c in &model.collections {
        seed_collections(state, c, &c.name);
    }
}

/// Every row's restriction flags from the file open before, dropped. The
/// flags are the file's (`outliner::key::is_restriction`), and the seeding
/// above only ever SETS one, so a flag the last file set — or a stale
/// session wrote — would otherwise stay set under the same row name, and
/// every file's master collection is named `col/Scene Collection`.
fn clear_restrictions(state: &mut BState) {
    let stale: Vec<String> = state.iter_keys().filter(|k| crate::v2::part::outliner::key::is_restriction(k.as_str())).cloned().collect();
    for key in stale {
        state.remove_encoded(&key);
    }
}

/// Status bar + Info log after a successful read — Blender's
/// `file_read_reports_finalize` line and the `info_stats` string.
pub fn report_loaded(state: &mut BState, model: &SceneModel) {
    let r = &model.report;
    if let Some(path) = &model.source_path {
        push(state, BReportLevel::Info, format!("Read blend: \"{path}\""));
    }
    push(state, BReportLevel::Info, r.duration_line());
    push(
        state,
        BReportLevel::Info,
        format!("Scene \"{}\" (file version {}) · {}", r.scene, r.file_version, r.stats_line()),
    );
    for w in &r.warnings {
        push(state, BReportLevel::Warning, w.clone());
    }
    state.set(CtxKey::Scene, r.scene.clone());
    let t = &model.timeline;
    state.set(CtxKey::Frame, t.current_frame.to_string());
    state.set(CtxKey::FrameStart, t.frame_start.to_string());
    state.set(CtxKey::FrameEnd, t.frame_end.to_string());
    state.set(CtxKey::Fps, t.fps.to_string());
    state.set(&BPath::owned(vec!["stats"]), r.stats_line());
}

/// Info log entry for a failed read (Blender: `Cannot read file "…": …`).
pub fn report_failed(state: &mut BState, path: Option<&str>, message: &str) {
    let text = match path {
        Some(p) => format!("Cannot read file \"{p}\": {message}"),
        None => message.to_string(),
    };
    push(state, BReportLevel::Error, text);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::state::ButsKey;
    use crate::v2::part::outliner::key::{is_render_hidden, is_restriction};

    /// The bug on pose 26: a session had `col/Scene Collection.hide_render`
    /// from an earlier file, the next file's master collection has the same
    /// id, and the seed only ever sets flags — so every file opened with
    /// its master camera off and every render was handed nothing.
    #[test]
    fn a_file_open_starts_every_restriction_from_the_file() {
        let mut state = BState::new();
        state.set(&k_restrict("col/Scene Collection", "hide_render"), V_TRUE);
        state.set(&k_restrict("Camera.004", "hide_viewport"), V_TRUE);
        state.set(&k_restrict("col", "hide_render"), V_TRUE);
        let model = SceneModel { collections: vec![SceneCollection { name: "Scene Collection".into(), ..Default::default() }], ..Default::default() };
        seed_view_layer(&mut state, &model);
        assert!(!is_render_hidden(&state, "col/Scene Collection"), "the file says the master renders");
        assert!(!state.has(&k_restrict("Camera.004", "hide_viewport"), V_TRUE), "another file's object is forgotten");
        assert!(state.has(&k_restrict("col", "hide_render"), V_TRUE), "the column toggle is UI and stays");
        assert!(is_restriction("ctx.outliner.col/Scene Collection/SHOES.exclude") && !is_restriction("ctx.outliner.search"));
    }

    /// Format ▸ Frame Rate shows the file's rate as Blender stores it; it
    /// read 1 and 0 whatever the file said.
    #[test]
    fn frame_rate_shows_the_files_rate() {
        let rate = |fps: f32| {
            let mut state = BState::new();
            let mut model = SceneModel::default();
            model.timeline.fps = fps;
            seed_output_props(&mut state, &model);
            (state.get(ButsKey::Value("fps")).map(str::to_string), state.get(ButsKey::Value("fps_base")).and_then(|v| v.parse::<f32>().ok()))
        };
        assert_eq!(rate(24.0), (Some("24".to_string()), Some(1.0)));
        let (frs, base) = rate(24.0 / 1.001);
        assert_eq!(frs.as_deref(), Some("24"));
        assert!(base.is_some_and(|b| (b - 1.001).abs() < 1e-4), "23.98 is 24 over 1.001: {base:?}");
    }
}
