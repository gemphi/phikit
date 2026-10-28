//! Face-domain attributes looked up **by exact name**: `sharp_face`
//! (`CD_PROP_BOOL`, Blender 3.5+; absent ⇒ every face smooth) and
//! `material_index` (`CD_PROP_INT32`). Blender ref: `DNA_customdata_types.h`,
//! `BKE_mesh_legacy_convert` (ME_SMOOTH → `sharp_face`).

use crate::blend::file::types::*;

/// Blender file version from which `sharp_face` replaces `ME_SMOOTH`.
const SHARP_FACE_SINCE: u32 = 305;
/// `CD_MPOLY` layer type and `ME_SMOOTH` poly flag (DNA_customdata_types.h, DNA_meshdata_types.h).
const CD_MPOLY: i32 = 25;
const ME_SMOOTH: u8 = 1;

impl BlendFile {
    /// Raw data bytes of the first `CustomData` layer accepted by `want(type, name)`.
    /// Iteration stops at the end of the written layer block (`totlayer` can
    /// over-count layers that were skipped on save).
    pub(crate) fn customdata_layer(
        &self,
        block: &BlendBlock,
        cd_offset: usize,
        dna: &DnaType,
        want: &dyn Fn(i32, &[u8]) -> bool,
    ) -> Option<&[u8]> {
        let ps = self.header.pointer_size as usize;
        let le = self.header.little_endian;
        let layers_addr = self.read_block_ptr(block, cd_offset, ps);
        let tot_off = self.customdata_totlayer_off(cd_offset, dna);
        let totlayer = block.data.get(tot_off..tot_off + 4).map(|b| self.read_i32(b))?.max(0) as usize;
        let layers = self.block_at(layers_addr)?;
        let size = Self::struct_size("CustomDataLayer", dna, ps)?;
        let name_off = Self::struct_field_offset_exact("CustomDataLayer", "name[68]", dna, ps)
            .or_else(|| Self::struct_field_offset_exact("CustomDataLayer", "name[64]", dna, ps))?;
        let data_off = Self::struct_field_offset_exact("CustomDataLayer", "*data", dna, ps)?;
        for i in 0..totlayer.min(1000) {
            let Some(rec) = layers.data.get(i * size..(i + 1) * size) else { break };
            let raw = &rec[name_off..];
            let n = raw.iter().position(|&b| b == 0).unwrap_or(raw.len()).min(68);
            if !want(self.read_i32(&rec[0..4]), &raw[..n]) {
                continue;
            }
            let addr = Self::read_ptr(rec, data_off, ps, le);
            return self.block_at(addr).map(|b| b.data.as_slice());
        }
        None
    }

    /// Raw data bytes of the `CustomData` layer named `name`, if present.
    pub(crate) fn read_customdata_named(
        &self,
        block: &BlendBlock,
        cd_offset: usize,
        dna: &DnaType,
        name: &str,
    ) -> Option<&[u8]> {
        self.customdata_layer(block, cd_offset, dna, &|_, n| n == name.as_bytes())
    }

    /// Legacy `MPoly` array (`CD_MPOLY`) that Blender ≤ 3.6 still writes for
    /// backward compatibility: `(mat_nr, flag)` per face.
    fn legacy_mpoly(&self, me: &BlendBlock, fd: usize, dna: &DnaType, faces: usize) -> Option<Vec<(u16, u8)>> {
        let ps = self.header.pointer_size as usize;
        let bytes = self.customdata_layer(me, fd, dna, &|t, _| t == CD_MPOLY)?;
        let size = Self::struct_size("MPoly", dna, ps)?;
        let mat = Self::struct_field_offset_exact("MPoly", "mat_nr_legacy", dna, ps)
            .or_else(|| Self::struct_field_offset_exact("MPoly", "mat_nr", dna, ps))?;
        let flag = Self::struct_field_offset_exact("MPoly", "flag_legacy", dna, ps)
            .or_else(|| Self::struct_field_offset_exact("MPoly", "flag", dna, ps))?;
        Some(
            (0..faces)
                .map(|i| {
                    let r = bytes.get(i * size..(i + 1) * size).unwrap_or(&[]);
                    let m = r.get(mat..mat + 2).map(|b| self.read_u16(b)).unwrap_or(0);
                    (m, r.get(flag).copied().unwrap_or(0))
                })
                .collect(),
        )
    }

    pub(crate) fn mesh_block_of(&self, ob: &BlendBlock, dna: &DnaType) -> Option<&BlendBlock> {
        let ps = self.header.pointer_size as usize;
        let off = Self::struct_field_offset_exact("Object", "*data", dna, ps)?;
        // Scoped to the object, for the same repeated-address reason as
        // every other pointer it owns (`file/scope.rs`).
        self.scoped(ob, || self.block_at_code(Self::read_ptr(&ob.data, off, ps, self.header.little_endian), "ME"))
    }

    fn face_data_offset(&self, dna: &DnaType) -> Option<usize> {
        let ps = self.header.pointer_size as usize;
        Self::struct_field_offset_exact("Mesh", "face_data", dna, ps)
            .or_else(|| Self::struct_field_offset_exact("Mesh", "pdata", dna, ps))
    }

    /// Per-face smooth flags for the mesh `ob` points at. Empty when unknown
    /// (legacy pre-3.5 files keep `ME_SMOOTH` in `MPoly`, not read here).
    pub(crate) fn extract_face_smooth(&self, ob: &BlendBlock, dna: &DnaType, faces: usize) -> Vec<bool> {
        let (Some(me), Some(fd)) = (self.mesh_block_of(ob, dna), self.face_data_offset(dna)) else {
            return Vec::new();
        };
        // Temporary compat arrays reuse addresses across meshes: resolve
        // within this mesh's data (`scope.rs`).
        self.scoped(me, || {
            let named = self.read_customdata_named(me, fd, dna, "sharp_face").map(std::borrow::Cow::Borrowed)
                .or_else(|| self.attr_bool(me, dna, "sharp_face", crate::blend::mesh::attrs::DOMAIN_FACE));
            if let Some(bytes) = named {
                return (0..faces).map(|i| bytes.get(i).is_none_or(|b| *b == 0)).collect();
            }
            if let Some(polys) = self.legacy_mpoly(me, fd, dna, faces) {
                return polys.iter().map(|(_, f)| f & ME_SMOOTH != 0).collect();
            }
            if self.header.version_number >= SHARP_FACE_SINCE { vec![true; faces] } else { Vec::new() }
        })
    }

    /// Per-face `material_index` (strict name match), empty when absent.
    pub(crate) fn extract_face_materials(&self, me: &BlendBlock, dna: &DnaType, faces: usize) -> Vec<u16> {
        let Some(fd) = self.face_data_offset(dna) else { return Vec::new() };
        if let Some(bytes) = self.read_customdata_named(me, fd, dna, "material_index") {
            return (0..faces)
                .map(|i| bytes.get(i * 4..i * 4 + 4).map(|b| self.read_i32(b).max(0) as u16).unwrap_or(0))
                .collect();
        }
        if let Some(v) = self.attr_int32(me, dna, "material_index", crate::blend::mesh::attrs::DOMAIN_FACE, faces) {
            return v.into_iter().map(|m| m.max(0) as u16).collect();
        }
        self.legacy_mpoly(me, fd, dna, faces)
            .map(|p| p.into_iter().map(|(m, _)| m).collect())
            .unwrap_or_default()
    }
}

