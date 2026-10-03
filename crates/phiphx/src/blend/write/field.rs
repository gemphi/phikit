//! Writing one field of one block, by the file's own DNA — the mirror of
//! `int_field` / `f32_at` / `read_ptr`: the struct's layout says where the
//! field sits and how wide it is, so the same code patches a `short` in a
//! 2.79 file and an `int` in a 5.0 one. Plus the block-level helpers a
//! patch needs: an ID block by name, a new `DATA` block, a block dropped.

use crate::blend::file::types::*;

/// The dimensions in `[4][4]`.
fn dims(spec: &str) -> Vec<usize> {
    spec.split('[').filter(|s| !s.is_empty()).filter_map(|s| s.trim_end_matches(']').parse().ok()).collect()
}

impl BlendFile {
    /// One field of `struct_name`: its offset, its size and its type.
    /// `field` is the DNA spelling less the array dimensions (`name` for
    /// `name[66]`, `*next` for the pointer), or one element of an array
    /// (`loc[1]`, `parentinv[3][2]`), which is that element's span.
    fn field_of(&self, struct_name: &str, field: &str, dna: &DnaType) -> Option<(usize, usize, String)> {
        let ps = self.header.pointer_size as usize;
        let struct_idx = dna.types.iter().position(|t| t == struct_name)?;
        let s = dna.structs.iter().find(|s| s.type_index as usize == struct_idx)?;
        let (base, wanted): (&str, Vec<usize>) = match field.find('[') {
            Some(at) => (&field[..at], dims(&field[at..])),
            None => (field, Vec::new()),
        };
        let mut offset = 0;
        for fld in &s.fields {
            let f_name = &dna.names[fld.name_index as usize];
            let f_type = &dna.types[fld.type_index as usize];
            let size = Self::field_size(f_type, f_name, dna, ps);
            let (f_base, f_dims) = match f_name.find('[') {
                Some(at) => (&f_name[..at], dims(&f_name[at..])),
                None => (f_name.as_str(), Vec::new()),
            };
            if f_name == field || (f_base == base && !f_dims.is_empty()) {
                if wanted.is_empty() || wanted.len() != f_dims.len() {
                    return Some((offset, size, f_type.clone()));
                }
                // One element: row-major over the declared dimensions.
                let elem = size / f_dims.iter().product::<usize>().max(1);
                let mut flat = 0;
                for (i, n) in wanted.iter().zip(&f_dims) {
                    if i >= n {
                        return None;
                    }
                    flat = flat * n + i;
                }
                return Some((offset + flat * elem, elem, f_type.clone()));
            }
            offset += size;
        }
        None
    }

    /// Where `path` sits in a block of `struct_name`, and how many bytes it
    /// takes — `cfra` in a `RenderData`, or `r.cfra` in a `Scene`, walking
    /// the embedded structs a dot at a time.
    pub(crate) fn field_span(&self, struct_name: &str, path: &str, dna: &DnaType) -> Option<(usize, usize)> {
        let mut owner = struct_name.to_string();
        let mut base = 0;
        let mut parts = path.split('.').peekable();
        while let Some(part) = parts.next() {
            let (off, size, ty) = self.field_of(&owner, part, dna)?;
            base += off;
            if parts.peek().is_none() {
                return Some((base, size));
            }
            owner = ty;
        }
        None
    }

    /// The index in `dna.structs` of `struct_name` — a block's `sdna_index`.
    pub fn sdna_index_of(&self, struct_name: &str) -> Option<u32> {
        let dna = self.dna.as_ref()?;
        let type_idx = dna.types.iter().position(|t| t == struct_name)?;
        dna.structs.iter().position(|s| s.type_index as usize == type_idx).map(|i| i as u32)
    }

    /// `sizeof(struct_name)` in this file.
    pub fn sizeof(&self, struct_name: &str) -> Option<usize> {
        let dna = self.dna.as_ref()?;
        let type_idx = dna.types.iter().position(|t| t == struct_name)?;
        dna.type_sizes.get(type_idx).map(|s| *s as usize)
    }

    /// Write `value` into an integer field, as wide as the DNA says
    /// (`char`, `short`, `int`, `int64`). False when the field is not there.
    pub fn set_int(&mut self, block: usize, struct_name: &str, field: &str, value: i64) -> bool {
        let Some(dna) = self.dna.as_ref() else { return false };
        let Some((off, size)) = self.field_span(struct_name, field, dna) else { return false };
        let le = self.header.little_endian;
        let Some(data) = self.blocks.get_mut(block).map(|b| &mut b.data) else { return false };
        if off + size > data.len() {
            return false;
        }
        let bytes = if le { value.to_le_bytes() } else { value.to_be_bytes() };
        match size {
            1 => data[off] = value as u8,
            2 | 4 | 8 => {
                let src = if le { &bytes[..size] } else { &bytes[8 - size..] };
                data[off..off + size].copy_from_slice(src);
            }
            _ => return false,
        }
        true
    }

    /// Write `value` into a `float` field.
    pub fn set_f32(&mut self, block: usize, struct_name: &str, field: &str, value: f32) -> bool {
        let Some(dna) = self.dna.as_ref() else { return false };
        let Some((off, 4)) = self.field_span(struct_name, field, dna) else { return false };
        let le = self.header.little_endian;
        let Some(data) = self.blocks.get_mut(block).map(|b| &mut b.data) else { return false };
        if off + 4 > data.len() {
            return false;
        }
        data[off..off + 4].copy_from_slice(&if le { value.to_le_bytes() } else { value.to_be_bytes() });
        true
    }

    /// Write a pointer-sized value (a saved address) into `field`.
    pub fn set_ptr(&mut self, block: usize, struct_name: &str, field: &str, address: u64) -> bool {
        self.set_int(block, struct_name, field, address as i64)
    }

    /// Write `text` into a `char[]` field, NUL-terminated and cut to fit.
    pub fn set_text(&mut self, block: usize, struct_name: &str, field: &str, text: &str) -> bool {
        let Some(dna) = self.dna.as_ref() else { return false };
        let Some((off, size)) = self.field_span(struct_name, field, dna) else { return false };
        let Some(data) = self.blocks.get_mut(block).map(|b| &mut b.data) else { return false };
        if size == 0 || off + size > data.len() {
            return false;
        }
        let mut bytes = text.as_bytes().to_vec();
        while bytes.len() > size - 1 {
            bytes.pop();
            while !bytes.is_empty() && std::str::from_utf8(&bytes).is_err() {
                bytes.pop();
            }
        }
        data[off..off + size].fill(0);
        data[off..off + bytes.len()].copy_from_slice(&bytes);
        true
    }

    /// The ID's name as the file spells it: the two-letter code and the name.
    pub fn set_id_name(&mut self, block: usize, name: &str) -> bool {
        let Some(code) = self.blocks.get(block).map(|b| b.code_str.clone()) else { return false };
        let code: String = code.chars().take(2).collect();
        self.set_text(block, "ID", "name", &format!("{code}{name}"))
    }

    /// The index of the ID block of `code` (`OB`, `SC`, `GR`…) called `name`.
    pub fn find_id(&self, code: &str, name: &str) -> Option<usize> {
        self.blocks
            .iter()
            .enumerate()
            .filter(|(_, b)| b.code_str == code)
            .find(|(_, b)| self.extract_name_from_block(b, "id.name").is_ok_and(|n| n == name))
            .map(|(i, _)| i)
    }

    /// The index of the block saved at `address`, as the reader resolves it.
    pub fn index_at(&self, address: u64) -> Option<usize> {
        self.block_at(address).and_then(|b| self.index_of(b))
    }

    /// A saved address no block has: past the highest one, aligned.
    pub fn fresh_address(&self) -> u64 {
        let top = self.blocks.iter().map(|b| b.old_address + b.size.max(8)).max().unwrap_or(0x1000);
        (top + 0x100) & !0xF
    }

    /// A new `DATA` block of one `struct_name`, zeroed, inserted after block
    /// `after` so it sits with the ID it belongs to; its index and address.
    pub fn push_block(&mut self, struct_name: &str, after: usize) -> Option<(usize, u64)> {
        let sdna_index = self.sdna_index_of(struct_name)?;
        let size = self.sizeof(struct_name)?;
        let address = self.fresh_address();
        let block = BlendBlock {
            code: *b"DATA",
            code_str: "DATA".to_string(),
            size: size as u64,
            old_address: address,
            sdna_index,
            count: 1,
            data: vec![0; size],
        };
        let at = (after + 1).min(self.blocks.len());
        self.blocks.insert(at, block);
        self.reindex();
        Some((at, address))
    }

    /// Drop the blocks at `indices`.
    pub fn remove_blocks(&mut self, mut indices: Vec<usize>) {
        indices.sort_unstable();
        indices.dedup();
        for i in indices.into_iter().rev() {
            if i < self.blocks.len() {
                self.blocks.remove(i);
            }
        }
        self.reindex();
    }

    /// Rebuild the address maps after blocks moved.
    pub(crate) fn reindex(&mut self) {
        self.addr_index.clear();
        self.dup_index.clear();
        for (i, block) in self.blocks.iter().enumerate() {
            if block.old_address == 0 {
                continue;
            }
            let first = *self.addr_index.entry(block.old_address).or_insert(i);
            if first != i {
                self.dup_index.entry(block.old_address).or_insert_with(|| vec![first]).push(i);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> BlendFile {
        let path = format!("{}/tests/fixtures/nla_cube.blend", env!("CARGO_MANIFEST_DIR"));
        BlendFile::parse_bytes(&std::fs::read(path).expect("fixture")).expect("parse")
    }

    #[test]
    fn a_frame_a_name_and_a_flag_are_written_where_the_dna_puts_them_and_read_back() {
        let mut file = fixture();
        let dna = file.dna.clone().unwrap();
        let scene = file.blocks.iter().position(|b| b.code_str == "SC").expect("a scene");
        let (r_off, _) = file.field_span("Scene", "r", &dna).expect("RenderData");
        let (cfra_off, cfra_size) = file.field_span("RenderData", "cfra", &dna).expect("cfra");
        assert_eq!(file.field_span("Scene", "r.cfra", &dna), Some((r_off + cfra_off, cfra_size)), "a dotted path walks the embedded struct");
        assert!(file.set_int(scene, "Scene", "r.cfra", 77));
        let read = file.read_i32(&file.blocks[scene].data[r_off + cfra_off..r_off + cfra_off + 4]);
        assert_eq!(read, 77);

        let cube = file.find_id("OB", "Cube").expect("the cube");
        assert!(file.set_id_name(cube, "Crate"));
        assert_eq!(file.extract_name_from_block(&file.blocks[cube], "id.name").unwrap(), "Crate");
        assert_eq!(file.find_id("OB", "Cube"), None);
        let field = file.object_visibility_field(&dna);
        assert!(file.set_int(cube, "Object", field, 0b110));
        assert_eq!(file.int_field(&file.blocks[cube], "Object", field, &dna), 0b110);
        assert!(!file.set_int(cube, "Object", "no_such_field", 1));

        // A new block sits after its ID with an address nobody else has.
        let before = file.blocks.len();
        let (at, address) = file.push_block("TimeMarker", scene).expect("a marker block");
        assert_eq!(at, scene + 1);
        assert_eq!(file.blocks.len(), before + 1);
        assert_eq!(file.index_at(address), Some(at));
        assert!(file.set_text(at, "TimeMarker", "name", "F_10") && file.set_int(at, "TimeMarker", "frame", 10));
        file.remove_blocks(vec![at]);
        assert_eq!(file.blocks.len(), before);
        assert_eq!(file.index_at(address), None);
    }
}
