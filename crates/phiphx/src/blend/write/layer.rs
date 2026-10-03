//! The blocks a save patches by walking from the scene: the current scene
//! itself, its view layer, the `Base` per object, the `LayerCollection`
//! per collection, and the collection tree by path — the same walks
//! `layer.rs` and `collection.rs` make to read, kept to block indices so
//! the writer can patch what they find.

use crate::blend::file::types::*;

/// One object's `Base` in the view layer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlendBaseAt {
    pub base: usize,
    pub base_address: u64,
    pub object_address: u64,
}

/// One collection: where its `Collection` and its `LayerCollection` are.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlendCollectionAt {
    /// Names from the scene collection down, joined by `/` — the path the
    /// Outliner keys the row on.
    pub path: String,
    pub collection: usize,
    pub collection_address: u64,
    pub layer_collection: Option<usize>,
}

impl BlendFile {
    /// The scene Blender opens (`FileGlobal.curscene`), else the first.
    pub fn current_scene_index(&self) -> Option<usize> {
        let dna = self.dna.as_ref()?;
        self.file_global_curscene(dna)
            .and_then(|b| self.index_of(b))
            .or_else(|| self.blocks.iter().position(|b| b.code_str == "SC"))
    }

    /// The view layer of the scene at `scene` (`FileGlobal.cur_view_layer`,
    /// else the scene's first).
    pub fn view_layer_index(&self, scene: usize) -> Option<usize> {
        let dna = self.dna.as_ref()?;
        let block = self.view_layer_block_for(self.blocks.get(scene)?, dna)?;
        self.index_of(block)
    }

    /// Every `Base` of the view layer, in list order.
    pub fn bases(&self, view_layer: usize) -> Vec<BlendBaseAt> {
        let Some(dna) = self.dna.as_ref() else { return Vec::new() };
        let Some(vl) = self.blocks.get(view_layer) else { return Vec::new() };
        let next_off = self.off("Base", "*next", dna);
        let ob_off = self.off("Base", "*object", dna);
        let mut out = Vec::new();
        let mut addr = self.ptr(vl, self.off("ViewLayer", "object_bases", dna));
        while let Some(base) = self.block_at(addr) {
            if out.len() > 1_000_000 {
                break;
            }
            if let Some(index) = self.index_of(base) {
                out.push(BlendBaseAt { base: index, base_address: addr, object_address: self.ptr(base, ob_off) });
            }
            addr = self.ptr(base, next_off);
        }
        out
    }

    /// The collection tree under the scene's master collection, by path,
    /// with each collection's `LayerCollection` in the view layer.
    pub fn collections_at(&self, scene: usize, view_layer: usize) -> Vec<BlendCollectionAt> {
        let Some(dna) = self.dna.as_ref() else { return Vec::new() };
        let Some(sc) = self.blocks.get(scene) else { return Vec::new() };
        let master = self.ptr(sc, self.off("Scene", "*master_collection", dna));
        let mut layers = std::collections::HashMap::new();
        if let Some(vl) = self.blocks.get(view_layer) {
            let first = self.ptr(vl, self.off("ViewLayer", "layer_collections", dna));
            self.layer_collection_blocks(first, dna, &mut layers, 0);
        }
        let mut out = Vec::new();
        self.collection_paths(master, "", dna, &layers, &mut out, 0);
        out
    }

    fn layer_collection_blocks(&self, mut addr: u64, dna: &DnaType, out: &mut std::collections::HashMap<u64, usize>, depth: u32) {
        if depth > 64 {
            return;
        }
        let next_off = self.off("LayerCollection", "*next", dna);
        let col_off = self.off("LayerCollection", "*collection", dna);
        let kids_off = self.off("LayerCollection", "layer_collections", dna);
        let mut guard = 0;
        while let Some(lc) = self.block_at(addr) {
            guard += 1;
            if guard > 100_000 {
                break;
            }
            if let Some(index) = self.index_of(lc) {
                out.insert(self.ptr(lc, col_off), index);
            }
            self.layer_collection_blocks(self.ptr(lc, kids_off), dna, out, depth + 1);
            addr = self.ptr(lc, next_off);
        }
    }

    fn collection_paths(
        &self,
        addr: u64,
        parent: &str,
        dna: &DnaType,
        layers: &std::collections::HashMap<u64, usize>,
        out: &mut Vec<BlendCollectionAt>,
        depth: u32,
    ) {
        if depth > 64 {
            return;
        }
        let Some(block) = self.block_at(addr) else { return };
        let Some(index) = self.index_of(block) else { return };
        let mut name = self.extract_name_from_block(block, "id.name").unwrap_or_default();
        if name.is_empty() {
            name = "Scene Collection".to_string();
        }
        let path = if parent.is_empty() { name } else { format!("{parent}/{name}") };
        out.push(BlendCollectionAt { path: path.clone(), collection: index, collection_address: addr, layer_collection: layers.get(&addr).copied() });
        let mut child = self.ptr(block, self.off("Collection", "children", dna));
        let next_off = self.off("CollectionChild", "*next", dna);
        let col_off = self.off("CollectionChild", "*collection", dna);
        let mut guard = 0;
        while let Some(link) = self.block_at(child) {
            guard += 1;
            if guard > 100_000 {
                break;
            }
            self.collection_paths(self.ptr(link, col_off), &path, dna, layers, out, depth + 1);
            child = self.ptr(link, next_off);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_scene_its_bases_and_its_collections_are_found_by_index() {
        let path = format!("{}/tests/fixtures/nla_cube.blend", env!("CARGO_MANIFEST_DIR"));
        let file = BlendFile::parse_bytes(&std::fs::read(path).expect("fixture")).expect("parse");
        let scene = file.current_scene_index().expect("a scene");
        assert_eq!(file.blocks[scene].code_str, "SC");
        let view_layer = file.view_layer_index(scene).expect("a view layer");
        let bases = file.bases(view_layer);
        assert!(!bases.is_empty(), "the cube has a base");
        let cube = file.find_id("OB", "Cube").expect("the cube");
        assert!(bases.iter().any(|b| file.index_at(b.object_address) == Some(cube)), "a base points at the cube");
        let collections = file.collections_at(scene, view_layer);
        assert_eq!(collections[0].path, "Scene Collection");
        assert!(collections.iter().skip(1).all(|c| c.path.starts_with("Scene Collection/")), "{:?}", collections.iter().map(|c| &c.path).collect::<Vec<_>>());
        assert!(collections.iter().all(|c| c.layer_collection.is_some()), "every collection has its layer collection");
    }
}
