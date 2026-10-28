//! Who instances and what is instanced: the visibility half of Blender's
//! dupli system, translated rule for rule.
//!
//! - An instancer draws its instances only while its base in the open view
//!   layer is `BASE_ENABLED_AND_MAYBE_VISIBLE_IN_VIEWPORT`
//!   (`BKE_object_visibility`, which the depsgraph's object iterator asks
//!   before `object_duplilist`). [`BlendFile::viewport_bases`] works that
//!   flag out as `BKE_layer_collection_sync` and `BKE_base_eval_flags` do.
//! - It instances a collection only when `get_dupli_generator` picks
//!   `gen_dupli_collection` for it ([`collection_generator`]).
//! - The members are `FOREACH_COLLECTION_VISIBLE_OBJECT_RECURSIVE` over
//!   `BKE_collection_object_cache_get` ([`BlendFile::collection_cache`]):
//!   the collection's view-layer state plays no part, only `Collection.flag`
//!   down the tree and the member's own `OB_HIDE_VIEWPORT`.

use super::layer::{BASE_HIDDEN, LAYER_COLLECTION_EXCLUDE};
use super::visibility::{COLLECTION_HIDE_VIEWPORT, OB_HIDE_VIEWPORT};
use crate::blend::file::types::*;
use std::collections::HashSet;

/// `Object.transflag` bits (`DNA_object_types.h`).
pub const OB_DUPLIVERTS: u32 = 1 << 4;
pub const OB_DUPLICOLLECTION: u32 = 1 << 8;
pub const OB_DUPLIFACES: u32 = 1 << 9;
pub const OB_DUPLIPARTS: u32 = 1 << 11;
/// `Collection.flag`: the scene's master collection.
const COLLECTION_IS_MASTER: u32 = 1 << 5;

/// One object of an instanced collection, as the dupli loop reads it.
pub(crate) struct DupliMember {
    pub object: BlendSceneObject,
    /// `Object.transflag` and `Object.dup_group`: what it instances itself.
    pub transflag: u32,
    pub collection: u64,
}

/// `get_dupli_generator` (`object_dupli.cc`) answering `gen_dupli_collection`:
/// not with the viewport restriction set, and not when particles, vertices
/// or faces take precedence — the `else if` chain tests them first.
/// Geometry-set instances (Geometry Nodes) are not modelled.
pub(crate) fn collection_generator(transflag: u32, visibility_flag: u32) -> bool {
    match () {
        _ if visibility_flag & OB_HIDE_VIEWPORT != 0 => false,
        _ if transflag & (OB_DUPLIPARTS | OB_DUPLIVERTS | OB_DUPLIFACES) != 0 => false,
        _ => transflag & OB_DUPLICOLLECTION != 0,
    }
}

impl BlendFile {
    /// The objects of the open view layer whose base is
    /// `BASE_ENABLED_AND_MAYBE_VISIBLE_IN_VIEWPORT`, or `None` for a file with
    /// no view layer (every object counts then).
    ///
    /// `layer_collection_sync` gives an object `BASE_ENABLED_VIEWPORT` from
    /// any layer collection that is not `LAYER_COLLECTION_EXCLUDE` (exclusion
    /// is not inherited) and whose collections up from it carry no
    /// `COLLECTION_HIDE_VIEWPORT` (the master's flag aside); the eye
    /// (`LAYER_COLLECTION_HIDE`) does not take it away. `BKE_base_eval_flags`
    /// then clears it for `OB_HIDE_VIEWPORT` and the base's `BASE_HIDDEN`.
    pub(crate) fn viewport_bases(&self, dna: &DnaType) -> Option<HashSet<u64>> {
        let scene = self.file_global_curscene(dna).or_else(|| self.blocks.iter().find(|b| b.code_str == "SC"))?;
        let layer = self.view_layer_block_for(scene, dna)?;
        let root = self.ptr(layer, self.off("ViewLayer", "layer_collections", dna));
        self.block_at(root)?;
        let mut enabled = HashSet::new();
        self.sync_layer_collections(root, 0, dna, &mut enabled, 0);
        let bases = self.extract_view_layer(scene, dna).base_flags;
        let field = self.object_visibility_field(dna);
        enabled.retain(|ob| {
            let hidden = bases.get(ob).is_some_and(|f| f & BASE_HIDDEN != 0);
            let restricted = self.block_at(*ob).is_some_and(|b| self.int_field(b, "Object", field, dna) & OB_HIDE_VIEWPORT != 0);
            !hidden && !restricted
        });
        Some(enabled)
    }

    /// `layer_collection_sync` / `layer_collection_objects_sync` over the
    /// layer collections from `addr` on, with the collection restriction
    /// inherited from above.
    fn sync_layer_collections(&self, addr: u64, restrict: u32, dna: &DnaType, enabled: &mut HashSet<u64>, depth: u32) {
        let next = self.off("LayerCollection", "*next", dna);
        for lc in self.list(addr, next).into_iter().filter(|_| depth <= 64) {
            let Some(collection) = self.block_at(self.ptr(lc, self.off("LayerCollection", "*collection", dna))) else { continue };
            let flag = self.int_field(collection, "Collection", "flag", dna);
            let restrict = match flag & COLLECTION_IS_MASTER {
                0 => restrict | flag,
                _ => restrict,
            };
            let kids = self.ptr(lc, self.off("LayerCollection", "layer_collections", dna));
            self.sync_layer_collections(kids, restrict, dna, enabled, depth + 1);
            let excluded = self.int_field(lc, "LayerCollection", "flag", dna) & LAYER_COLLECTION_EXCLUDE != 0;
            if excluded || restrict & COLLECTION_HIDE_VIEWPORT != 0 {
                continue;
            }
            enabled.extend(self.collection_objects(collection, dna));
        }
    }

    /// `Collection.gobject`: the addresses of the collection's own objects.
    fn collection_objects(&self, block: &BlendBlock, dna: &DnaType) -> Vec<u64> {
        let first = self.ptr(block, self.off("Collection", "gobject", dna));
        let ob = self.off("CollectionObject", "*ob", dna);
        self.list(first, self.off("CollectionObject", "*next", dna)).into_iter().map(|l| self.ptr(l, ob)).filter(|&a| a != 0).collect()
    }

    /// The objects `FOREACH_COLLECTION_VISIBLE_OBJECT_RECURSIVE` yields for
    /// the collection at `addr` in the viewport, in its order.
    ///
    /// `collection_object_cache_fill` lists each object once, the
    /// collection's own before its children's, and enables its base unless
    /// `COLLECTION_HIDE_VIEWPORT` is set on the collection or any collection
    /// above it in the walk — the instanced collection's own flag included;
    /// an object reached twice is enabled if either path enables it. The
    /// loop then skips `OB_HIDE_VIEWPORT` objects, and the depsgraph drops
    /// a dupli `DEG_iterator_dupli_is_visible` refuses: a meta-ball, or an
    /// object whose parent instances on its vertices or faces
    /// (`deg_object_hide_original`).
    pub(crate) fn collection_cache(&self, addr: u64, dna: &DnaType) -> Vec<(u64, u32)> {
        let mut cache: Vec<(u64, bool)> = Vec::new();
        self.cache_fill(addr, 0, dna, &mut cache, 0);
        let field = self.object_visibility_field(dna);
        let parent = self.off("Object", "*parent", dna);
        cache
            .into_iter()
            .filter_map(|(ob, enabled)| {
                let block = self.block_at(ob).filter(|_| enabled)?;
                let hidden = self.int_field(block, "Object", field, dna) & OB_HIDE_VIEWPORT != 0;
                let meta = self.int_field(block, "Object", "type", dna) == OB_MBALL;
                let by_parent = self.block_at(self.ptr(block, parent)).is_some_and(|p| self.int_field(p, "Object", "transflag", dna) & (OB_DUPLIVERTS | OB_DUPLIFACES) != 0);
                (!hidden && !meta && !by_parent).then(|| (ob, self.int_field(block, "Object", "transflag", dna)))
            })
            .collect()
    }

    /// `collection_object_cache_fill` (`collection.cc`).
    fn cache_fill(&self, addr: u64, parent_restrict: u32, dna: &DnaType, cache: &mut Vec<(u64, bool)>, depth: u32) {
        let Some(block) = self.block_at(addr).filter(|_| depth <= 64) else { return };
        let restrict = self.int_field(block, "Collection", "flag", dna) | parent_restrict;
        let enabled = restrict & COLLECTION_HIDE_VIEWPORT == 0;
        for ob in self.collection_objects(block, dna) {
            match cache.iter_mut().find(|(at, _)| *at == ob) {
                Some(base) => base.1 |= enabled,
                None => cache.push((ob, enabled)),
            }
        }
        let child = self.off("CollectionChild", "*collection", dna);
        for link in self.list(self.ptr(block, self.off("Collection", "children", dna)), self.off("CollectionChild", "*next", dna)) {
            self.cache_fill(self.ptr(link, child), restrict, dna, cache, depth + 1);
        }
    }
}

/// `Object.type` of a meta-ball (`OB_MBALL`).
const OB_MBALL: u32 = 5;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_shown_collection_instancer_generates() {
        assert!(collection_generator(OB_DUPLICOLLECTION, 0));
        assert!(!collection_generator(OB_DUPLICOLLECTION, OB_HIDE_VIEWPORT), "restricted in viewports");
        assert!(!collection_generator(OB_DUPLICOLLECTION | OB_DUPLIVERTS, 0), "vertices come first");
        assert!(!collection_generator(OB_DUPLICOLLECTION | OB_DUPLIPARTS, 0), "particles come first");
        assert!(!collection_generator(0, 0));
    }
}
