//! View layer extraction — what Blender's Outliner "View Layer" mode shows.
//!
//! Blender ref (`DNA_layer_types.h`): `Scene.view_layers` → `ViewLayer`
//! { `object_bases` (list of `Base { *object, flag }`), `*basact`,
//! `layer_collections` (tree of `LayerCollection { *collection, flag,
//! layer_collections }`) }. `BASE_SELECTED = 1<<0`, `BASE_HIDDEN = 1<<8`,
//! `LAYER_COLLECTION_EXCLUDE = 1<<4`, `LAYER_COLLECTION_HIDE = 1<<7`.
//! Blender opens the layer `FileGlobal.cur_view_layer` points at; we fall
//! back to the first one.

use crate::blend::file::types::*;
use std::collections::HashMap;

pub const BASE_SELECTED: u32 = 1 << 0;
pub const BASE_HIDDEN: u32 = 1 << 8;
pub const LAYER_COLLECTION_EXCLUDE: u32 = 1 << 4;
pub const LAYER_COLLECTION_HIDE: u32 = 1 << 7;

/// Per-object and per-collection view-layer flags, keyed by saved address.
#[derive(Debug, Default, Clone)]
pub struct ViewLayerInfo {
    pub name: String,
    /// `Base.flag` per `Object` address.
    pub base_flags: HashMap<u64, u32>,
    /// `ViewLayer.basact → Base.object` address, 0 when none.
    pub active_object: u64,
    /// `LayerCollection.flag` per `Collection` address.
    pub collection_flags: HashMap<u64, u32>,
}

impl BlendFile {
    pub(crate) fn off(&self, s: &str, f: &str, dna: &DnaType) -> Option<usize> {
        Self::struct_field_offset_exact(s, f, dna, self.header.pointer_size as usize)
    }

    pub(crate) fn ptr(&self, block: &BlendBlock, off: Option<usize>) -> u64 {
        match off {
            Some(o) => Self::read_ptr(&block.data, o, self.header.pointer_size as usize, self.header.little_endian),
            None => 0,
        }
    }

    /// Read an integer field whose DNA width may be `char`, `short` or `int`.
    pub(crate) fn int_field(&self, block: &BlendBlock, s: &str, f: &str, dna: &DnaType) -> u32 {
        let ps = self.header.pointer_size as usize;
        // The member's offset and width, both from the layout worked out
        // once (`file/sdna.rs`).
        let layout = dna.layout(ps);
        let Some(&(o, width)) = layout.structure(s).and_then(|st| st.fields.get(f)) else { return 0 };
        match width {
            1 if o < block.data.len() => block.data[o] as u32,
            2 if o + 2 <= block.data.len() => self.read_u16(&block.data[o..o + 2]) as u32,
            4 if o + 4 <= block.data.len() => self.read_i32(&block.data[o..o + 4]) as u32,
            _ => 0,
        }
    }

    /// The `ViewLayer` block Blender opens for `scene_block`.
    pub(crate) fn view_layer_block_for(&self, scene_block: &BlendBlock, dna: &DnaType) -> Option<&BlendBlock> {
        self.view_layer_block(scene_block, dna)
    }

    fn view_layer_block(&self, scene_block: &BlendBlock, dna: &DnaType) -> Option<&BlendBlock> {
        // Preferred: FileGlobal.cur_view_layer.
        if let Some(glob) = self.blocks.iter().find(|b| b.code_str == "GLOB") {
            let vl = self.ptr(glob, self.off("FileGlobal", "*cur_view_layer", dna));
            if let Some(b) = self.block_at(vl) {
                return Some(b);
            }
        }
        // Fallback: Scene.view_layers.first.
        let first = self.ptr(scene_block, self.off("Scene", "view_layers", dna));
        self.block_at(first)
    }

    /// Walk `object_bases` and `layer_collections` of the active view layer.
    pub(crate) fn extract_view_layer(&self, scene_block: &BlendBlock, dna: &DnaType) -> ViewLayerInfo {
        let mut info = ViewLayerInfo::default();
        let Some(vl) = self.view_layer_block(scene_block, dna) else { return info };
        if let Some(o) = self.off("ViewLayer", "name[64]", dna) {
            let end = (o + 64).min(vl.data.len());
            let bytes = &vl.data[o.min(end)..end];
            let n = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
            info.name = String::from_utf8_lossy(&bytes[..n]).to_string();
        }

        // Bases.
        let mut addr = self.ptr(vl, self.off("ViewLayer", "object_bases", dna));
        let next_off = self.off("Base", "*next", dna);
        let ob_off = self.off("Base", "*object", dna);
        let mut guard = 0;
        while let Some(base) = self.block_at(addr) {
            guard += 1;
            if guard > 1_000_000 { break; }
            let ob = self.ptr(base, ob_off);
            if ob != 0 {
                info.base_flags.insert(ob, self.int_field(base, "Base", "flag", dna));
            }
            addr = self.ptr(base, next_off);
        }
        let basact = self.ptr(vl, self.off("ViewLayer", "*basact", dna));
        if let Some(base) = self.block_at(basact) {
            info.active_object = self.ptr(base, ob_off);
        }

        // Layer collections (recursive).
        let first = self.ptr(vl, self.off("ViewLayer", "layer_collections", dna));
        self.walk_layer_collections(first, dna, &mut info.collection_flags, 0);
        info
    }

    fn walk_layer_collections(&self, mut addr: u64, dna: &DnaType, out: &mut HashMap<u64, u32>, depth: u32) {
        if depth > 64 { return; }
        let next_off = self.off("LayerCollection", "*next", dna);
        let col_off = self.off("LayerCollection", "*collection", dna);
        let kids_off = self.off("LayerCollection", "layer_collections", dna);
        let mut guard = 0;
        while let Some(lc) = self.block_at(addr) {
            guard += 1;
            if guard > 100_000 { break; }
            let col = self.ptr(lc, col_off);
            if col != 0 {
                out.insert(col, self.int_field(lc, "LayerCollection", "flag", dna));
            }
            self.walk_layer_collections(self.ptr(lc, kids_off), dna, out, depth + 1);
            addr = self.ptr(lc, next_off);
        }
    }
}
