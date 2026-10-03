//! Vertex-group weights (`CD_MDEFORMVERT` in the vertex `CustomData`, or the
//! pre-2.9 `Mesh.dvert` array): `MDeformVert { MDeformWeight *dw; int
//! totweight; int flag }`, `MDeformWeight { int def_nr; float weight }`.

use crate::blend::file::types::*;

const CD_MDEFORMVERT: i32 = 2;

impl BlendFile {
    /// Per-vertex `(group index, weight)` lists; empty when the mesh has none.
    pub(crate) fn deform_verts(&self, me: &BlendBlock, dna: &DnaType, verts: usize) -> Vec<Vec<(u32, f32)>> {
        let ps = self.header.pointer_size as usize;
        let (Some(size), Some(dw_o), Some(tot_o)) = (
            Self::struct_size("MDeformVert", dna, ps),
            self.off("MDeformVert", "*dw", dna),
            self.off("MDeformVert", "totweight", dna),
        ) else {
            return Vec::new();
        };
        self.scoped(me, || {
            let vd = self.off("Mesh", "vdata", dna).or_else(|| self.off("Mesh", "vert_data", dna));
            let layer = vd.and_then(|o| self.customdata_layer(me, o, dna, &|t, _| t == CD_MDEFORMVERT));
            let legacy = || self.block_at(self.ptr(me, self.off("Mesh", "*dvert", dna))).map(|b| b.data.as_slice());
            let Some(bytes) = layer.or_else(legacy) else { return Vec::new() };
            (0..verts)
                .map(|i| {
                    let Some(rec) = bytes.get(i * size..(i + 1) * size) else { return Vec::new() };
                    let n = self.read_i32(&rec[tot_o..tot_o + 4]).clamp(0, 256) as usize;
                    let dw = Self::read_ptr(rec, dw_o, ps, self.header.little_endian);
                    let Some(block) = self.block_at(dw).filter(|_| n > 0) else { return Vec::new() };
                    block
                        .data
                        .chunks_exact(8)
                        .take(n)
                        .map(|c| (self.read_i32(&c[0..4]).max(0) as u32, self.read_f32(&c[4..8])))
                        .collect()
                })
                .collect()
        })
    }
}
