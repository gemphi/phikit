//! A data block's place in its ID's run — Blender reads an ID's data from
//! the blocks written right after it, so what an edit adds to an ID goes
//! at the end of that run — and the reads and writes of fields on blocks
//! found there by address.

use crate::blend::anim::fcurve::BFCurve;
use crate::blend::file::types::*;

impl BlendFile {
    /// The ID block at `address` with `code`.
    pub(super) fn id_index(&self, address: u64, code: &str) -> Option<usize> {
        self.blocks.iter().position(|b| b.old_address == address && b.code_str == code)
    }

    /// The data block at `address` in the run of the ID at `owner`.
    pub(super) fn run_block(&self, owner: u64, address: u64) -> Option<usize> {
        let id = self.blocks.iter().position(|b| b.old_address == owner && b.code_str != "DATA")?;
        let end = self.blocks[id + 1..].iter().position(|b| b.code_str != "DATA").map_or(self.blocks.len(), |n| id + 1 + n);
        (id + 1..end).find(|&i| self.blocks[i].old_address == address)
    }

    /// The last block of `owner`'s run: where its new data goes, after it.
    pub(super) fn run_last(&self, owner: u64) -> Option<usize> {
        let id = self.blocks.iter().position(|b| b.old_address == owner && b.code_str != "DATA")?;
        Some(self.blocks[id + 1..].iter().position(|b| b.code_str != "DATA").map_or(self.blocks.len(), |n| id + 1 + n) - 1)
    }

    /// A new block of `count` × `struct_name` (zeroed), or of raw bytes when
    /// `struct_name` is `None`, at the end of `owner`'s run; its address.
    pub(super) fn push_owned(&mut self, owner: u64, struct_name: Option<&str>, count: usize, raw: Vec<u8>) -> Option<u64> {
        let after = self.run_last(owner)?;
        let (sdna_index, data) = match struct_name {
            Some(st) => (self.sdna_index_of(st)?, vec![0u8; self.sizeof(st)? * count]),
            None => (0, raw),
        };
        let address = self.fresh_address();
        let block = BlendBlock { code: *b"DATA", code_str: "DATA".to_string(), size: data.len() as u64, old_address: address, sdna_index, count: count.max(1) as u64, data };
        self.blocks.insert(after + 1, block);
        self.reindex();
        Some(address)
    }

    pub(super) fn pointer_bytes(&self, list: &[u64]) -> Vec<u8> {
        let (ps, le) = (self.header.pointer_size as usize, self.header.little_endian);
        list.iter().flat_map(|a| if le { a.to_le_bytes()[..ps].to_vec() } else { a.to_be_bytes()[8 - ps..].to_vec() }).collect()
    }

    /// Point `st.field` of the block at `block` (in `owner`'s run) at a
    /// pointer array holding `list`, `num` its count: the array's block
    /// rewritten, or made when there was none.
    pub(super) fn set_pointer_array(&mut self, owner: u64, block: u64, st: &str, field: &str, num: &str, list: &[u64]) -> Option<()> {
        let dna = self.dna.clone()?;
        let i = self.run_block(owner, block).or_else(|| self.id_index(block, "AC"))?;
        let bytes = self.pointer_bytes(list);
        let old = self.ptr(&self.blocks[i], self.off(st, field, &dna));
        match self.run_block(owner, old).filter(|_| old != 0) {
            Some(at) => {
                let b = &mut self.blocks[at];
                (b.size, b.data) = (bytes.len() as u64, bytes);
            }
            None => {
                let made = self.push_owned(owner, None, 1, bytes)?;
                let i = self.run_block(owner, block).or_else(|| self.id_index(block, "AC"))?;
                self.set_ptr(i, st, field, made);
            }
        }
        let i = self.run_block(owner, block).or_else(|| self.id_index(block, "AC"))?;
        self.set_int(i, st, num, list.len() as i64);
        Some(())
    }

    /// The pointers of `st.field` (`num` of them) on the block at `block`.
    pub(super) fn pointer_list(&self, owner: u64, block: u64, st: &str, field: &str, num: &str) -> Vec<u64> {
        let Some(dna) = self.dna.as_ref() else { return Vec::new() };
        let Some(i) = self.run_block(owner, block).or_else(|| self.id_index(block, "AC")) else { return Vec::new() };
        let b = &self.blocks[i];
        let Some(array) = self.run_block(owner, self.ptr(b, self.off(st, field, dna))) else { return Vec::new() };
        let (n, ps) = (self.int_field(b, st, num, dna) as usize, self.header.pointer_size as usize);
        let data = &self.blocks[array].data;
        (0..n.min(data.len() / ps)).map(|k| Self::read_ptr(data, k * ps, ps, self.header.little_endian)).collect()
    }

    /// `curve`'s keys as a `BezTriple` array.
    /// A key the curve had at the same frame in `old` (its `BezTriple`
    /// array) keeps what phiber does not model — its keyframe type, radius,
    /// weight — under the values written over it.
    pub(super) fn bezt_bytes(&self, curve: &BFCurve, old: &[u8]) -> Option<Vec<u8>> {
        let dna = self.dna.as_ref()?;
        let size = self.sizeof("BezTriple")?;
        let le = self.header.little_endian;
        let f32b = |v: f32| if le { v.to_le_bytes() } else { v.to_be_bytes() };
        let off = |f: &str| self.off("BezTriple", f, dna);
        let vec = off("vec[3][3]")?;
        let frame_of = |t: &[u8]| {
            let b: [u8; 4] = t[vec + 12..vec + 16].try_into().unwrap_or_default();
            if le { f32::from_le_bytes(b) } else { f32::from_be_bytes(b) }
        };
        let mut out = vec![0u8; size * curve.keys.len()];
        for (k, key) in curve.keys.iter().enumerate() {
            let d = &mut out[k * size..(k + 1) * size];
            if let Some(was) = old.chunks_exact(size).find(|t| frame_of(t) == key.co[0]) {
                d.copy_from_slice(was);
            }
            for (n, v) in [key.h1[0], key.h1[1], 0.0, key.co[0], key.co[1], 0.0, key.h2[0], key.h2[1], 0.0].into_iter().enumerate() {
                d[vec + n * 4..vec + n * 4 + 4].copy_from_slice(&f32b(v));
            }
            for (field, value) in [("ipo", key.ipo), ("h1", key.h1_type), ("h2", key.h2_type), ("f1", key.h1_selected as u8), ("f2", key.selected as u8), ("f3", key.h2_selected as u8), ("easing", key.easing)] {
                if let Some(o) = off(field) {
                    d[o] = value;
                }
            }
            for (field, value) in [("back", key.back), ("amplitude", key.amplitude), ("period", key.period)] {
                if let Some(o) = off(field) {
                    d[o..o + 4].copy_from_slice(&f32b(value));
                }
            }
        }
        Some(out)
    }
}

impl BlendFile {
    /// The block at `address`: data in `owner`'s run, or the owner itself.
    pub(super) fn owned(&self, owner: u64, address: u64) -> Option<usize> {
        self.run_block(owner, address).or_else(|| self.blocks.iter().position(|b| b.old_address == address && b.old_address == owner))
    }

    /// Set `field` on the block at `address`, the first of `names` the
    /// file's DNA has (the DNA keeps a field's first name).
    pub(super) fn set_text_any(&mut self, owner: u64, address: u64, st: &str, names: &[&str], text: &str) -> bool {
        let Some(i) = self.owned(owner, address) else { return false };
        names.iter().any(|name| self.set_text(i, st, name, text))
    }

    pub(super) fn set_ptr_any(&mut self, owner: u64, address: u64, st: &str, names: &[&str], to: u64) -> bool {
        let Some(i) = self.owned(owner, address) else { return false };
        names.iter().any(|name| self.set_ptr(i, st, name, to))
    }

    pub(super) fn set_int_at(&mut self, owner: u64, address: u64, st: &str, field: &str, value: i64) -> bool {
        self.owned(owner, address).is_some_and(|i| self.set_int(i, st, field, value))
    }

    pub(super) fn set_f32_at(&mut self, owner: u64, address: u64, st: &str, field: &str, value: f32) -> bool {
        self.owned(owner, address).is_some_and(|i| self.set_f32(i, st, field, value))
    }

    pub(super) fn int_at(&self, owner: u64, address: u64, st: &str, field: &str) -> i64 {
        let Some(dna) = self.dna.as_ref() else { return 0 };
        self.owned(owner, address).map_or(0, |i| i64::from(self.int_field(&self.blocks[i], st, field, dna) as i32))
    }

    pub(super) fn ptr_at(&self, owner: u64, address: u64, st: &str, field: &str) -> u64 {
        let Some(dna) = self.dna.as_ref() else { return 0 };
        self.owned(owner, address).map_or(0, |i| self.ptr(&self.blocks[i], self.off(st, field, dna)))
    }
}
