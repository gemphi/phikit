//! Meshes their rigs deform, at any frame: each moving rig posed
//! (`blend/rig`), each mesh its Armature modifier deforms skinned by that
//! pose and its later modifiers run again, drawn with the triangles it was
//! read with — so a frame changes where the vertices are, never what the
//! triangles are, and the tracer takes it as a new pose (`Tracer::repose`).

use super::SceneMesh;
use super::object::STriangles;
use crate::blend::modifier::BlendSkinSource;
use crate::blend::modifier::armature::deform;
use crate::blend::BMotionAt;
use crate::blend::rig::{BEmptyScene, BlendRigSource};
use glam::Mat4;
use rayon::prelude::*;
use std::sync::Arc;

/// One deformed mesh.
#[derive(Debug)]
pub struct SceneSkin {
    /// Index into `SceneModel::objects`.
    pub object: usize,
    /// Index into the motion list (the object's blend index).
    pub motion: usize,
    /// Index into [`SceneRigs::rigs`].
    pub rig: usize,
    pub source: Arc<BlendSkinSource>,
    pub tris: STriangles,
}

/// The rigs and the meshes the moving ones deform.
#[derive(Clone, Debug, Default)]
pub struct SceneRigs {
    pub rigs: Vec<Arc<BlendRigSource>>,
    pub skins: Vec<Arc<SceneSkin>>,
    /// Each rig's bones (`pose_mat`) at the saved frame, by rig index —
    /// what the viewport draws until a frame poses them.
    pub saved: Vec<Arc<Vec<Mat4>>>,
}

impl PartialEq for SceneRigs {
    fn eq(&self, other: &Self) -> bool {
        self.rigs.len() == other.rigs.len()
            && self.rigs.iter().zip(&other.rigs).all(|(a, b)| Arc::ptr_eq(a, b))
            && self.skins.len() == other.skins.len()
            && self.skins.iter().zip(&other.skins).all(|(a, b)| Arc::ptr_eq(a, b))
    }
}

impl SceneRigs {
    /// Does a rig move a mesh?
    pub fn animated(&self) -> bool {
        !self.skins.is_empty()
    }

    /// Does a rig pose with the frame (an object may follow its bones)?
    pub fn moves(&self) -> bool {
        self.rigs.iter().any(|r| r.moves)
    }

    /// Each rig's bones (`pose_mat`) at a frame solved with every rig
    /// posed, by rig index.
    pub fn bones_at(&self, at: &BMotionAt) -> Vec<Arc<Vec<Mat4>>> {
        self.rigs.iter().map(|r| Arc::new(at.poses.get(&r.object).map(|p| p.pose.clone()).unwrap_or_default())).collect()
    }

    /// Each deformed mesh at a solved frame, by object index.
    pub fn meshes_at(&self, at: &BMotionAt) -> Vec<(usize, SceneMesh)> {
        let skinned = self.skinned_at(at);
        self.skins
            .par_iter()
            .zip(skinned)
            .map(|(s, (object, posed))| {
                let mesh = s.source.evaluate(posed, Self::world(&at.world, s.motion));
                (object, s.tris.mesh(mesh.vertices, &[]))
            })
            .collect()
    }

    /// Each deformed mesh as its Armature modifier leaves it at a solved
    /// frame — before the modifiers after it — by object index.
    pub fn skinned_at(&self, at: &BMotionAt) -> Vec<(usize, Vec<[f32; 3]>)> {
        let bones: Vec<_> = self
            .rigs
            .par_iter()
            .map(|r| match at.poses.get(&r.object) {
                Some(pose) => r.deform_bones(pose),
                None => r.deform_bones(&r.pose_at(at.frame, Self::world(&at.world, r.object), &BEmptyScene)),
            })
            .collect();
        self.skins
            .par_iter()
            .map(|s| {
                let base = &s.source.base;
                let (arm, ob) = (Self::world(&at.world, self.rigs[s.rig].object), Self::world(&at.world, s.motion));
                let posed = deform(&base.vertices, &base.dverts, &s.source.groups, &bones[s.rig], arm.inverse() * ob).unwrap_or_else(|| base.vertices.clone());
                (s.object, posed)
            })
            .collect()
    }

    fn world(worlds: &[[[f32; 4]; 4]], i: usize) -> Mat4 {
        worlds.get(i).map_or(Mat4::IDENTITY, Mat4::from_cols_array_2d)
    }
}
