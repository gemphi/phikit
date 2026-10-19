//! The scene at the frame the timeline shows.
//!
//! Blender's depsgraph re-evaluates every animated object when `cfra`
//! changes and the viewport draws the result; here the frame is `ctx.frame`
//! and the result is the file's motion list solved at it
//! (`SceneModel::motion`). A still scene is the same at every frame, so it
//! never re-solves: `moving_frame` is `None` for it and every memo keyed on
//! the frame stays put while the playhead runs.
//!
//! The frame is scoped per render pass like the loaded model
//! (`source::set_loaded_shared`), so the readers that have no state in hand
//! — the tracer's lights, the overlay's cameras and empties — pose the same
//! frame the object list did — and, when the Transform panel or G / R / S
//! moved a camera, light or empty (or its parent), where the edits put it
//! (`transform::BTransforms::extra_worlds`), so the camera view, the
//! rendered lights and F12 see the moved camera and lamps.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use crate::phiphx::scene::{SceneCamera, SceneLight, SceneModel, SceneMotionFrame};
use crate::v2::state::{BState, CtxKey};

thread_local! {
    /// The frame in scope for the current render pass.
    static FRAME: Cell<Option<f32>> = const { Cell::new(None) };
    /// The edited cameras', lights' and empties' worlds at that frame, by
    /// file name.
    static MOVED: RefCell<Option<Rc<HashMap<String, glam::Mat4>>>> = const { RefCell::new(None) };
}

/// Put the edits' worlds for the frame in scope in scope with it.
pub fn set_moved(moved: Option<HashMap<String, glam::Mat4>>) {
    MOVED.with(|cell| *cell.borrow_mut() = moved.map(Rc::new));
}

/// Where the edits put `name` at `frame`, when that is the frame in scope.
fn moved(name: &str, frame: Option<f32>) -> Option<[[f32; 4]; 4]> {
    if frame != scoped_frame() {
        return None;
    }
    MOVED.with(|cell| cell.borrow().as_ref().and_then(|m| m.get(name)).map(|m| m.to_cols_array_2d()))
}

/// Put `ctx.frame` in scope for this render pass (with the loaded model).
pub fn set_shown_frame(state: &BState) {
    FRAME.with(|cell| cell.set(shown_frame(state)));
}

/// The frame in scope, for readers without state.
pub fn scoped_frame() -> Option<f32> {
    FRAME.with(Cell::get)
}

/// Put a frame in scope directly, for a thread that has no `BState`.
///
/// The scene scope is thread-local and armed by the draw pass, so a worker
/// thread starts with none of it (`scene::source::adopt_scope`).
pub fn set_scoped_frame(frame: Option<f32>) {
    FRAME.with(|cell| cell.set(frame));
}

/// `ctx.frame` as the timeline holds it, when it holds one.
pub fn shown_frame(state: &BState) -> Option<f32> {
    state.get(CtxKey::Frame).and_then(|raw| raw.trim().parse().ok())
}

/// The frame that changes what the viewport draws: the shown frame while
/// the scene moves, `None` for a still scene or no frame.
pub fn moving_frame(model: &SceneModel, frame: Option<f32>) -> Option<f32> {
    frame.filter(|_| model.motion.animated())
}

/// The scene's matrices at `frame`, when it moves.
pub fn posed(model: &SceneModel, frame: Option<f32>) -> Option<SceneMotionFrame<'_>> {
    moving_frame(model, frame).map(|frame| model.motion.at(frame))
}

/// The lights where they stand at `frame`.
pub fn lights_at(model: &SceneModel, frame: Option<f32>) -> Vec<SceneLight> {
    let posed = posed(model, frame);
    model
        .lights
        .iter()
        .enumerate()
        .map(|(i, light)| match moved(&light.name, frame).or_else(|| posed.as_ref().and_then(|p| p.light(i))) {
            Some(matrix_world) => SceneLight { matrix_world, location: [matrix_world[3][0], matrix_world[3][1], matrix_world[3][2]], ..light.clone() },
            None => light.clone(),
        })
        .collect()
}

/// The cameras where they stand at `frame`.
pub fn cameras_at(model: &SceneModel, frame: Option<f32>) -> Vec<SceneCamera> {
    let posed = posed(model, frame);
    model
        .cameras
        .iter()
        .enumerate()
        .map(|(i, camera)| match moved(&camera.name, frame).or_else(|| posed.as_ref().and_then(|p| p.camera(i))) {
            Some(matrix_world) => SceneCamera { matrix_world, location: [matrix_world[3][0], matrix_world[3][1], matrix_world[3][2]], ..camera.clone() },
            None => camera.clone(),
        })
        .collect()
}

/// The active camera where it stands at `frame`.
pub fn active_camera_at(model: &SceneModel, frame: Option<f32>) -> Option<SceneCamera> {
    let cameras = cameras_at(model, frame);
    cameras.iter().find(|c| c.name == model.render.camera).or_else(|| cameras.first()).cloned()
}
