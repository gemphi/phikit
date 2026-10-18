//! A file's own Render settings reach the panel.
//!
//! Cycles stores its settings as ID properties on the scene
//! (`scene["cycles"]`), and only the ones that differ from the RNA default
//! are written. So opening Female-01, which saved `samples = 20`, has to show
//! 20 — while `diffuse_bounces`, which the file never wrote, stays at
//! Blender's 4.

use phiber::phiphx::Phx;
use phiber::v2::app::seed::seed_render_props;
use phiber::v2::scene::render_set;
use phiber::v2::state::BState;

const FILE: &str = "assets/CW-Fashion/Epic-Pods-Female-01/Female-01-Chatty-Whims-Rigged-Lace.blend";

fn opened() -> Option<(BState, phiber::phiphx::scene::SceneModel)> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(FILE);
    let model = Phx::open(&path).ok()?;
    let mut state = BState::new();
    seed_render_props(&mut state, &model);
    Some((state, model))
}

#[test]
fn the_scenes_cycles_group_is_read() {
    let Some((_, model)) = opened() else { return };
    let cycles = &model.render.cycles;
    assert_eq!(cycles.get("samples").and_then(|v| v.as_int()), Some(20));
    assert_eq!(cycles.get("preview_samples").and_then(|v| v.as_int()), Some(20));
    // What the file does not store is absent, not zero — that distinction is
    // the whole reason the panel keeps RNA defaults.
    assert!(cycles.get("diffuse_bounces").is_none());
}

#[test]
fn the_panel_shows_the_files_samples_and_blenders_defaults_elsewhere() {
    let Some((state, _)) = opened() else { return };
    let set = render_set(&state);
    assert_eq!(set.samples, 20, "the file's Max Samples did not reach the panel");
    assert_eq!(set.preview_samples, 20);
    // Untouched by the file, so Blender's own defaults stand.
    assert_eq!(set.diffuse_bounces, 4);
    assert!(set.denoise);
}

#[test]
fn the_engine_comes_from_the_file() {
    let Some((state, model)) = opened() else { return };
    assert_eq!(model.render.engine, "CYCLES");
    assert_eq!(
        state.get(phiber::v2::state::ButsKey::Value("engine")),
        Some("CYCLES"),
        "Render Engine does not show what the file saved"
    );
}

/// The quality floor reaches the panel when a file opens, so Max Samples
/// shows the number that will actually render. Pose 26
/// (`CHATTY-POSES-RENDER-SCENE-03_Duo pose 26_1.blend`) is saved at **20**
/// samples — a turntable preview — and rendered as twenty, which is what
/// sent its Render Result out noisy.
#[test]
fn a_files_sample_count_opens_at_the_quality_floor_and_a_typed_one_still_wins() {
    use phiber::phiphx::blend::{BIdGroup, BIdValue};
    use phiber::phiphx::scene::SceneModel;
    use phiber::v2::scene::render_quality::BRenderQuality;
    use phiber::v2::scene::render_set;
    use phiber::v2::state::{BState, ButsKey, val::BValue};

    let seeded = |saved: i32| {
        let mut model = SceneModel::default();
        let mut cycles = BIdGroup::default();
        cycles.set("samples", BIdValue::Int(saved));
        model.render.cycles = cycles;
        let mut s = BState::new();
        phiber::v2::app::seed::seed_render_props(&mut s, &model);
        phiber::v2::scene::source::set_loaded_scene(Some(model));
        let samples = render_set(&s).samples;
        phiber::v2::scene::source::set_loaded_scene(None);
        (s, samples)
    };

    // Below the floor: lifted, and the panel holds the lifted value.
    let (s, samples) = seeded(20);
    assert_eq!(samples, BRenderQuality::SAMPLE_FLOOR, "pose 26's twenty opens at the floor");
    assert_eq!(
        s.get(ButsKey::Value("cycles.samples")),
        Some(BValue::Int(i64::from(BRenderQuality::SAMPLE_FLOOR)).encode()).as_deref(),
        "and Max Samples SHOWS it, so the panel and the render agree",
    );

    // Above the floor: the file's own count is kept, not lowered.
    let (_, samples) = seeded(4096);
    assert_eq!(samples, 4096, "a file that asks for more than the floor keeps it");

    // Typed over afterwards: the panel wins, so a quick look is still possible.
    let (mut s, _) = seeded(20);
    s.set(ButsKey::Value("cycles.samples"), BValue::Int(64).encode());
    assert_eq!(render_set(&s).samples, 64, "a count typed into the panel is the count that renders");
}
