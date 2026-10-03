//! 4.x/5.x `position` and `.corner_vert` attributes (attribute storage or CustomData).

use crate::blend::file::types::*;

impl BlendFile {
    /// Read the `position` float3 attribute (Blender 4.x), matched by EXACT
    /// attribute name (`"position"`).
    ///
    /// Walks the CustomData `layers` array in `vert_data` looking for a layer
    /// whose name is exactly `"position"`, then decodes up to `verts_num`
    /// float3 positions from the layer's `*data` block. Every 12-byte vertex
    /// read is clamped to the data block's byte length; a read that would
    /// overrun stops the loop rather than slicing past the end (Requirements
    /// 2.1, 2.2, 2.3).
    pub(crate) fn read_positions(
        &self,
        block: &BlendBlock,
        off: Option<usize>,
        verts_num: usize,
        dna: &DnaType,
    ) -> Vec<[f32; 3]> {
        let Some(cd_offset) = off else {
            return Vec::new();
        };
        let ps = self.header.pointer_size as usize;
        let layers_ptr_off = cd_offset;
        let totlayer_off = self.customdata_totlayer_off(cd_offset, dna);
        if totlayer_off + 4 > block.data.len() || verts_num == 0 || verts_num >= 10_000_000 {
            return Vec::new();
        }
        let layers_addr = self.read_block_ptr(block, layers_ptr_off, ps);
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

        for i in 0..totlayer.min(1000) {
            let base = i * layer_size;
            if base + layer_size > layers_block.data.len() {
                break;
            }
            // EXACT name match — no "any float3 layer" / "contains" fallback.
            if self.layer_name_at(&layers_block.data, base + name_off) != "position" {
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
            let len = data_block.data.len();
            let mut verts = Vec::with_capacity(verts_num);
            for v in 0..verts_num {
                let start = v * 12;
                let end = start + 12;
                if start > len || end > len {
                    break;
                }
                let x = self.read_f32(&data_block.data[start..start + 4]);
                let y = self.read_f32(&data_block.data[start + 4..start + 8]);
                let z = self.read_f32(&data_block.data[start + 8..end]);
                verts.push([x, y, z]);
            }
            if !verts.is_empty() {
                return verts;
            }
        }
        Vec::new()
    }

    /// Read the `.corner_vert` int attribute (Blender 4.x), matched by EXACT
    /// attribute name (`".corner_vert"`).
    ///
    /// This replaces the old `layer_type == 7 || name.contains(..)` heuristic
    /// (root cause #2), which could pick the wrong INT32 layer (e.g.
    /// `.corner_edge` or `material_index`). Walks the CustomData `layers` array
    /// in `corner_data` for a layer whose name is exactly `".corner_vert"`,
    /// then decodes up to `corners_num` ints from the layer's `*data` block.
    /// Every 4-byte read is clamped to the data block's byte length; a read
    /// that would overrun stops the loop rather than slicing past the end
    /// (Requirements 2.1, 2.2, 2.3).
    pub(crate) fn read_corner_verts(
        &self,
        block: &BlendBlock,
        off: Option<usize>,
        corners_num: usize,
        dna: &DnaType,
    ) -> Vec<i32> {
        let Some(cd_offset) = off else {
            return Vec::new();
        };
        let ps = self.header.pointer_size as usize;
        let layers_ptr_off = cd_offset;
        let totlayer_off = self.customdata_totlayer_off(cd_offset, dna);
        if totlayer_off + 4 > block.data.len() || corners_num == 0 || corners_num >= 10_000_000 {
            return Vec::new();
        }
        let layers_addr = self.read_block_ptr(block, layers_ptr_off, ps);
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

        for i in 0..totlayer.min(1000) {
            let base = i * layer_size;
            if base + layer_size > layers_block.data.len() {
                break;
            }
            // EXACT name match — replaces the loose `layer_type == 7` heuristic.
            if self.layer_name_at(&layers_block.data, base + name_off) != ".corner_vert" {
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
            let len = data_block.data.len();
            let mut values = Vec::with_capacity(corners_num);
            for v in 0..corners_num {
                let start = v * 4;
                let end = start + 4;
                if start > len || end > len {
                    break;
                }
                values.push(self.read_i32(&data_block.data[start..end]));
            }
            if !values.is_empty() {
                return values;
            }
        }
        Vec::new()
    }
}
