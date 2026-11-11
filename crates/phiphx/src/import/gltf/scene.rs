//! glTF scene wrapper with primary/active-subject selection.
//!
//! Mirrors the blend importer's `BlendScene`: a set of extracted meshes plus a
//! recorded active/selected subject, resolved through the shared
//! [`select_primary_subject`](crate::import::select_primary_subject) so glTF,
//! USD, and blend all pick a scene's primary subject identically.

use crate::Result;
use crate::{Mesh, SubjectSelector};

use crate::import::gltf::full::import_gltf_meshes;

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
#[path = "test/scene.rs"]
mod scene_test;
