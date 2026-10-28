//! The vertex-parented children of a progressive open: the parent links
//! kept while the modifiers run, and where each moved object goes once
//! they have (`progressive.rs`). Blender places such a child from its
//! parent's evaluated mesh (`give_parvert`, `object/vertex_parent.rs`), so
//! it cannot stand right before that mesh exists.

use super::SceneModel;
use crate::blend::{BlendSceneObject, DeferredRaw, is_vertex_parent};
use std::collections::HashSet;

/// Every blend object's parent links (transforms only, no geometry; the
/// saved vertices of a vertex parent with no modifier job), for
/// `vertex_children_worlds` once the evaluated meshes land. Empty when no
/// object is vertex-parented, so the common file pays nothing.
pub(super) fn parent_links(objects: &[BlendSceneObject], raw: Option<&DeferredRaw>) -> Vec<BlendSceneObject> {
    if !objects.iter().any(|o| is_vertex_parent(o.partype)) {
        return Vec::new();
    }
    let jobs: HashSet<usize> = raw.map(|r| r.jobs.iter().map(|j| j.index).collect()).unwrap_or_default();
    let parents: HashSet<u64> = objects.iter().filter(|o| is_vertex_parent(o.partype)).map(|o| o.parent_address).collect();
    objects
        .iter()
        .enumerate()
        .map(|(j, o)| BlendSceneObject {
            block_address: o.block_address,
            parent_address: o.parent_address,
            parentinv: o.parentinv,
            partype: o.partype,
            parent_verts: o.parent_verts,
            matrix_local: o.matrix_local,
            matrix_world: o.matrix_world,
            vertices: match parents.contains(&o.block_address) && !jobs.contains(&j) {
                true => o.vertices.clone(),
                false => Vec::new(),
            },
            ..Default::default()
        })
        .collect()
}

/// The nearest vertex parent above blend object `j` — the parent of the
/// first vertex-parented object on the way up from `j` (itself included).
pub(super) fn vertex_parent_of(links: &[BlendSceneObject], j: usize) -> Option<usize> {
    let index_of = |addr: u64| links.iter().position(|o| o.block_address == addr && addr != 0);
    let (mut at, mut hops) = (j, 0);
    while hops < links.len() {
        let o = links.get(at)?;
        let parent = index_of(o.parent_address).filter(|&p| p != at)?;
        if is_vertex_parent(o.partype) {
            return Some(parent);
        }
        at = parent;
        hops += 1;
    }
    None
}

impl SceneModel {
    /// Put blend object `j` — whichever of object, camera, light or empty it
    /// became — at `world` (the motion's index maps say which).
    pub(super) fn place(&mut self, j: usize, world: [[f32; 4]; 4]) {
        let m = &self.motion;
        if let Some(o) = m.object.iter().position(|&b| b == j).and_then(|k| self.objects.get_mut(k)) {
            o.matrix_world = world;
        }
        if let Some(c) = m.camera.iter().position(|&b| b == j).and_then(|k| self.cameras.get_mut(k)) {
            c.matrix_world = world;
        }
        if let Some(l) = m.light.iter().position(|&b| b == j).and_then(|k| self.lights.get_mut(k)) {
            l.matrix_world = world;
        }
        if let Some(e) = m.empty.iter().position(|&b| b == j).and_then(|k| self.empties.get_mut(k)) {
            e.matrix_world = world;
        }
    }

}
