//! The file's own Render- and Output-panel values seeded into `ctx.buts.*`
//! (split from `seed.rs`, which seeds the view layer and reports the open).

use crate::phiphx::scene::SceneModel;
use crate::v2::state::{BState, ButsKey, val::BValue};

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
        ("view_settings.look", render.look.clone()),
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
