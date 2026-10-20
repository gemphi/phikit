//! Seed Phiber's UI state from a loaded file's view layer, so the Outliner
//! and viewport open the way Blender does: the file's active object is the
//! active row, and objects hidden in the view layer (`BASE_HIDDEN`) or living
//! in a hidden / excluded collection (`LAYER_COLLECTION_HIDE` / `_EXCLUDE`)
//! start with their eye closed; the monitor, arrow and camera toggles
//! (`Object.visibility_flag`, `Collection.flag`) start as the file has them.

use crate::v2::state::{ButsKey, CtxKey, val::BValue};
use crate::phiphx::scene::{SceneCollection, SceneModel};
use crate::v2::part::outliner::tree::k_restrict;
use crate::v2::part::text::info::{BReportLevel, push};
use crate::v2::state::{
    BState,
    val::V_TRUE,
};
use crate::v2::state::BPath;

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

/// The file's own Render-panel values — `scene["cycles"]` plus the engine —
/// written into `ctx.buts.*`, which is what the panel reads and what
/// `scene::render_set` hands the tracer.
///
/// Blender keeps no separate "session" copy of these: the file's values *are*
/// the session's until someone edits them, so opening a file that saved
/// `samples = 20` has to show 20. Only the values a file differs from the RNA
/// default on are stored (`idprop.rs`), so everything absent keeps the
/// property's own default (`BPropSpec::default`) and nothing is invented here.
///
/// The value is written in the category the *control* reads, not the one the
/// file used: an add-on may store a flag as an integer, and a checkbox parses
/// `true` / `false`.
pub fn seed_render_props(state: &mut BState, model: &SceneModel) {
    use crate::phiphx::blend::BIdValue;
    use crate::v2::part::props::prop::BPropKind;
    use crate::v2::part::props::render::spec_of;
    // The Engine row opens on the engine phiber will actually use, not the
    // one the file names. `BRenderSet::engine_at` falls back to
    // `BRenderQuality::engine_for_file`, but only when the row is unset —
    // seeding the file's raw `CYCLES` here set it, so the fallback never
    // ran and a Cycles file opened reading "Cycles" while rendering on
    // Phiray. The row and the renderer now say the same word. Workbench is
    // kept, because it is the one engine whose choice changes what is
    // drawn rather than what draws it.
    state.set(
        ButsKey::Value("engine"),
        crate::v2::scene::render_quality::BRenderQuality::engine_for_file(&model.render.engine).ident().to_string(),
    );
    let cycles = &model.render.cycles;
    for name in cycles.names() {
        let Some(value) = cycles.get(name) else { continue };
        // The panel's own spec supplies both the category to encode in and
        // the `'static` key to store under; a property it does not draw is
        // skipped rather than stored under a key nothing reads.
        let Some(spec) = spec_of(&format!("cycles.{name}")) else { continue };
        let encoded = match (spec.kind, value) {
            (BPropKind::Bool, v) => BValue::Bool(v.as_bool().unwrap_or(false)).encode(),
            (BPropKind::Int { .. }, v) => BValue::Int(i64::from(v.as_int().unwrap_or(0))).encode(),
            (BPropKind::Float { .. }, v) => BValue::Float(v.as_float().unwrap_or(0.0) as f32).encode(),
            (BPropKind::Enum(_) | BPropKind::Str, BIdValue::Str(v)) => v.clone(),
            // An add-on's enumeration is stored as the item's number
            // (`render_edit::BCyclesEnum`).
            (BPropKind::Enum(_), BIdValue::Int(i)) => match crate::v2::scene::render_edit::BCyclesEnum::name(name, *i) {
                Some(id) => id.to_string(),
                None => continue,
            },
            // A shape no control edits.
            _ => continue,
        };
        state.set(ButsKey::Value(spec.key), encoded);
    }
    // The quality default (`scene::render_quality::BRenderQuality`): a file
    // that never set Max Samples opens at the default, and the Render panel
    // SHOWS that number, so what is on screen is what will render. A count
    // the file did save is believed, as Blender believes it.
    if cycles.get("samples").is_none()
        && let Some(spec) = spec_of("cycles.samples")
    {
        let default = crate::v2::scene::render_quality::BRenderQuality::SAMPLE_FLOOR;
        state.set(ButsKey::Value(spec.key), BValue::Int(i64::from(default)).encode());
    }
}

/// The file's own Output-panel values — resolution, frame rate, and where a
/// render is written and in what format (`phiphx::BlendOutput`) — written
/// into `ctx.buts.*` under the RNA keys the Output tab reads
/// (`part/props/output.rs`).
///
/// These come out of DNA rather than ID properties, so unlike the Render
/// panel every one of them is present in every file; there is no
/// "absent means the RNA default" case to honour.
pub fn seed_output_props(state: &mut BState, model: &SceneModel) {
    use crate::v2::part::props::set::prop_named;
    let render = &model.render;
    let out = &render.output;
    let mode = match out.channel {
        1 => "BW",
        2 => "BW_A",
        3 => "RGB",
        _ => "RGBA",
    };
    // Color Management (`Scene.view_settings`), so the panel shows the
    // file's and the views read the panel (`render_set::BColorManagement`).
    let view = [
        ("view_settings.view_transform", render.view_transform.clone()),
        ("view_settings.exposure", BValue::Float(render.exposure).encode()),
        ("view_settings.gamma", BValue::Float(render.gamma).encode()),
        // Film ▸ Transparent (`Scene.r.alphamode`), on the Render tab.
        ("render.film_transparent", BValue::Bool(render.film_transparent).encode()),
    ];
    // Output ▸ Encoding (`FFMpegCodecData`), the DNA values as RNA names.
    let movie = &render.movie;
    let name = |pairs: &[(i32, &str)], v: i32| pairs.iter().find(|(k, _)| *k == v).map_or(String::new(), |(_, n)| (*n).to_string());
    let encoding = [
        ("ffmpeg.format", name(&[(0, "MPEG1"), (1, "MPEG2"), (2, "MPEG4"), (3, "AVI"), (4, "QUICKTIME"), (5, "DV"), (8, "FLASH"), (9, "MKV"), (10, "OGG"), (12, "WEBM")], movie.container)),
        ("ffmpeg.codec", name(&[(0, "NONE"), (27, "H264"), (173, "H265"), (226, "AV1"), (99, "DNXHD"), (24, "DV"), (33, "FFV1"), (21, "FLASH"), (25, "HUFFYUV"), (1, "MPEG1"), (2, "MPEG2"), (12, "MPEG4"), (61, "PNG"), (55, "QTRLE"), (30, "THEORA"), (167, "WEBM"), (147, "PRORES")], movie.codec)),
        ("ffmpeg.constant_rate_factor", name(&[(-1, "NONE"), (0, "LOSSLESS"), (17, "PERC_LOSSLESS"), (20, "HIGH"), (23, "MEDIUM"), (26, "LOW"), (29, "VERYLOW"), (32, "LOWEST")], movie.crf)),
        ("ffmpeg.ffmpeg_preset", name(&[(10, "GOOD"), (11, "BEST"), (12, "REALTIME")], movie.preset)),
        ("ffmpeg.gop_size", BValue::Int(i64::from(movie.gop)).encode()),
        ("ffmpeg.video_bitrate", BValue::Int(i64::from(movie.bitrate)).encode()),
    ];
    // Format ▸ Frame Rate as Blender stores it, `frs_sec / frs_sec_base`,
    // from the rate the file resolves to: 24 is 24 / 1, 23.98 is 24 / 1.001.
    let fps = model.timeline.fps.max(1.0);
    let frs_sec = fps.ceil();
    let pair: [(&str, String); 10] = [
        ("fps", BValue::Int(frs_sec as i64).encode()),
        ("fps_base", BValue::Float(frs_sec / fps).encode()),
        // Output ▸ Frame Range ▸ Step (Start and End are the timeline's).
        ("frame_step", BValue::Int(i64::from(model.timeline.frame_step.max(1))).encode()),
        ("resolution_x", BValue::Int(i64::from(render.resolution_x)).encode()),
        ("resolution_y", BValue::Int(i64::from(render.resolution_y)).encode()),
        ("resolution_percentage", BValue::Int(i64::from(render.resolution_percent)).encode()),
        ("filepath", out.path.clone()),
        ("image_settings.file_format", out.format.rna().to_string()),
        ("image_settings.color_mode", mode.to_string()),
        ("image_settings.color_depth", out.depth.to_string()),
    ];
    // The render region (`Scene.r.border`, Output ▸ Format).
    crate::v2::scene::render_border::BRenderBorder::seed(state, render.border);
    // The View Layer panel's pass toggles, so they show what the file asks
    // for rather than the property defaults.
    for (key, on) in crate::v2::scene::pass_state(render.pass) {
        if let Some(spec) = prop_named(key) {
            state.set(ButsKey::Value(spec.key), on);
        }
    }
    for (key, encoded) in pair.into_iter().chain(view).chain(encoding) {
        // The panel's own spec supplies the `'static` key to store under, so
        // a value never lands where no control reads it.
        let Some(spec) = prop_named(key) else { continue };
        if encoded.is_empty() {
            continue;
        }
        state.set(ButsKey::Value(spec.key), encoded);
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
