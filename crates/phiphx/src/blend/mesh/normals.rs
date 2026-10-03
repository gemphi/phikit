//! Per-corner custom / split normals.

use crate::blend::mesh::offsets::MeshOffsets;
use crate::blend::file::types::*;

impl BlendFile {
    /// Read per-loop normals from the Mesh's custom split normal pointer or
    /// from a CustomData layer named ".corner_normal" (Blender 3.5+).
    ///
    /// Blender stores custom split normals as `short[4][3]` (quantized, in the
    /// `lnors_spacearr`) or as `float[3]` per loop. We try the float variant
    /// first (common in 3.5+ via CustomData), and fall back to decoding shorts.
    pub(crate) fn read_loop_normals(
        &self,
        block: &BlendBlock,
        off: &MeshOffsets,
        loop_count: usize,
        dna: &DnaType,
    ) -> Vec<[f32; 3]> {
        // Try CustomData ldata layer ".corner_normal" (Blender 4.x)
        if let Some(ldata_off) = off.ldata {
            let normals =
                self.read_customdata_float3(block, ldata_off, loop_count, dna, ".corner_normal");
            if normals.len() == loop_count {
                return normals;
            }
        }
        // Try the explicit *lnors pointer (rare, older files)
        let Some(ptr_off) = off.lnor_ptr else {
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
        let Some(nr_block) = self.block_at(addr) else {
            return Vec::new();
        };
        // float[3] per loop
        if nr_block.data.len() >= loop_count * 12 {
            let mut normals = Vec::with_capacity(loop_count);
            for i in 0..loop_count {
                let base = i * 12;
                let nx = self.read_f32(&nr_block.data[base..base + 4]);
                let ny = self.read_f32(&nr_block.data[base + 4..base + 8]);
                let nz = self.read_f32(&nr_block.data[base + 8..base + 12]);
                if nx.is_finite() && ny.is_finite() && nz.is_finite() {
                    normals.push([nx, ny, nz]);
                } else {
                    normals.push([0.0, 1.0, 0.0]);
                }
            }
            return normals;
        }
        Vec::new()
    }
}
