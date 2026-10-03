//! THYMUS poses on a thread of its own (`scene::pose`): the render pass
//! never waits for the rig — it draws what the poser has and asks for
//! another pass until the shown frame lands — and while playing, the poser
//! works on the frames after the playhead, so each is posed before the
//! playhead gets there. What it draws is what posing in place draws.

use phiber::phiphx::phx::Phx;
use phiber::phiphx::scene::{SceneMesh, SceneModel};
use phiber::v2::scene::pose::BPoser;
use phiber::v2::scene::source::{clear_loaded_scene, scene_objects, set_loaded_at};
use phiber::v2::scene::{BObject3D, current_scene};
use phiber::v2::state::{BState, CtxKey};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

const ARM: &str = "arm_lower_geo";

fn at(frame: &str, playing: Option<&str>) -> BState {
    let mut state = BState::new();
    state.set(CtxKey::Frame, frame);
    if let Some(playing) = playing {
        state.set(CtxKey::Playing, playing);
    }
    state
}

fn mesh_of(objects: &[BObject3D], name: &str) -> Arc<SceneMesh> {
    objects
        .iter()
        .find_map(|o| match o {
            BObject3D::Loaded { name: n, mesh, .. } if n == name => Some(Arc::clone(mesh)),
            _ => None,
        })
        .expect("the object")
}

/// One render pass at `state`: what it draws, and whether it asks for
/// another because the rig is still posing.
fn pass(model: &Arc<SceneModel>, state: &BState) -> (Vec<BObject3D>, bool) {
    set_loaded_at(Some(Arc::clone(model)), state);
    ((*scene_objects(state)).clone(), BPoser::waiting())
}

fn until(mut done: impl FnMut() -> bool) {
    let start = Instant::now();
    while !done() {
        assert!(start.elapsed() < Duration::from_secs(300), "the poser never got there");
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn the_rig_poses_on_its_own_thread_ahead_of_the_playhead() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../phiphx/tests/fixtures/THYMUS-BOLDROID-WAVE-003.blend");
    let model = Arc::new(Phx::open(&path).expect("THYMUS opens"));
    let read = Arc::clone(&model.objects.iter().find(|o| o.name == ARM).expect("the arm").mesh);

    // Stopped at 66: the first pass does not wait for the rig. It draws the
    // meshes as read and asks for another pass.
    let stopped = at("66", None);
    let clock = Instant::now();
    let (first, waiting) = pass(&model, &stopped);
    let passed = clock.elapsed();
    assert!(waiting, "the pass asks for another while the rig poses");
    assert!(Arc::ptr_eq(&mesh_of(&first, ARM), &read), "until the pose lands, the mesh as read");

    // Once it lands, the pass draws what posing in place draws.
    until(|| !pass(&model, &stopped).1);
    let clock = Instant::now();
    let exact = current_scene(&stopped, Some(&model));
    eprintln!("a pass waiting on the rig: {passed:?}; posing in place: {:?}", clock.elapsed());
    let (posed, want) = (mesh_of(&pass(&model, &stopped).0, ARM), mesh_of(&exact, ARM));
    assert!(posed.vertices == want.vertices && posed.indices == want.indices, "frame 66 as posed in place");
    assert!(!Arc::ptr_eq(&posed, &read), "the arm is posed");

    // Playing from 66, the poser works on 67 and 68 unasked, and the pass
    // at 67 has it at once.
    pass(&model, &at("66", Some("true")));
    until(|| BPoser::ready(&model, 67.0) && BPoser::ready(&model, 68.0));
    let playing = at("67", Some("true"));
    let (objects, waiting) = pass(&model, &playing);
    assert!(!waiting, "frame 67 was posed ahead of the playhead");
    let want = mesh_of(&current_scene(&playing, Some(&model)), ARM);
    assert!(mesh_of(&objects, ARM).vertices == want.vertices, "frame 67 as posed in place");

    // Played in reverse from the start of the range, the frames ahead wrap
    // to its end.
    let (start, end) = (model.timeline.frame_start as f32, model.timeline.frame_end as f32);
    let mut reverse = at(&start.to_string(), Some("reverse"));
    reverse.set(CtxKey::FrameStart, start.to_string());
    reverse.set(CtxKey::FrameEnd, end.to_string());
    pass(&model, &reverse);
    until(|| BPoser::ready(&model, end) && BPoser::ready(&model, end - 1.0));
    clear_loaded_scene();
}

#[test]
fn a_playhead_faster_than_the_poser_still_shows_the_rig_moving() {
    // A debug build poses THYMUS slower than playback steps the playhead,
    // and a late tick steps several frames at once (`app::playback`). The
    // frame the pass wants is then never the one the poser just finished —
    // but what it finished is newer than what is drawn, and must be shown,
    // or the character never moves while playing.
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../phiphx/tests/fixtures/THYMUS-BOLDROID-WAVE-003.blend");
    let model = Arc::new(Phx::open(&path).expect("THYMUS opens"));
    let read = Arc::clone(&model.objects.iter().find(|o| o.name == ARM).expect("the arm").mesh);
    let (start, end) = (model.timeline.frame_start, model.timeline.frame_end);
    let mut frame = start + 10;
    let mut shown = Vec::new();
    let clock = Instant::now();
    while clock.elapsed() < Duration::from_secs(20) && shown.len() < 3 {
        let mut state = at(&frame.to_string(), Some("true"));
        state.set(CtxKey::FrameStart, start.to_string());
        state.set(CtxKey::FrameEnd, end.to_string());
        let arm = mesh_of(&pass(&model, &state).0, ARM);
        if !Arc::ptr_eq(&arm, &read) && !shown.iter().any(|m: &Arc<SceneMesh>| Arc::ptr_eq(m, &arm)) {
            shown.push(arm);
        }
        // Faster than any pose: several frames a pass, as a late tick steps.
        frame = if frame + 7 > end { start + 10 } else { frame + 7 };
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(shown.len() >= 3, "the arm was drawn in {} poses while playing", shown.len());
    clear_loaded_scene();
}
