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
    // Nothing chosen and no file: EEVEE, as a fresh Blender scene names.
    assert_eq!(render_set(&s).engine, BRenderEngine::Eevee);
    assert!(render_set(&s).engine.traces(), "EEVEE path-traces here");

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
    // refusing to render, as Blender does with a missing engine.
    s.set(ButsKey::Value("engine"), "SOME_ADDON_ENGINE");
    assert_eq!(render_set(&s).engine, BRenderEngine::Eevee);

    // An empty row is "the panel has never held one", not "no engine".
    s.set(ButsKey::Value("engine"), "");
    assert_eq!(render_set(&s).engine, BRenderEngine::Eevee);

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
