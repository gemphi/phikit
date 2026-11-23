//! Shape keys, as the evaluated mesh starts from them — Blender
//! `BKE_key_evaluate_object` (`blenkernel/intern/key.cc`) for a relative
//! key, `key_evaluate_relative`: the reference key's positions, plus for
//! every other key block that is not muted and has a value, that value
//! times the block's offset from the block it is relative to, weighted by
//! the block's vertex group where it names one. This runs before the
//! modifier stack (`BKE_mesh_calc_modifiers` evaluates the key first).
//!
//! The value is `KeyBlock.curval` as the file saved it: what the slider,
//! its animation or its driver last left there.
//!
//! Nothing read them: a mesh came in as its stored positions, so a boot
//! shaped to its foot by a key at 1.0, or hair fitted by one, drew in its
//! basis shape.
//!
//! Not here: absolute keys (`KEY_NORMAL`, positions over `Key.ctime`),
//! which leave the mesh as stored, and an object's Shape Key Lock
//! (`OB_SHAPE_LOCK`), which shows the active key alone.

use crate::blend::file::types::*;

/// `Key.type`: states to blend between.
const KEY_RELATIVE: u32 = 1;
/// `KeyBlock.flag`: left out of the mix.
const KEYBLOCK_MUTE: u32 = 1;

/// One key block: its positions, value and what it is measured against.
struct Block {
    data: Vec<[f32; 3]>,
    value: f32,
    relative: usize,
    muted: bool,
    group: String,
}

impl BlendFile {
    /// `verts` as the mesh's shape keys leave them. The mesh's own
    /// positions when it has no relative key, or a key whose blocks do not
    /// match its vertices.
    pub(crate) fn shaped(&self, me: &BlendBlock, verts: Vec<[f32; 3]>, dna: &DnaType) -> Vec<[f32; 3]> {
        let Some(key) = self.block_at(self.ptr(me, self.off("Mesh", "*key", dna))) else { return verts };
        if self.int_field(key, "Key", "type", dna) != KEY_RELATIVE {
            return verts;
        }
        let block_list = self.key_blocks(key, verts.len(), dna);
        let reference = self.ptr(key, self.off("Key", "*refkey", dna));
        let first = self.ptr(key, self.off("Key", "block", dna));
        // The reference is the first block in every file Blender writes;
        // one that is not would need its place in the list.
        let Some(basis) = block_list.first().filter(|b| reference == first && b.data.len() == verts.len()) else { return verts };
        let live: Vec<&Block> = block_list.iter().skip(1).filter(|b| !b.muted && b.value != 0.0 && b.data.len() == verts.len()).collect();
        if live.is_empty() {
            return verts;
        }
        let weight_list = self.key_weights(me, &live, verts.len(), dna);
        let mut out = basis.data.clone();
        for (block, weights) in live.iter().zip(&weight_list) {
            let Some(from) = block_list.get(block.relative).filter(|r| r.data.len() == verts.len()) else { continue };
            for (v, at) in out.iter_mut().enumerate() {
                let share = block.value * weights.as_ref().map_or(1.0, |w| w[v]);
                for k in 0..3 {
                    at[k] += share * (block.data[v][k] - from.data[v][k]);
                }
            }
        }
        out
    }

    /// The key's blocks in list order, each with `verts` positions when
    /// its data holds that many.
    fn key_blocks(&self, key: &BlendBlock, verts: usize, dna: &DnaType) -> Vec<Block> {
        let name_at = self.off("KeyBlock", "vgroup[64]", dna);
        self.list(self.ptr(key, self.off("Key", "block", dna)), self.off("KeyBlock", "*next", dna))
            .into_iter()
            .map(|kb| {
                let held = self.int_field(kb, "KeyBlock", "totelem", dna) as usize;
                let data = match (held == verts, self.block_at(self.ptr(kb, self.off("KeyBlock", "*data", dna)))) {
                    (true, Some(d)) if d.data.len() >= verts * 12 => (0..verts).map(|v| [0, 4, 8].map(|o| self.read_f32(&d.data[v * 12 + o..v * 12 + o + 4]))).collect(),
                    _ => Vec::new(),
                };
                let group = name_at
                    .and_then(|o| kb.data.get(o..o + 64))
                    .map(|bytes| String::from_utf8_lossy(&bytes[..bytes.iter().position(|b| *b == 0).unwrap_or(bytes.len())]).into_owned())
                    .unwrap_or_default();
                Block {
                    data,
                    value: self.float_field(kb, "KeyBlock", "curval", dna, 0.0),
                    relative: self.int_field(kb, "KeyBlock", "relative", dna) as usize,
                    muted: self.int_field(kb, "KeyBlock", "flag", dna) & KEYBLOCK_MUTE != 0,
                    group,
                }
            })
            .collect()
    }

    /// Per live block, its vertex group's weight at each vertex
    /// (`BKE_keyblock_get_per_block_weights`): `None` for a block that
    /// names no group. A group the mesh does not have weighs nothing in,
    /// as a missing group's weights are all zero in Blender.
    fn key_weights(&self, me: &BlendBlock, live: &[&Block], verts: usize, dna: &DnaType) -> Vec<Option<Vec<f32>>> {
        if live.iter().all(|b| b.group.is_empty()) {
            return vec![None; live.len()];
        }
        // The names are the mesh's own from 3.0 on (`vertex_group_names`);
        // an older file keeps them on the object, which a mesh read alone
        // does not have, so its grouped keys weigh in whole.
        let named = self.off("Mesh", "vertex_group_names", dna).is_some();
        let names = if named { self.vertex_group_names(me, me, dna) } else { Vec::new() };
        let per_vert = self.deform_verts(me, dna, verts);
        live.iter()
            .map(|b| match (b.group.is_empty(), named) {
                (true, _) | (false, false) => None,
                (false, true) => {
                    let index = names.iter().position(|n| *n == b.group).map(|i| i as u32);
                    Some((0..verts).map(|v| per_vert.get(v).and_then(|ws| ws.iter().find(|(g, _)| Some(*g) == index)).map_or(0.0, |(_, w)| *w)).collect())
                }
            })
            .collect()
    }
}
