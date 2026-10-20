//! Image → Save As actually writes the Render Result.
//!
//! The writer and the Output reader are covered elsewhere; what this covers
//! is the wiring between them — that the menu item opens the browser where
//! Properties → Output points, and that committing a name there produces a
//! file in the panel's format rather than in the typed extension's.
//!
//! The render job is process-wide, so the parts run in sequence inside one
//! test rather than as separate ones racing for it.

use phiber::phiphx::scene::SceneCamera;
use phiber::v2::scene::{BObject3D, BRenderPlan, BShading, camera_of, output_set, render_job};
use phiber::v2::view::draw::canvas::view3d::rendered::BStillLook;
use phiber::v2::state::{BKey as _, BState, ButsKey, FileKey, action::apply_action};
use phiber::v2::view::prim::BAction;

/// Drive a key the way a click does: the reducer, not a bare store write, is
/// what runs the rules this test is about.
fn click(state: &mut BState, key: FileKey, value: &str) {
    apply_action(state, &BAction::Set(key.path(), value.to_string()));
}

fn a_state_pointing_at(folder: &std::path::Path, format: &str) -> BState {
    let mut state = BState::new();
    // What `seed_output_props` writes when a file is opened.
    state.set(ButsKey::Value("filepath"), folder.join("shot").to_string_lossy().to_string());
    state.set(ButsKey::Value("image_settings.file_format"), format);
    state.set(ButsKey::Value("image_settings.color_depth"), "32");
    state.set(ButsKey::Value("image_settings.color_mode"), "RGB");
    state
}

fn a_cube() -> Vec<BObject3D> {
    vec![BObject3D::Primitive { name: "Cube".into(), shape: phiber::v2::scene::BShape::Cube, x: 0.0, y: 0.0 }]
}

fn a_plan(samples: u32) -> BRenderPlan {
    let cam = SceneCamera {
        name: "Camera".into(),
        location: [0.0, -4.0, 0.0],
        matrix_world: [[1.0, 0.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, -1.0, 0.0, 0.0], [0.0, -4.0, 0.0, 1.0]],
        kind: "PERSP".into(),
        focal_length_mm: 50.0,
        sensor_width: 36.0,
        ortho_scale: 6.0,
        clip_start: 0.1,
        clip_end: 100.0,
        display_size: 1.0,
        dof: Default::default(),
    };
    BRenderPlan { camera: camera_of(&cam, 16, 16), w: 16, h: 16, samples, denoise: false, adapt: [0.0; 4], time_limit: 0.0, look: BStillLook::new(BShading::Rendered, gpui::hsla(0.0, 0.0, 0.5, 1.0)), region: Default::default() }
}

/// Wait for the job to go idle, so the next `start` is not refused.
fn settled() {
    for _ in 0..600 {
        if !render_job::rendering() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    panic!("the render never stopped");
}

/// Start a render of `samples` and return once the job is running, without
/// waiting for it to finish.
fn started(samples: u32) -> bool {
    if phiray::Ray::gpu().is_none() {
        return false;
    }
    render_job::start(a_cube(), a_plan(samples));
    true
}

/// Render to completion.
fn a_render() -> bool {
    if !started(2) {
        return false;
    }
    for _ in 0..600 {
        if render_job::shot().0.is_some_and(|shot| shot.done && shot.has_image()) {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    panic!("the render never finished");
}

#[test]
fn save_as_opens_where_output_points_and_writes_that_format() {
    let folder = std::env::temp_dir().join("phiber_save_as");
    let _ = std::fs::remove_dir_all(&folder);
    std::fs::create_dir_all(&folder).unwrap();
    let mut state = a_state_pointing_at(&folder, "OPEN_EXR");

    // The panel round-trips: what the state holds reads back as an EXR.
    assert_eq!(output_set(&state).format, phiphx::blend::BlendImageFormat::OpenExr);
    assert_eq!(output_set(&state).depth, 32);

    // Image → Save As sets the purpose, which raises the browser at the
    // Output folder with the name the panel would use — not the home folder.
    click(&mut state, FileKey::Purpose, "save_image");
    assert_eq!(state.get(FileKey::Filename), Some("shot.exr"), "the browser should be pre-filled from the Output path");
    assert_eq!(state.get(FileKey::Path).map(std::path::PathBuf::from), Some(folder.clone()), "the browser should open in the Output folder");

    // Committing before anything has been rendered reports rather than
    // writing a black file — and still closes the browser, so the next Save
    // As starts clean. This runs before the render because the job is
    // process-wide and there is no way back to "nothing rendered".
    assert!(render_job::shot().0.is_none(), "another test in this binary rendered first");
    click(&mut state, FileKey::Selected, &folder.join("early.exr").to_string_lossy());
    assert!(!folder.join("early.exr").exists(), "a save with no render should write nothing");
    assert_eq!(state.get(FileKey::Purpose), None);

    // The same refusal mid-render: the job publishes its target before the
    // first sample, so there is a shot with a sample count and no pixels,
    // and saving that would write a black file.
    if !started(4000) {
        return;
    }
    assert!(render_job::shot().0.is_some_and(|shot| !shot.has_image()), "the job should publish its target before the first sample");
    click(&mut state, FileKey::Purpose, "save_image");
    click(&mut state, FileKey::Selected, &folder.join("partial.exr").to_string_lossy());
    assert!(!folder.join("partial.exr").exists(), "a render still in flight should not be written");
    render_job::cancel();
    settled();

    if !a_render() {
        return;
    }
    click(&mut state, FileKey::Purpose, "save_image");

    // Committing a name writes the file. The typed name wins, the panel's
    // format still decides the extension — as in Blender, where changing the
    // name does not change the format.
    click(&mut state, FileKey::Selected, &folder.join("beauty.png").to_string_lossy());
    let written = folder.join("beauty.exr");
    assert!(written.exists(), "nothing was written; the folder holds {:?}", std::fs::read_dir(&folder).unwrap().flatten().map(|e| e.file_name()).collect::<Vec<_>>());
    assert!(!folder.join("beauty.png").exists(), "the typed extension should not override the panel's format");

    // It really is an EXR, and it really is the render.
    let read = image::open(&written).unwrap().to_rgb32f();
    assert_eq!(read.dimensions(), (16, 16));
    assert!(read.pixels().any(|px| px.0.iter().any(|v| *v > 0.0)), "the saved EXR is entirely black");

    // The browser closes and the purpose is dropped, so the next Save As
    // starts clean rather than re-committing this one.
    assert_eq!(state.get(FileKey::Purpose), None);
}
