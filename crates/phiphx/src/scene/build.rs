//! A scene built from code rather than read from a file: a game's arena,
//! its players and its ball (`phiplay`, through Phiber's Play host).
//!
//! A model read from a `.blend` keeps several tables in step — the objects,
//! their types, their own matrices, their UV maps, the motion each moves
//! with, the collection that lists them, the rigs and the meshes those
//! deform. Adding an object by hand means adding a row to each, and these
//! are the three ways in, each keeping them in step:
//!
//! - [`SceneModel::add_object`]: a mesh, still or moved by a play;
//! - [`SceneModel::adopt_rig`]: a rigged character taken from another model
//!   — its armature and the meshes it deforms — posed by a play;
//! - [`SceneModel::add_skinned`]: a new mesh deformed by a rig already
//!   here, with its own bone weights — a garment cut from the body, a
//!   hairstyle grown on the head.
//!
//! The adopted and added meshes share what they can with their source by
//! `Arc` (the base mesh, the skin's tail), so a team of eleven adopted from
//! one figure holds one figure's data eleven times by reference.

use super::{SceneCollection, SceneMaterial, SceneModel, SceneObject, SceneSkin};
use crate::blend::BPlayRef;
use crate::blend::ObjectTransformRaw;
use crate::blend::{BFaces, BlendEvalMesh, BlendMotion};
use std::sync::Arc;

/// A mesh to be deformed by a rig: polygons over vertices, a UV per
/// vertex, and each vertex's weights by the rig's vertex-group names.
#[derive(Clone, Debug, Default)]
pub struct BSkinnedMesh {
    pub vertices: Vec<[f32; 3]>,
    pub faces: Vec<Vec<u32>>,
    pub uvs: Vec<[f32; 2]>,
    /// The material slot of each face.
    pub face_materials: Vec<u16>,
    /// The vertex-group names `weights` index.
    pub groups: Vec<String>,
    /// Per vertex: `(group, weight)`.
    pub weights: Vec<Vec<(u32, f32)>>,
    /// Shaded smooth.
    pub smooth: bool,
}

/// What [`SceneModel::adopt_rig`] made.
#[derive(Clone, Debug, PartialEq)]
pub struct BAdopted {
    /// The rig's index in `motion.rigs.rigs`.
    pub rig: usize,
    /// The meshes adopted with it: (the name they had, their new index).
    pub objects: Vec<(String, usize)>,
}

impl SceneModel {
    /// Adds `object` as it stands (its own `matrix_world`, no parent),
    /// moved by `play` when one is given. Its index.
    pub fn add_object(&mut self, object: SceneObject, play: Option<BPlayRef>) -> usize {
        let raw = raw_of(object.matrix_world);
        let motion = BlendMotion { play, ..BlendMotion::still(object.name.clone(), raw) };
        self.push_object(object, motion)
    }

    /// Takes rig `rig` of `from` — its armature and every mesh it deforms
    /// whose name `keep` admits — into this model as `name`, posed by
    /// `play`. The meshes are renamed by replacing the rig's own name
    /// prefix (everything before " Rig") with `name`.
    pub fn adopt_rig(&mut self, from: &SceneModel, rig: usize, name: &str, keep: impl Fn(&str) -> bool, play: BPlayRef) -> BAdopted {
        let source = &from.motion.rigs.rigs[rig];
        let old_rig = &from.motion.motion.list[source.object];
        let prefix = old_rig.name.strip_suffix(" Rig").unwrap_or(&old_rig.name).to_string();
        let rename = |old: &str| old.strip_prefix(&prefix).map_or_else(|| format!("{name} {old}"), |rest| format!("{name}{rest}"));
        // The armature: its motion, now played, and its rig source.
        let rig_motion = self.motion.motion.list.len();
        self.motion.motion.list.push(BlendMotion { name: rename(&old_rig.name), play: Some(play.clone()), parent: None, ..old_rig.clone() });
        let new_rig = self.motion.rigs.rigs.len();
        self.motion.rigs.rigs.push(Arc::new(crate::blend::rig::BlendRigSource {
            object: rig_motion,
            moves: true,
            play: Some(play),
            nla: Vec::new(),
            curves: Vec::new(),
            ..(**source).clone()
        }));
        self.motion.rigs.saved.push(from.motion.rigs.saved.get(rig).cloned().unwrap_or_default());
        // The meshes it deforms.
        let mut objects = Vec::new();
        for skin in from.motion.rigs.skins.iter().filter(|s| s.rig == rig) {
            let old = &from.objects[skin.object];
            if !keep(&old.name) {
                continue;
            }
            let new_name = rename(&old.name);
            let old_motion = &from.motion.motion.list[skin.motion];
            let parent = old_motion.parent.filter(|&p| p == source.object).map(|_| rig_motion);
            let motion = BlendMotion { name: new_name.clone(), parent, play: None, ..old_motion.clone() };
            let index = self.push_object(SceneObject { name: new_name.clone(), ..old.clone() }, motion);
            if let Some(uv) = from.uv_maps.get(skin.object) {
                self.uv_maps[index] = uv.clone();
            }
            if parent.is_some() {
                self.parents.push((new_name, self.motion.motion.list[rig_motion].name.clone()));
            }
            self.motion.rigs.skins.push(Arc::new(SceneSkin {
                object: index,
                motion: self.motion.object[index],
                rig: new_rig,
                source: skin.source.clone(),
                tris: skin.tris.clone(),
            }));
            objects.push((old.name.clone(), index));
        }
        BAdopted { rig: new_rig, objects }
    }

    /// Adds `mesh` deformed by rig `rig` (an index into
    /// `motion.rigs.rigs`), drawn with `materials`. It stands where the
    /// rig's other meshes stand (it moves with the rig's first mesh), and
    /// its own bone weights decide how it follows the bones. Its index.
    ///
    /// `None` when the rig deforms nothing yet to stand with, or when a
    /// face names a vertex the mesh lacks.
    pub fn add_skinned(&mut self, rig: usize, name: &str, mesh: BSkinnedMesh, materials: Vec<SceneMaterial>) -> Option<usize> {
        let n = mesh.vertices.len();
        if mesh.faces.iter().flatten().any(|&v| v as usize >= n) {
            return None;
        }
        let beside = self.motion.rigs.skins.iter().find(|s| s.rig == rig)?.clone();
        let mut faces = BFaces::default();
        for face in &mesh.faces {
            faces.push(face);
        }
        let loop_uvs: Vec<[f32; 2]> = mesh.faces.iter().flatten().map(|&v| mesh.uvs.get(v as usize).copied().unwrap_or_default()).collect();
        let base = BlendEvalMesh {
            vertices: mesh.vertices.clone(),
            faces,
            loop_uvs,
            face_materials: mesh.face_materials.clone(),
            face_smooth: vec![mesh.smooth; mesh.faces.len()],
            dverts: mesh.weights.clone(),
            ..Default::default()
        };
        let mut tris = super::object::STriangles::from_eval(&base);
        let drawn = tris.mesh(mesh.vertices.clone(), &[], &[]);
        let material_indices = std::mem::take(&mut tris.material_indices);
        let object = SceneObject {
            materials,
            material_indices: Arc::new(material_indices),
            uvs: Arc::new(mesh.uvs.clone()),
            ..SceneObject::identity(name, drawn)
        };
        let template = &self.motion.motion.list[beside.motion];
        let motion = BlendMotion { name: name.to_string(), play: None, ..template.clone() };
        let index = self.push_object(object, motion);
        let source = crate::blend::modifier::BlendSkinSource::bare(index, beside.source.armature, base, mesh.groups);
        self.motion.rigs.skins.push(Arc::new(SceneSkin { object: index, motion: self.motion.object[index], rig, source: Arc::new(source), tris }));
        Some(index)
    }

    /// One row in every per-object table.
    fn push_object(&mut self, object: SceneObject, motion: BlendMotion) -> usize {
        let index = self.objects.len();
        self.motion.motion.list.push(motion);
        self.motion.object.push(self.motion.motion.list.len() - 1);
        self.object_types.push("MESH".to_string());
        self.locals.push(object.matrix_world);
        self.uv_maps.push(Default::default());
        match self.collections.first_mut() {
            Some(root) => root.object_indices.push(index),
            None => self.collections.push(SceneCollection {
                name: "Scene Collection".into(),
                children: Vec::new(),
                object_indices: vec![index],
                extra_names: Vec::new(),
                exclude: false,
                hide_viewport: false,
                visibility: Default::default(),
            }),
        }
        self.objects.push(object);
        index
    }
}

/// The location, rotation (as a quaternion) and scale of a world matrix,
/// as an object's own channels.
fn raw_of(m: [[f32; 4]; 4]) -> ObjectTransformRaw {
    let (scale, rotation, location) = glam::Mat4::from_cols_array_2d(&m).to_scale_rotation_translation();
    ObjectTransformRaw::from_parts(location.into(), [rotation.w, rotation.x, rotation.y, rotation.z], scale.into())
}

#[cfg(test)]
mod tests;
