//! Preparing a file's meshes as it loads, on a thread of their own: the pick
//! tree of each mesh large enough to want one (`pick/tree.rs`) and the
//! extreme points of each the outline would otherwise project whole
//! (`mesh/extreme.rs`).
//!
//! Both were made on first use, on the UI thread: a second's pause on the
//! first click over a two-million-triangle mesh in a debug build, and
//! another on the first orbit with it selected. Made here, the first click
//! and the first orbit cost what the next ones do. The UI thread still makes
//! what it asks for before the thread gets there, so nothing waits on it;
//! the thread's copy is then dropped (`BMeshKeep::of` keeps the first).
//!
//! Largest first: that is the one the pause was for.

use crate::phiphx::scene::{SceneMesh, SceneModel};
use crate::v2::scene::pick::tree::{self, BPickTree};
use crate::v2::view::draw::canvas::view3d::mesh::extreme;
use std::sync::Arc;

/// The meshes a loaded file wants prepared.
pub struct BWarm {
    meshes: Vec<Arc<SceneMesh>>,
}

impl BWarm {
    /// Every distinct mesh of `model` with a tree or points worth making,
    /// largest first.
    pub fn of(model: &SceneModel) -> Self {
        let mut meshes: Vec<Arc<SceneMesh>> = Vec::new();
        for object in &model.objects {
            let m = &object.mesh;
            if (m.indices.len() / 3 >= tree::WHOLE || m.vertices.len() > extreme::WHOLE) && !meshes.iter().any(|k| Arc::ptr_eq(k, m)) {
                meshes.push(Arc::clone(m));
            }
        }
        meshes.sort_by_key(|m| std::cmp::Reverse(m.indices.len()));
        Self { meshes }
    }

    /// How many meshes it will prepare.
    pub fn len(&self) -> usize {
        self.meshes.len()
    }

    /// Prepare them on a thread of their own; the handle joins it.
    pub fn start(self) -> Option<std::thread::JoinHandle<()>> {
        if self.meshes.is_empty() {
            return None;
        }
        std::thread::Builder::new().name("phiber-warm".into()).spawn(move || self.run()).ok()
    }

    /// Prepare them here.
    pub fn run(&self) {
        for m in &self.meshes {
            if m.indices.len() / 3 >= tree::WHOLE && !BPickTree::ready(m) {
                BPickTree::of(m);
            }
            if m.vertices.len() > extreme::WHOLE && !extreme::ready(m) {
                extreme::of(m);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phiphx::scene::{SceneNormals, SceneObject};

    fn sheet(n: usize) -> SceneMesh {
        let mut v = Vec::new();
        for y in 0..=n {
            for x in 0..=n {
                v.push([x as f32, y as f32, 0.0]);
            }
        }
        let (mut i, row) = (Vec::new(), (n + 1) as u32);
        for y in 0..n as u32 {
            for x in 0..n as u32 {
                let p = y * row + x;
                i.extend([p, p + 1, p + row, p + 1, p + row + 1, p + row]);
            }
        }
        SceneMesh { vertices: v, indices: i, normals: SceneNormals::PerFace(Vec::new()), corners: Default::default() }
    }

    #[test]
    fn a_loaded_file_s_large_meshes_are_prepared_before_the_first_click() {
        let mut model = SceneModel::default();
        let big = Arc::new(sheet(40));
        model.objects.push(SceneObject::identity("A", sheet(2)));
        for k in 0..3 {
            let mut o = SceneObject::identity(format!("B{k}"), sheet(2));
            o.mesh = Arc::clone(&big);
            model.objects.push(o);
        }
        let warm = BWarm::of(&model);
        assert_eq!(warm.len(), 1, "the small mesh is walked whole; the shared one is listed once");
        assert!(!BPickTree::ready(&big) && !extreme::ready(&big));
        warm.start().expect("a thread").join().unwrap();
        assert!(BPickTree::ready(&big) && extreme::ready(&big), "both made as the file loaded");
    }
}
