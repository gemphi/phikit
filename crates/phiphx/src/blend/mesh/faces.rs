//! Face-domain readers: `CustomData` int layers (`material_index`,
//! `.corner_vert`) and `face_offset_indices`.

use crate::blend::file::types::*;

impl BlendFile {
    /// Read int data from a CustomData struct's layers (Blender 3.5+).
    /// Used for corner_verts (CD_PROP_INT32 = 7).
    pub(crate) fn read_customdata_int(
        &self,
        block: &BlendBlock,
        cd_offset: usize,
        count: usize,
        dna: &DnaType,
        attr_name: &str,
    ) -> Vec<i32> {
        let ps = self.header.pointer_size as usize;
        let layers_addr = self.read_block_ptr(block, cd_offset, ps);
        let totlayer_off = self.customdata_totlayer_off(cd_offset, dna);
        if totlayer_off + 4 > block.data.len() {
            return Vec::new();
        }
        let totlayer = self.read_i32(&block.data[totlayer_off..totlayer_off + 4]) as usize;
        if layers_addr == 0 || totlayer == 0 || totlayer > 1000 {
            return Vec::new();
        }

        let Some(layers_block) = self.block_at(layers_addr) else {
            return Vec::new();
        };
        let layer_size = Self::struct_size("CustomDataLayer", dna, ps).unwrap_or(0);
        if layer_size < 12 {
            return Vec::new();
        }
        let name_off = Self::struct_field_offset("CustomDataLayer", "name", dna, ps).unwrap_or(8);
        let data_off = Self::struct_field_offset("CustomDataLayer", "*data", dna, ps)
            .or_else(|| Self::struct_field_offset("CustomDataLayer", "data", dna, ps))
            .unwrap_or(ps + 4);

        for i in 0..totlayer.min(200) {
            let base = i * layer_size;
            if base + layer_size > layers_block.data.len() {
                break;
            }
            let layer_type = self.read_i32(&layers_block.data[base..base + 4]);
            let name_start = base + name_off;
            let name_end = (name_start + 64).min(layers_block.data.len());
            let name_slice = &layers_block.data[name_start..name_end];
            let nul = name_slice
                .iter()
                .position(|&b| b == 0)
                .unwrap_or(name_slice.len());
            let layer_name = String::from_utf8_lossy(&name_slice[..nul]);

            // CD_PROP_INT32 = 7; match by name or type
            let is_match =
                layer_name == attr_name || layer_name.contains(attr_name) || layer_type == 7;
            if !is_match {
                continue;
            }

            let data_ptr_pos = base + data_off;
            if data_ptr_pos + ps > layers_block.data.len() {
                continue;
            }
            let data_addr = Self::read_ptr(
                &layers_block.data,
                data_ptr_pos,
                ps,
                self.header.little_endian,
            );
            if data_addr == 0 {
                continue;
            }
            let Some(data_block) = self.block_at(data_addr) else {
                continue;
            };

            let mut values = Vec::with_capacity(count);
            for v in 0..count {
                let off = v * 4;
                if off + 4 > data_block.data.len() {
                    break;
                }
                values.push(self.read_i32(&data_block.data[off..off + 4]));
            }
            if !values.is_empty() {
                return values;
            }
        }
        Vec::new()
    }

    /// Read the modern `int *face_offset_indices` array (Blender 4.x).
    ///
    /// The array has `faces_num + 1` entries; face `f` spans corners
    /// `[offsets[f] .. offsets[f + 1])` and the final entry equals the total
    /// corner count. This reads *up to* `faces_num + 1` ints from the pointed
    /// block, clamping every `[start, end)` range to the block's actual byte
    /// length and stopping (rather than slicing past the end) once a read would
    /// overrun. The returned vector may therefore be shorter than
    /// `faces_num + 1` when the pointed block is short — never longer, never a
    /// panic (Requirements 2.1, 2.2, 2.3).
    pub(crate) fn read_face_offsets(
        &self,
        block: &BlendBlock,
        off: Option<usize>,
        faces_num: usize,
        ptr_size: usize,
    ) -> Vec<usize> {
        let Some(ptr_off) = off else {
            return Vec::new();
        };
        // The pointer itself must fit inside the mesh block.
        if ptr_off + ptr_size > block.data.len() || faces_num == 0 || faces_num >= 10_000_000 {
            return Vec::new();
        }
        let addr = self.read_block_ptr(block, ptr_off, ptr_size);
        if addr == 0 {
            return Vec::new();
        }
        let Some(data_block) = self.block_at(addr) else {
            return Vec::new();
        };
        let len = data_block.data.len();
        let want = faces_num + 1;
        let mut offsets = Vec::with_capacity(want);
        for i in 0..want {
            let start = i * 4;
            let end = start + 4;
            // Clamp to actual block length: skip (stop) rather than slice past.
            if start > len || end > len {
                break;
            }
            offsets.push(self.read_i32(&data_block.data[start..end]) as usize);
        }
        offsets
    }
}
