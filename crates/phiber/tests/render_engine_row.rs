//! Properties ▸ Render ▸ Engine reaches a render. The row was a dropdown
//! nothing read: a render asked the *file* which engine it was saved with,
//! so changing the row changed nothing — and `PHIRAY`, which no `.blend`
//! can name, was selectable with nothing behind it at all.
//!
//! `BRenderSet` carries the engine now, the panel's value first and the
//! file's only until the panel holds one, which is how every other row in
//! that struct reads (`scene::render_engine`).

use phiber::v2::scene::render_engine::BRenderEngine;
use phiber::v2::scene::render_set;
use phiber::v2::state::{BState, ButsKey};

#[test]
fn the_engine_row_decides_what_a_render_draws_with() {
    let mut s = BState::new();
    // Nothing chosen and no file: Phiray, the one engine phiber has for a
    // traced frame (`scene::render_quality::BRenderQuality::ENGINE`). It
    // used to read EEVEE, a name phiber cannot actually run.
    assert_eq!(render_set(&s).engine, BRenderEngine::Phiray);
    assert!(render_set(&s).engine.traces(), "Phiray path-traces");

    for (chosen, want) in [
        ("CYCLES", BRenderEngine::Cycles),
        ("PHIRAY", BRenderEngine::Phiray),
        ("BLENDER_WORKBENCH", BRenderEngine::Workbench),
        ("BLENDER_EEVEE_NEXT", BRenderEngine::Eevee),
    ] {
        s.set(ButsKey::Value("engine"), chosen);
        assert_eq!(render_set(&s).engine, want, "the row's value is the engine: {chosen}");
    }

    // Workbench is the one engine that does not path-trace: it rasterizes
    // `Scene.display.shading` (`workbench_render.cc`).
    s.set(ButsKey::Value("engine"), "BLENDER_WORKBENCH");
    assert!(!render_set(&s).engine.traces());
    s.set(ButsKey::Value("engine"), "PHIRAY");
    assert!(render_set(&s).engine.traces(), "Phiray is the tracer, named as itself");

    // An engine the build does not know falls back to EEVEE rather than
    // refusing to render, as Blender does with a missing engine. The row
    // was *chosen*, so it is taken at its word and traced.
    s.set(ButsKey::Value("engine"), "SOME_ADDON_ENGINE");
    assert_eq!(render_set(&s).engine, BRenderEngine::Eevee);

    // An empty row is "the panel has never held one", not "no engine", so
    // the default applies.
    s.set(ButsKey::Value("engine"), "");
    assert_eq!(render_set(&s).engine, BRenderEngine::Phiray);

    assert_eq!(
        [BRenderEngine::Eevee, BRenderEngine::Workbench, BRenderEngine::Cycles, BRenderEngine::Phiray].map(BRenderEngine::label),
        ["EEVEE", "Workbench", "Cycles", "Phiray"],
        "the names the Engine row and the Info report show",
    );
}

/// The parity the row must not break: a file saved under Workbench still
/// renders Workbench with the panel untouched, and the row then overrides
/// it. The seed writes the file's engine into the row when a file opens
/// (`app::seed`), so this is the path a file takes before any seeding —
/// and the one that used to be the *only* path.
#[test]
fn an_untouched_row_is_the_files_engine_and_a_chosen_one_overrides_it() {
    use phiber::phiphx::scene::SceneModel;
    let mut model = SceneModel::default();
    model.render.engine = "BLENDER_WORKBENCH".into();
    phiber::v2::scene::source::set_loaded_scene(Some(model));
    let mut s = BState::new();
    assert_eq!(render_set(&s).engine, BRenderEngine::Workbench, "the file's, with the row untouched");
    s.set(ButsKey::Value("engine"), "PHIRAY");
    assert_eq!(render_set(&s).engine, BRenderEngine::Phiray, "the row wins once it holds a value");
    phiber::v2::scene::source::set_loaded_scene(None);
}

/// The two rules the command line already had and the app did not: a file
/// that names an engine phiber cannot run is rendered by the tracer, and a
/// final render takes at least the sample floor whatever the file saved.
/// Pose 26 (`CHATTY-POSES-RENDER-SCENE-03_Duo pose 26_1.blend`) is the case
/// that found this: `CYCLES` at **20 samples**, which rendered as twenty.
#[test]
fn a_files_engine_and_sample_count_meet_the_quality_policy() {
    use phiber::phiphx::scene::SceneModel;
    use phiber::v2::scene::render_quality::BRenderQuality;

    let mut model = SceneModel::default();
    model.render.engine = "CYCLES".into();
    phiber::v2::scene::source::set_loaded_scene(Some(model));
    let s = BState::new();
    assert_eq!(render_set(&s).engine, BRenderEngine::Phiray, "a Cycles file is traced by phiray, and says so");
    phiber::v2::scene::source::set_loaded_scene(None);

    // The floor itself, where both the CLI and F12 now read it.
    assert_eq!(BRenderQuality::ENGINE, "PHIRAY");
    assert_eq!(BRenderQuality::SAMPLE_FLOOR, 2048);
    assert_eq!(BRenderQuality::samples(Some(20), false), 2048, "pose 26's twenty is lifted");
    assert_eq!(BRenderQuality::samples(Some(20), true), 20, "--draft still renders twenty");
}
