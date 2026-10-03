//! What the Outliner's drag and drop does to a file: an object dropped on a
//! collection moves there (`outliner_collection_drop`, Move — out of every
//! collection that held it, into that one; its base in the view layer stays
//! as it was), and an object dropped on another object is parented to it
//! (`parent_drop_exec`, Object parenting) keeping where it stands: its
//! `parentinv` becomes the inverse of the new parent's world matrix times
//! what the old parent contributed, so `parent · parentinv · local` is the
//! world matrix it had.

use crate::blend::file::types::*;
use glam::Mat4;

/// `Object.partype` for object parenting (`PAROBJECT`).
pub const PAROBJECT: i64 = 0;

impl BlendFile {
    /// Move the object at `object` into the collection at `path`: false
    /// when the collection is not in the file.
    pub fn move_object(&mut self, object: usize, path: &str) -> bool {
        let (Some(scene), Some(dna)) = (self.current_scene_index(), self.dna.clone()) else { return false };
        let Some(view_layer) = self.view_layer_index(scene) else { return false };
        if !self.collections_at(scene, view_layer).iter().any(|c| c.path == path) {
            return false;
        }
        let address = self.blocks[object].old_address;
        let ob_off = self.off("CollectionObject", "*ob", &dna);
        let first_off = self.field_span("Collection", "gobject.*first", &dna).map(|(o, _)| o);
        let mut gone = Vec::new();
        for c in self.collections_at(scene, view_layer) {
            let first = self.ptr(&self.blocks[c.collection], first_off);
            let nodes: Vec<usize> = self.list(first, self.off("CollectionObject", "*next", &dna)).into_iter().filter_map(|b| self.index_of(b)).collect();
            let nodes: Vec<usize> = nodes.into_iter().filter(|&n| self.ptr(&self.blocks[n], ob_off) == address).collect();
            for node in nodes {
                self.list_unlink(c.collection, "Collection", "gobject", "CollectionObject", node);
                gone.push(node);
            }
        }
        self.remove_blocks(gone);
        // Indices moved with the removal: look the target up again.
        let (Some(scene), Some(view_layer)) = (self.current_scene_index(), self.current_scene_index().and_then(|s| self.view_layer_index(s))) else { return false };
        let Some(target) = self.collections_at(scene, view_layer).into_iter().find(|c| c.path == path) else { return false };
        self.link_to_collection(scene, target.collection, address).is_some()
    }

    /// Put the object at `object` into the collection at `path` as well —
    /// the Outliner's Ctrl-drag, Link (`outliner_collection_drop`,
    /// `COLLECTION_DROP_LINK`): it stays in the collections that hold it.
    pub fn link_object_to(&mut self, object: usize, path: &str) -> bool {
        let Some(scene) = self.current_scene_index() else { return false };
        let Some(view_layer) = self.view_layer_index(scene) else { return false };
        let Some(target) = self.collections_at(scene, view_layer).into_iter().find(|c| c.path == path) else { return false };
        let address = self.blocks[object].old_address;
        if self.collections_holding(scene, view_layer, address).iter().any(|p| p == path) {
            return true;
        }
        self.link_to_collection(scene, target.collection, address).is_some()
    }

    /// Nest the collection named `child` in the one named `parent`
    /// (`outliner_collection_drop` onto a collection, Move): its
    /// `CollectionChild` node leaves the parent that held it and a new one
    /// joins `parent`'s `children`, in `parent`'s own data. Collection names
    /// are unique in a file, so a name finds it wherever it stands now.
    pub fn nest_collection(&mut self, child: &str, parent: &str) -> bool {
        let (Some(scene), Some(dna)) = (self.current_scene_index(), self.dna.clone()) else { return false };
        let Some(view_layer) = self.view_layer_index(scene) else { return false };
        let named = |c: &super::layer::BlendCollectionAt, name: &str| c.path.rsplit('/').next() == Some(name);
        let collections = self.collections_at(scene, view_layer);
        let (Some(moving), true) = (collections.iter().find(|c| named(c, child)), collections.iter().any(|c| named(c, parent))) else { return false };
        let address = self.blocks[moving.collection].old_address;
        let collection_off = self.off("CollectionChild", "*collection", &dna);
        let first_off = self.field_span("Collection", "children.*first", &dna).map(|(o, _)| o);
        let mut gone = Vec::new();
        for c in &collections {
            let first = self.ptr(&self.blocks[c.collection], first_off);
            let nodes: Vec<usize> = self.list(first, self.off("CollectionChild", "*next", &dna)).into_iter().filter_map(|b| self.index_of(b)).collect();
            let nodes: Vec<usize> = nodes.into_iter().filter(|&n| self.ptr(&self.blocks[n], collection_off) == address).collect();
            for node in nodes {
                self.list_unlink(c.collection, "Collection", "children", "CollectionChild", node);
                gone.push(node);
            }
        }
        self.remove_blocks(gone);
        // Indices moved with the removal: look the new parent up again.
        let (Some(scene), Some(view_layer)) = (self.current_scene_index(), self.current_scene_index().and_then(|s| self.view_layer_index(s))) else { return false };
        let Some(target) = self.collections_at(scene, view_layer).into_iter().find(|c| named(c, parent)) else { return false };
        // The master collection's nodes are the scene's data.
        let owner = if self.blocks[target.collection].code_str == "GR" { target.collection } else { scene };
        let Some((node, node_address)) = self.push_block("CollectionChild", owner) else { return false };
        let target = if node <= target.collection { target.collection + 1 } else { target.collection };
        self.set_ptr(node, "CollectionChild", "*collection", address);
        self.list_append(target, "Collection", "children", "CollectionChild", node, node_address);
        true
    }

    /// Parent `child` to `parent`, keeping `child` where it stands:
    /// `child_world` is its world matrix now and `parent_world` the new
    /// parent's, both Blender's `float[4][4]` (columns).
    pub fn parent_object(&mut self, child: usize, parent: usize, child_world: [[f32; 4]; 4], child_local: [[f32; 4]; 4], parent_world: [[f32; 4]; 4]) -> bool {
        let parent_address = self.blocks[parent].old_address;
        // world = P · inv · local  ⇒  inv = P⁻¹ · world · local⁻¹.
        let (world, local, p) = (Mat4::from_cols_array_2d(&child_world), Mat4::from_cols_array_2d(&child_local), Mat4::from_cols_array_2d(&parent_world));
        let inv = (p.inverse() * world * local.inverse()).to_cols_array_2d();
        let mut done = self.set_ptr(child, "Object", "*parent", parent_address);
        done &= self.set_int(child, "Object", "partype", PAROBJECT);
        for (c, col) in inv.iter().enumerate() {
            for (r, value) in col.iter().enumerate() {
                self.set_f32(child, "Object", &format!("parentinv[{c}][{r}]"), *value);
            }
        }
        done
    }
}
