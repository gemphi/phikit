//! `CustomData` float3 layers (`position`, legacy vdata) and layer helpers.

use crate::blend::file::types::*;

impl BlendFile {
    /// Read float3 data from a CustomData struct's layers (Blender 3.5+).
    /// Walks the `CustomData.layers` array looking for a layer whose name
    /// matches `attr_name` (e.g. "position") and type is CD_PROP_FLOAT3 (18).
    /// Returns the decoded vec3 positions.
    pub(crate) fn read_customdata_float3(
        &self,
        block: &BlendBlock,
        cd_offset: usize,
        count: usize,
        dna: &DnaType,
        attr_name: &str,
    ) -> Vec<[f32; 3]> {
        let ps = self.header.pointer_size as usize;
        // CustomData has `*layers` pointer and `totlayer` int.
        let layers_ptr_off = cd_offset; // First field of CustomData is typically *layers
        let totlayer_off = self.customdata_totlayer_off(cd_offset, dna);
        if totlayer_off + 4 > block.data.len() {
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
        // Each CustomDataLayer is a struct. We need its size to iterate.
        // Find CustomDataLayer struct size from DNA.
        let layer_size = Self::struct_size("CustomDataLayer", dna, ps).unwrap_or(0);
        if layer_size < 12 {
            return Vec::new();
        }
        // Within CustomDataLayer: type (int, offset 0), name (char[68], varies),
        // *data pointer. Find them from DNA.
        let type_off_in_layer = 0usize; // `type` is always the first int field
        let name_off_in_layer =
            Self::struct_field_offset("CustomDataLayer", "name", dna, ps).unwrap_or(8);
        let data_off_in_layer = Self::struct_field_offset("CustomDataLayer", "*data", dna, ps)
            .or_else(|| Self::struct_field_offset("CustomDataLayer", "data", dna, ps))
            .unwrap_or(ps + 4); // fallback guess

        for i in 0..totlayer.min(200) {
            let base = i * layer_size;
            if base + layer_size > layers_block.data.len() {
                break;
            }
            let layer_type = self.read_i32(
                &layers_block.data[base + type_off_in_layer..base + type_off_in_layer + 4],
            );
            // CD_PROP_FLOAT3 = 18 (positions), also check by name
            let name_start = base + name_off_in_layer;
            let name_end = (name_start + 64).min(layers_block.data.len());
            let name_slice = &layers_block.data[name_start..name_end];
            let nul = name_slice
                .iter()
                .position(|&b| b == 0)
                .unwrap_or(name_slice.len());
            let layer_name = String::from_utf8_lossy(&name_slice[..nul]);

            let is_match = layer_name == attr_name || (layer_type == 18 && attr_name == "position");
            if !is_match {
                continue;
            }

            let data_ptr_pos = base + data_off_in_layer;
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

            // Every vertex, as Blender stores it: no range filtering and no
            // minimum count (edge-only widget meshes have 2 vertices).
            let verts: Vec<[f32; 3]> = data_block
                .data
                .chunks_exact(12)
                .take(count)
                .map(|c| [self.read_f32(&c[0..4]), self.read_f32(&c[4..8]), self.read_f32(&c[8..12])])
                .collect();
            if !verts.is_empty() {
                return verts;
            }
        }
        Vec::new()
    }

    /// Decode a NUL-terminated `CustomDataLayer.name` at `start` within a
    /// layers block, clamped to the block length. Returns an empty string when
    /// `start` is out of range so callers can compare by exact name safely.
    pub(crate) fn layer_name_at(&self, data: &[u8], start: usize) -> String {
        if start >= data.len() {
            return String::new();
        }
        let name_end = (start + 64).min(data.len());
        let name_slice = &data[start..name_end];
        let nul = name_slice
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(name_slice.len());
        String::from_utf8_lossy(&name_slice[..nul]).into_owned()
    }

    /// Get the total size of a DNA struct by name.
    pub(crate) fn struct_size(name: &str, dna: &DnaType, ptr_size: usize) -> Option<usize> {
        dna.layout(ptr_size).struct_size(name)
    }
}
