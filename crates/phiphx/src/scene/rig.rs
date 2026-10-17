//! Meshes their rigs deform, at any frame: each moving rig posed
//! (`blend/rig`), each mesh its Armature modifier deforms skinned by that
//! pose and its later modifiers run again, drawn with the triangles it was
//! read with — so a frame changes where the vertices are, never what the
//! triangles are, and the tracer takes it as a new pose (`Tracer::repose`).

use super::SceneMesh;
use super::object::STriangles;
use crate::blend::modifier::BlendSkinSource;
use crate::blend::modifier::driven::BlendDrivenSource;
use crate::blend::modifier::armature::deform;
use crate::blend::BMotionAt;
use crate::blend::rig::{BEmptyScene, BlendRigSource};
use glam::Mat4;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

/// One frame's driven modifier settings, by the object's saved address.
pub type DrivenAt = HashMap<u64, Vec<(crate::blend::BModWrite, crate::blend::BModValue)>>;

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

/// A non-rigged mesh with modifier drivers, re-evaluated per frame.
/// Unlike `SceneSkin`, topology can change (a driven Subdivision level),
/// so there are no saved triangles — each frame re-triangulates.
#[derive(Debug)]
pub struct SceneDriven {
    /// Index into `SceneModel::objects`.
    pub object: usize,
    /// Index into the motion list (the object's blend index).
    pub motion: usize,
    pub source: Arc<BlendDrivenSource>,
}

/// The rigs and the meshes the moving ones deform.
#[derive(Clone, Debug, Default)]
pub struct SceneRigs {
    pub rigs: Vec<Arc<BlendRigSource>>,
    pub skins: Vec<Arc<SceneSkin>>,
    /// Modifier drivers, by the OBJECT they sit on. A frame's values land on
    /// a clone of that object's tail before it is evaluated
    /// (`BlendSkinSource::evaluate_driven`), which is Blender's own shape:
    /// the depsgraph drives a copy, never the file's stack.
    ///
    /// Empty for almost every file, and an empty set costs one lookup and no
    /// clone.
    pub drivers: Vec<crate::blend::BModifierDrivers>,
    /// Each rig's bones (`pose_mat`) at the saved frame, by rig index —
    /// what the viewport draws until a frame poses them.
    pub saved: Vec<Arc<Vec<Mat4>>>,
    /// Non-rigged meshes with modifier drivers — re-evaluated per frame
    /// when a driver changes a modifier's setting.
    pub driven: Vec<Arc<SceneDriven>>,
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
    /// Does a rig move a mesh, or does a driver move a non-rigged one?
    pub fn animated(&self) -> bool {
        !self.skins.is_empty() || !self.driven.is_empty()
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

    /// Each deformed mesh at a solved frame, by object index — rigged
    /// meshes posed by their Armature, and non-rigged driven meshes
    /// re-evaluated with this frame's modifier-driver settings.
    ///
    /// `driven_at` is this frame's modifier-driver values by object
    /// address — `SceneMotion::meshes_at` works them out, because the
    /// driver scene comes from the motion list and this type does not
    /// hold one.
    pub fn meshes_at(&self, at: &BMotionAt, driven_at: &DrivenAt) -> Vec<(usize, SceneMesh)> {
        let skinned = self.skinned_at(at);
        let none: Vec<(crate::blend::BModWrite, crate::blend::BModValue)> = Vec::new();
        let mut out: Vec<(usize, SceneMesh)> = self.skins
            .par_iter()
            .zip(skinned)
            .map(|(s, (object, posed))| {
                let values = driven_at.get(&s.source.object_address).unwrap_or(&none);
                let mesh = s.source.evaluate_driven(posed, Self::world(&at.world, s.motion), values);
                (object, s.tris.mesh(mesh.vertices, &[], &[]))
            })
            .collect();
        // Non-rigged driven meshes: full re-evaluation + re-triangulation.
        let driven_meshes: Vec<(usize, SceneMesh)> = self.driven
            .par_iter()
            .filter_map(|d| {
                let values = driven_at.get(&d.source.object_address)?;
                let eval = d.source.evaluate_at(Self::world(&at.world, d.motion), values);
                let tris = STriangles::from_eval(&eval);
                Some((d.object, tris.mesh(eval.vertices, &[], &[])))
            })
            .collect();
        out.extend(driven_meshes);
        out
    }

    /// This frame's driven modifier settings, by object address — empty when
    /// no object in the scene carries a modifier driver.
    pub fn driven_at(&self, motion: &crate::blend::BlendMotionList, frame: f32) -> DrivenAt {
        if self.drivers.iter().all(|set| set.is_empty()) {
            return DrivenAt::new();
        }
        let state = motion.driver_state_at(frame);
        let scene = state.scene();
        self.drivers
            .iter()
            .filter_map(|set| {
                let values = set.values(&scene, frame);
                (!values.is_empty()).then(|| (set.address, values))
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
