//! The meshes a scene's rigs bend and the rigs' bones, at the frame the
//! viewport shows — posed on a thread of their own ([`BPoser`],
//! `worker.rs`), so the UI never waits on a rig.
//!
//! Blender evaluates the depsgraph on the main thread when the frame
//! changes, and the viewport waits for it. THYMUS's rig and its eleven
//! meshes take 35 ms a frame on two cores (batch 85), which the render pass
//! paid on every frame the playhead moved. Now the render pass asks the
//! poser for the shown frame and draws what it has:
//! - the frame, when it is posed. While playing, the poser works on the
//!   frames after the playhead, so each is ready when the playhead gets
//!   there;
//! - otherwise the frame it drew last (the meshes as read, before the
//!   first), and the window draws again ([`BPoser::waiting`]) until the
//!   frame lands. A playhead the poser cannot keep up with shows the newest
//!   frame it has, as Blender's frame dropping does.
//!
//! F12, and anything else that needs the exact frame, poses in place
//! ([`BPosed::exact`], through `current_scene`).

mod worker;

pub use worker::BPoser;

use super::{BObject3D, OBJECTS, edit, motion, transform};
use crate::phiphx::scene::{SceneMesh, SceneModel};
use crate::v2::state::BState;
use std::collections::HashMap;
use std::sync::Arc;

/// The meshes a scene's rigs deform, by object index, and every rig's
/// bones (`pose_mat`, by rig index), posed at one frame.
#[derive(Clone, Debug, Default)]
pub struct BPosed {
    /// The frame they were posed at; `None` for the meshes as read.
    pub frame: Option<f32>,
    pub meshes: Arc<HashMap<usize, Arc<SceneMesh>>>,
    pub bones: Arc<Vec<Arc<Vec<glam::Mat4>>>>,
}

impl BPosed {
    /// Posed here, at `frame`, when a rig moves the scene.
    pub fn exact(model: Option<&SceneModel>, frame: Option<f32>) -> Self {
        match model.and_then(|m| Self::moving(m, frame).map(|f| (m, f))) {
            Some((model, frame)) => Self::at(model, frame),
            None => Self::default(),
        }
    }

    /// The frame to pose `model` at: the shown one, while a rig moves.
    pub fn moving(model: &SceneModel, frame: Option<f32>) -> Option<f32> {
        motion::moving_frame(model, frame).filter(|_| model.motion.rigs.animated() || model.motion.rigs.moves())
    }

    /// What the viewport draws at the shown frame: the poser's (`BPoser::shown`).
    pub fn shown(s: &BState) -> Self {
        super::source::loaded_shared().map(|m| BPoser::shown(&m, s)).unwrap_or_default()
    }

    /// Rig `rig`'s bones: as posed here, else as the file was saved.
    pub fn bones_of<'a>(&'a self, model: &'a SceneModel, rig: usize) -> Option<&'a [glam::Mat4]> {
        self.bones.get(rig).or_else(|| model.motion.rigs.saved.get(rig)).map(|b| b.as_slice())
    }

    /// Posed at `frame` (phiphx `SceneMotion::posed_at`).
    fn at(model: &SceneModel, frame: f32) -> Self {
        let (meshes, bones) = model.motion.posed_at(frame);
        let meshes = meshes.into_iter().map(|(i, m)| (i, Arc::new(m))).collect();
        Self { frame: Some(frame), meshes: Arc::new(meshes), bones: Arc::new(bones) }
    }

    /// The Scene_Source with these meshes: the Default_Scene (one
    /// [`BObject3D::Primitive`] per entry in [`OBJECTS`]) when no model is
    /// loaded, otherwise one [`BObject3D::Loaded`] per object in `loaded`,
    /// in order — where it stands at the shown frame, a rigged mesh as posed
    /// here — then the session's edits and the Transform panel over them.
    pub fn scene(&self, s: &BState, loaded: Option<&SceneModel>) -> Vec<BObject3D> {
        let objects = match loaded {
            None => OBJECTS.iter().map(|o| BObject3D::Primitive { name: o.name.to_string(), shape: o.shape, x: o.x, y: o.y }).collect(),
            Some(model) => {
                let posed = motion::posed(model, motion::shown_frame(s));
                model
                    .objects
                    .iter()
                    .enumerate()
                    .map(|(i, o)| BObject3D::Loaded {
                        name: o.name.clone(),
                        mesh: self.meshes.get(&i).cloned().unwrap_or_else(|| o.mesh.clone()),
                        location: o.location,
                        scale: o.scale,
                        matrix_world: posed.as_ref().and_then(|p| p.object(i)).unwrap_or(o.matrix_world),
                        materials: o.materials.clone(),
                        material_indices: o.material_indices.clone(),
                        uvs: o.uvs.clone(),
                        color: o.color,
                    })
                    .collect()
            }
        };
        let mut objects = edit::BSceneEdit::now(s).apply(objects);
        if let Some(model) = loaded {
            transform::BTransforms::now(s).apply(s, model, &mut objects);
        }
        objects
    }
}
