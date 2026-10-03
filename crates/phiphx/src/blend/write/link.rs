//! Putting an object into the scene, and taking one out.
//!
//! `BKE_collection_object_add`: a `CollectionObject { *next, *prev, *ob }`
//! appended to the collection's `gobject` list; `BKE_view_layer_base_add`
//! (what `BKE_layer_collection_sync` would do on load): a `Base { *next,
//! *prev, *object, flag }` appended to the view layer's `object_bases`.
//! Removing an object is the reverse — its links taken out of every list,
//! then its `OB` block and the data blocks that follow it dropped; the
//! mesh goes too when no other object uses it. Blender's reader turns a
//! pointer to a missing ID into null, so anything else that pointed at the
//! object (a parent, a constraint target) simply loses it, as
//! `BKE_id_delete` would leave it.

use crate::blend::file::types::*;

impl BlendFile {
    /// The next `*next` / last of a `ListBase` at `field` of block `owner`.
    fn list_last(&self, owner: usize, struct_name: &str, field: &str, dna: &DnaType) -> u64 {
        self.ptr(&self.blocks[owner], self.field_span(struct_name, &format!("{field}.*last"), dna).map(|(o, _)| o))
    }

    /// Append the block at `address` (a list node of `node_struct`) to the
    /// `ListBase` `field` of block `owner`.
    pub(crate) fn list_append(&mut self, owner: usize, struct_name: &str, field: &str, node_struct: &str, node: usize, address: u64) {
        let Some(dna) = self.dna.clone() else { return };
        let last = self.list_last(owner, struct_name, field, &dna);
        self.set_ptr(node, node_struct, "*prev", last);
        self.set_ptr(node, node_struct, "*next", 0);
        if last == 0 {
            self.set_ptr(owner, struct_name, &format!("{field}.*first"), address);
        } else if let Some(last_index) = self.index_at(last) {
            self.set_ptr(last_index, node_struct, "*next", address);
        }
        self.set_ptr(owner, struct_name, &format!("{field}.*last"), address);
    }

    /// Take the node at `node` out of the `ListBase` `field` of `owner`.
    pub(crate) fn list_unlink(&mut self, owner: usize, struct_name: &str, field: &str, node_struct: &str, node: usize) {
        let Some(dna) = self.dna.clone() else { return };
        let block = &self.blocks[node];
        let prev = self.ptr(block, self.off(node_struct, "*prev", &dna));
        let next = self.ptr(block, self.off(node_struct, "*next", &dna));
        let me = block.old_address;
        match self.index_at(prev) {
            Some(p) if prev != 0 => {
                self.set_ptr(p, node_struct, "*next", next);
            }
            _ => {
                self.set_ptr(owner, struct_name, &format!("{field}.*first"), next);
            }
        }
        match self.index_at(next) {
            Some(n) if next != 0 => {
                self.set_ptr(n, node_struct, "*prev", prev);
            }
            _ => {
                let _ = me;
                self.set_ptr(owner, struct_name, &format!("{field}.*last"), prev);
            }
        }
    }

    /// Put the object at `object_address` into the collection at
    /// `collection` (a `GR` block, or the master collection's data block)
    /// and give it a `Base` in the view layer with `base_flag`.
    pub fn link_object(&mut self, scene: usize, view_layer: usize, collection: usize, object_address: u64, base_flag: i64) -> Option<()> {
        let node = self.link_to_collection(scene, collection, object_address)?;
        let (scene, view_layer) = (if node <= scene { scene + 1 } else { scene }, if node <= view_layer { view_layer + 1 } else { view_layer });
        let (base, base_address) = self.push_block("Base", scene)?;
        let view_layer = if base <= view_layer { view_layer + 1 } else { view_layer };
        self.set_ptr(base, "Base", "*object", object_address);
        self.set_int(base, "Base", "flag", base_flag);
        self.list_append(view_layer, "ViewLayer", "object_bases", "Base", base, base_address);
        Some(())
    }

    /// Put the object at `object_address` into the collection at
    /// `collection` — its `CollectionObject` node only, no base. The index
    /// the node was inserted at (indices after it moved up by one).
    pub(crate) fn link_to_collection(&mut self, scene: usize, collection: usize, object_address: u64) -> Option<usize> {
        // The collection's link node sits in the collection's own data —
        // the master collection is the scene's data, so its nodes follow
        // the scene block.
        let owner_id = if self.blocks[collection].code_str == "GR" { collection } else { scene };
        let (node, node_address) = self.push_block("CollectionObject", owner_id)?;
        let collection = if node <= collection { collection + 1 } else { collection };
        self.set_ptr(node, "CollectionObject", "*ob", object_address);
        self.list_append(collection, "Collection", "gobject", "CollectionObject", node, node_address);
        Some(node)
    }

    /// The data blocks that follow the ID block at `id`, up to the next ID.
    fn data_after(&self, id: usize) -> Vec<usize> {
        (id + 1..self.blocks.len()).take_while(|&i| self.blocks[i].code_str == "DATA").collect()
    }

    /// Remove the object at `object` from the scene: its collection links,
    /// its base, the block and its data, and its mesh when nothing else
    /// uses it. Returns how many blocks went.
    /// The paths of the collections that hold the object at `address`
    /// directly, from the scene collection down.
    pub fn collections_holding(&self, scene: usize, view_layer: usize, address: u64) -> Vec<String> {
        let Some(dna) = self.dna.as_ref() else { return Vec::new() };
        let ob_off = self.off("CollectionObject", "*ob", dna);
        let first_off = self.field_span("Collection", "gobject.*first", dna).map(|(o, _)| o);
        self.collections_at(scene, view_layer)
            .into_iter()
            .filter(|c| {
                let first = self.ptr(&self.blocks[c.collection], first_off);
                self.list(first, self.off("CollectionObject", "*next", dna)).into_iter().any(|node| self.ptr(node, ob_off) == address)
            })
            .map(|c| c.path)
            .collect()
    }

    pub fn remove_object(&mut self, scene: usize, view_layer: usize, object: usize) -> usize {
        let Some(dna) = self.dna.clone() else { return 0 };
        let address = self.blocks[object].old_address;
        let mesh = self.ptr(&self.blocks[object], self.off("Object", "*data", &dna));
        // Collection links: every CollectionObject whose `ob` is this object.
        let collections = self.view_layer_index(scene).map(|vl| self.collections_at(scene, vl)).unwrap_or_default();
        let ob_off = self.off("CollectionObject", "*ob", &dna);
        let mut gone: Vec<usize> = Vec::new();
        for c in &collections {
            let first = self.ptr(&self.blocks[c.collection], self.field_span("Collection", "gobject.*first", &dna).map(|(o, _)| o));
            let nodes: Vec<usize> = self.list(first, self.off("CollectionObject", "*next", &dna)).into_iter().filter_map(|b| self.index_of(b)).collect();
            for node in nodes {
                if self.ptr(&self.blocks[node], ob_off) == address {
                    self.list_unlink(c.collection, "Collection", "gobject", "CollectionObject", node);
                    gone.push(node);
                }
            }
        }
        for base in self.bases(view_layer) {
            if base.object_address == address {
                self.list_unlink(view_layer, "ViewLayer", "object_bases", "Base", base.base);
                gone.push(base.base);
            }
        }
        // The active object, if it was this one.
        if self.ptr(&self.blocks[view_layer], self.off("ViewLayer", "*basact", &dna)) != 0 {
            let basact = self.ptr(&self.blocks[view_layer], self.off("ViewLayer", "*basact", &dna));
            if gone.iter().any(|g| self.blocks[*g].old_address == basact) {
                self.set_ptr(view_layer, "ViewLayer", "*basact", 0);
            }
        }
        gone.push(object);
        gone.extend(self.data_after(object));
        // The mesh, when this was its only user.
        if let Some(me) = self.index_at(mesh).filter(|i| self.blocks[*i].code_str == "ME") {
            let data_off = self.off("Object", "*data", &dna);
            let used = self.blocks.iter().enumerate().any(|(i, b)| i != object && b.code_str == "OB" && self.ptr(b, data_off) == mesh);
            if !used {
                gone.push(me);
                gone.extend(self.data_after(me));
            }
        }
        let count = gone.len();
        self.remove_blocks(gone);
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blend::write::mesh::BlendNewMesh;

    fn triangle(name: &str) -> BlendNewMesh {
        BlendNewMesh { name: name.into(), vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]], faces: vec![vec![0, 1, 2]] }
    }

    #[test]
    fn a_linked_object_is_in_the_scene_and_an_unlinked_one_is_gone_with_its_mesh() {
        for name in ["nla_cube.blend", "THYMUS-BOLDROID-WAVE-003.blend"] {
            let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
            let mut file = BlendFile::parse_bytes(&std::fs::read(path).expect("fixture")).expect("parse");
            let before = file.extract_scene().expect("scene");
            let scene = file.current_scene_index().unwrap();
            let view_layer = file.view_layer_index(scene).unwrap();
            let master = file.collections_at(scene, view_layer)[0].collection;
            let (_, mesh_address) = file.add_mesh(file.mesh_template().unwrap(), &triangle("Tri")).unwrap();
            let (_, object_address) = file.add_object(file.mesh_object_template().unwrap(), "Tri", mesh_address, [0.0; 3], [1.0; 3]).unwrap();
            // Indices moved: find them again.
            let scene = file.current_scene_index().unwrap();
            let view_layer = file.view_layer_index(scene).unwrap();
            let master = file.collections_at(scene, view_layer).iter().find(|c| c.collection_address == file.blocks[master.min(file.blocks.len() - 1)].old_address || c.path == "Scene Collection").map(|c| c.collection).unwrap();
            file.link_object(scene, view_layer, master, object_address, 1).expect("linked");
            let after = BlendFile::parse_bytes(&file.to_bytes()).unwrap().extract_scene().unwrap();
            assert_eq!(after.objects.len(), before.objects.len() + 1, "{name}");
            let tri = after.objects.iter().position(|o| o.name == "Tri").unwrap_or_else(|| panic!("{name}: Tri is in the scene"));
            assert_eq!(after.objects[tri].vertices.len(), 3);
            assert!(after.collections[0].object_indices.contains(&tri) || after.collections[0].object_names.contains(&"Tri".to_string()), "{name}: in the scene collection");
            assert!(after.selected.contains(&tri), "{name}: its base is selected");

            // Now remove the first original object: gone, with its mesh.
            let victim = before.objects[0].name.clone();
            let mut file = BlendFile::parse_bytes(&file.to_bytes()).unwrap();
            let scene = file.current_scene_index().unwrap();
            let view_layer = file.view_layer_index(scene).unwrap();
            let ob = file.find_id("OB", &victim).unwrap();
            let blocks = file.blocks.len();
            let removed = file.remove_object(scene, view_layer, ob);
            assert!(removed >= 3, "{name}: the object, its link, its base: {removed}");
            assert_eq!(file.blocks.len(), blocks - removed);
            let after = BlendFile::parse_bytes(&file.to_bytes()).unwrap().extract_scene().unwrap();
            assert!(!after.objects.iter().any(|o| o.name == victim), "{name}: {victim} is gone");
            assert_eq!(after.objects.len(), before.objects.len(), "{name}: one in, one out");
            assert!(after.objects.iter().any(|o| o.name == "Tri"));
        }
    }
}
