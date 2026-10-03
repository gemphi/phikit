//! Per-corner UVs: legacy `MLoopUV` and 3.5+ float2 `CustomData` layers.

use crate::blend::file::types::*;

impl BlendFile {
    /// Read per-loop UV coordinates from the Mesh's `*mloopuv` pointer.
    ///
    /// `MLoopUV` is a struct of `{ float uv[2]; int flag; }` = 12 bytes in
    /// Blender 2.x–3.4. There is one entry per loop (same count as `totloop`).
    /// Returns an empty vec if the pointer is absent or the data doesn't parse.
    pub(crate) fn read_loop_uvs(
        &self,
        block: &BlendBlock,
        mloopuv_ptr: Option<usize>,
        loop_count: usize,
    ) -> Vec<[f32; 2]> {
        let Some(ptr_off) = mloopuv_ptr else {
            return Vec::new();
        };
        let ps = self.header.pointer_size as usize;
        if ptr_off + ps > block.data.len() || loop_count == 0 {
            return Vec::new();
        }
        let addr = self.read_block_ptr(block, ptr_off, ps);
        if addr == 0 {
            return Vec::new();
        }
        let Some(uv_block) = self.block_at(addr) else {
            return Vec::new();
        };
        // MLoopUV stride: 12 bytes (float[2] + int flag) in most versions.
        let stride = if loop_count > 0 && uv_block.data.len() >= loop_count * 8 {
            uv_block.data.len() / loop_count
        } else {
            12
        };
        if stride < 8 {
            return Vec::new();
        }
        let mut uvs = Vec::with_capacity(loop_count);
        for i in 0..loop_count {
            let base = i * stride;
            if base + 8 > uv_block.data.len() {
                break;
            }
            let u = self.read_f32(&uv_block.data[base..base + 4]);
            let v = self.read_f32(&uv_block.data[base + 4..base + 8]);
            if u.is_finite() && v.is_finite() {
                uvs.push([u, v]);
            } else {
                uvs.push([0.0, 0.0]);
            }
        }
        uvs
    }

    /// Active UV map from corner `CustomData` (Blender
    /// `CustomData_get_active_layer_index`): the first layer of the UV type
    /// plus its `active` offset. 3.5+ stores `CD_PROP_FLOAT2` (49, 8 bytes per
    /// corner); ≤ 3.4 stores `CD_MLOOPUV` (16, `MLoopUV` = uv[2] + flag).
    pub(crate) fn read_customdata_uvs(
        &self,
        block: &BlendBlock,
        cd_offset: usize,
        count: usize,
        dna: &DnaType,
    ) -> Vec<[f32; 2]> {
        let ps = self.header.pointer_size as usize;
        let tot_off = self.customdata_totlayer_off(cd_offset, dna);
        let Some(total) = block.data.get(tot_off..tot_off + 4).map(|b| self.read_i32(b).clamp(0, 1000) as usize) else {
            return Vec::new();
        };
        let Some(layers) = self.block_at(self.read_block_ptr(block, cd_offset, ps)) else { return Vec::new() };
        let (Some(size), Some(active_o), Some(data_o)) = (
            Self::struct_size("CustomDataLayer", dna, ps),
            self.off("CustomDataLayer", "active", dna),
            self.off("CustomDataLayer", "*data", dna),
        ) else {
            return Vec::new();
        };
        let recs: Vec<&[u8]> = (0..total).map_while(|i| layers.data.get(i * size..(i + 1) * size)).collect();
        for (ty, stride) in [(CD_PROP_FLOAT2, 8), (CD_MLOOPUV, Self::struct_size("MLoopUV", dna, ps).unwrap_or(12))] {
            let Some(first) = recs.iter().position(|r| self.read_i32(&r[0..4]) == ty) else { continue };
            let active = self.read_i32(&recs[first][active_o..active_o + 4]).max(0) as usize;
            let rec = recs.get(first + active).filter(|r| self.read_i32(&r[0..4]) == ty).unwrap_or(&recs[first]);
            let Some(data) = self.block_at(Self::read_ptr(rec, data_o, ps, self.header.little_endian)) else { continue };
            return data
                .data
                .chunks_exact(stride.max(8))
                .take(count)
                .map(|c| [self.read_f32(&c[0..4]), self.read_f32(&c[4..8])])
                .collect();
        }
        Vec::new()
    }
}

const CD_MLOOPUV: i32 = 16;
const CD_PROP_FLOAT2: i32 = 49;
