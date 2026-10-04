//! The Render and Output tabs reach the file: the engine, the Cycles
//! settings the panel draws (ID properties on `scene["cycles"]`, under the
//! file's system-properties root where its layout has one) and the output
//! path and format are what Ctrl S writes, and what the file shows when it
//! is opened again — here, and in Blender (`phiphx/tests/blender_opens_
//! saves.rs`). They used to be seeded from the file and never written back.

use gpui::{TestAppContext, px, size};
use phiber::phiphx::blend::{BlendFile, BlendImageFormat};
use phiber::phiphx::scene::SceneModel;
use phiber::v2::app::BPhiberApp;
use phiber::v2::state::val::BValue;
use phiber::v2::state::{BKey as _, ButsKey, ModalKey};
use phiber::v2::view::BAction;
use std::path::PathBuf;

fn model_of(path: &std::path::Path) -> SceneModel {
    let scene = BlendFile::parse_bytes(&std::fs::read(path).unwrap()).unwrap().extract_scene().unwrap();
    SceneModel::from_blend_owned(scene, &path.to_string_lossy())
}

#[gpui::test]
async fn the_render_and_output_tabs_are_written_by_ctrl_s(cx: &mut TestAppContext) {
    let dir = std::env::temp_dir().join(format!("phiber_render_save_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let blend = dir.join("cube.blend");
    // A 5.0 file: its Cycles settings live under `ID.system_properties`.
    std::fs::copy(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/blend/startup-5.blend"), &blend).unwrap();
    let model = model_of(&blend);
    assert!(model.render.cycles.get("samples").is_none(), "the factory file carries no sample count");

    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state.set(ModalKey::Open, "");
        app.state.set(&phiber::v2::scene::loaded_path_key(), blend.to_string_lossy().to_string());
        phiber::v2::app::seed::seed_render_props(&mut app.state, &model);
        phiber::v2::app::seed::seed_output_props(&mut app.state, &model);
        app.scene_cache = Some((blend.to_string_lossy().to_string(), std::sync::Arc::new(model)));
        cx.notify();
    });
    cx.run_until_parked();

    // The Render tab: Cycles, 48 samples, Denoise off, Diffuse bounces 2;
    // the Output tab: a path, OpenEXR at 16 bits, RGB.
    let set = |key: &'static str, v: String| BAction::Set(ButsKey::Value(key).path(), v);
    app.update(cx, |app, cx| {
        for action in [
            set("engine", "CYCLES".into()),
            set("render.film_transparent", BValue::Bool(true).encode()),
            set("cycles.samples", BValue::Int(48).encode()),
            set("cycles.use_denoising", BValue::Bool(false).encode()),
            set("cycles.diffuse_bounces", BValue::Int(2).encode()),
            set("cycles.pixel_filter_type", BValue::Enum("GAUSSIAN".into()).encode()),
            set("filepath", "//renders/shot_".into()),
            set("image_settings.file_format", "OPEN_EXR".into()),
            set("image_settings.color_depth", "16".into()),
            set("image_settings.color_mode", "RGB".into()),
        ] {
            app.handle_action(&action, cx);
        }
    });
    cx.run_until_parked();
    cx.simulate_keystrokes("ctrl-s");
    cx.run_until_parked();

    let saved = model_of(&blend);
    let cycles = |name: &str| saved.render.cycles.get(name).and_then(|v| v.as_int());
    assert_eq!(saved.render.engine, "CYCLES");
    assert!(saved.render.film_transparent, "Film ▸ Transparent (`r.alphamode`)");
    // And the render plan carries it to the tracer (`Camera::film`).
    let look = phiber::v2::view::draw::canvas::view3d::rendered::BRayLook { sky: gpui::black(), grid: gpui::white(), grid_hide: 0, world: None, lights: Vec::new(), wire: None, view: [0.0, 0.0, 1.0, 0.0], environment: None, set: Default::default(), studio: false };
    let set = app.read_with(cx, |a, _| phiber::v2::scene::render_set(&a.state));
    assert!(set.film_transparent);
    assert_eq!(phiber::v2::scene::render_plan(&saved, set, &look, gpui::white()).expect("the factory camera").camera.film, [1, 0, 0, 0]);
    assert_eq!((cycles("samples"), cycles("use_denoising"), cycles("diffuse_bounces")), (Some(48), Some(0), Some(2)), "Ctrl S wrote the Cycles settings");
    assert_eq!(cycles("pixel_filter_type"), Some(1), "the enumeration as Blender's number (Gaussian)");
    // Opened again, the panel shows Gaussian.
    let mut again = phiber::v2::state::BState::new();
    phiber::v2::app::seed::seed_render_props(&mut again, &saved);
    assert_eq!(again.get(ButsKey::Value("cycles.pixel_filter_type")), Some("GAUSSIAN"));
    assert_eq!(saved.render.cycles.get("sampling_pattern").and_then(|v| v.as_int()), Some(1), "what the file already carried is kept");
    let out = &saved.render.output;
    assert_eq!((out.path.as_str(), out.format, out.depth, out.channel), ("//renders/shot_", BlendImageFormat::OpenExr, 16, 3), "Ctrl S wrote the Output tab");
    let _ = std::fs::remove_dir_all(&dir);
}
