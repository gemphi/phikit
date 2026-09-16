//! glTF scene wrapper with primary/active-subject selection.
//!
//! Mirrors the blend importer's `BlendScene`: a set of extracted meshes plus a
//! recorded active/selected subject, resolved through the shared
//! [`select_primary_subject`](crate::importers::select_primary_subject) so glTF,
//! USD, and blend all pick a scene's primary subject identically.

use crate::phx::Result;
use crate::phx::{Mesh, SubjectSelector};

use super::full::import_gltf_meshes;

/// A parsed glTF scene: the extracted per-node meshes plus the same
/// primary/active-subject selection semantics used by the blend importer.
#[derive(Debug, Clone, Default)]
pub struct GltfScene {
    /// Meshes extracted from the glTF nodes, in scene order.
    pub meshes: Vec<Mesh>,
    /// Index into [`Self::meshes`] of the recorded active subject (e.g. the
    /// default scene's primary node), if any. Invariant: when `Some(i)`,
    /// `i < meshes.len()`.
    pub active_object: Option<usize>,
    /// Indices into [`Self::meshes`] of selected subjects. Each entry is
    /// `< meshes.len()`.
    pub selected: Vec<usize>,
}

impl GltfScene {
    /// Wrap already-extracted meshes with no recorded active subject.
    pub fn from_meshes(meshes: Vec<Mesh>) -> Self {
        Self {
            meshes,
            active_object: None,
            selected: Vec::new(),
        }
    }

    /// Record the active subject by name, setting `active_object` to the first
    /// mesh whose name matches. Leaves it unchanged when no mesh matches,
    /// preserving the `i < len` invariant.
    pub fn with_active_named(mut self, name: &str) -> Self {
        if let Some(i) = self.meshes.iter().position(|m| m.name == name) {
            self.active_object = Some(i);
        }
        self
    }

    /// The subject to frame/inspect by default.
    ///
    /// Resolution order matches the blend importer: the active object when
    /// recorded and in range, otherwise the largest mesh by vertex count with a
    /// deterministic name-based tie-break, otherwise `None` for empty scenes.
    pub fn primary_object(&self) -> Option<&Mesh> {
        SubjectSelector::select_primary_subject(
            &self.meshes,
            self.active_object,
            |m| m.positions.len(),
            |m| m.name.as_str(),
        )
        .map(|i| &self.meshes[i])
    }
}

impl GltfScene {
    /// Import a glTF/GLB file as a [`GltfScene`], preserving the per-node meshes so
    /// the primary/active subject can be resolved consistently with the blend and
    /// USD importers.
    pub fn import(path: &str) -> Result<GltfScene> {
        let meshes = import_gltf_meshes(path)?;
        Ok(GltfScene::from_meshes(meshes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mesh_named(name: &str, verts: usize) -> Mesh {
        let mut m = Mesh::new();
        m.name = name.to_string();
        m.positions = vec![[0.0, 0.0, 0.0]; verts];
        m
    }

    #[test]
    fn empty_scene_has_no_primary() {
        let scene = GltfScene::from_meshes(vec![]);
        assert!(scene.primary_object().is_none());
    }

    #[test]
    fn falls_back_to_largest_mesh() {
        let scene = GltfScene::from_meshes(vec![mesh_named("small", 3), mesh_named("big", 10)]);
        assert_eq!(scene.primary_object().unwrap().name, "big");
    }

    #[test]
    fn active_takes_precedence() {
        let mut scene = GltfScene::from_meshes(vec![mesh_named("small", 3), mesh_named("big", 10)]);
        scene.active_object = Some(0);
        assert_eq!(scene.primary_object().unwrap().name, "small");
    }

    #[test]
    fn out_of_range_active_falls_back() {
        let mut scene = GltfScene::from_meshes(vec![mesh_named("only", 5)]);
        scene.active_object = Some(9);
        assert_eq!(scene.primary_object().unwrap().name, "only");
    }

    #[test]
    fn tie_break_by_name_is_order_independent() {
        let a = GltfScene::from_meshes(vec![mesh_named("beta", 4), mesh_named("alpha", 4)]);
        let b = GltfScene::from_meshes(vec![mesh_named("alpha", 4), mesh_named("beta", 4)]);
        assert_eq!(a.primary_object().unwrap().name, "alpha");
        assert_eq!(b.primary_object().unwrap().name, "alpha");
    }

    #[test]
    fn with_active_named_sets_index() {
        let scene = GltfScene::from_meshes(vec![mesh_named("small", 3), mesh_named("hero", 5)])
            .with_active_named("hero");
        assert_eq!(scene.active_object, Some(1));
        assert_eq!(scene.primary_object().unwrap().name, "hero");
    }
}
