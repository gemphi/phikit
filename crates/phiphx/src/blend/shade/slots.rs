//! Material slot resolution: `Object/Mesh.**mat` + `totcol` → `MA` blocks,
//! and the exact-name DNA field-offset lookup the material readers use.

use crate::blend::file::types::*;

impl BlendFile {
    /// Follow `<struct>.**mat` and return ALL material blocks (not just the first).
    pub(crate) fn all_material_blocks<'a>(
        &'a self,
        block: &BlendBlock,
        struct_name: &str,
        dna: &DnaType,
    ) -> Option<Vec<&'a BlendBlock>> {
        let ps = self.header.pointer_size as usize;
        let le = self.header.little_endian;
        let off = Self::struct_field_offset_exact(struct_name, "**mat", dna, ps)?;
        let arr_addr = Self::read_ptr(&block.data, off, ps, le);
        if arr_addr == 0 {
            return None;
        }
        let arr = self.block_at(arr_addr)?;
        // totcol tells us how many material pointers are in the array.
        let totcol_off = Self::struct_field_offset_exact(struct_name, "totcol", dna, ps);
        let count = totcol_off
            .and_then(|o| {
                (o + 2 <= block.data.len()).then(|| self.read_u16(&block.data[o..o + 2]) as usize)
            })
            .unwrap_or(1)
            .min(64); // sanity cap

        let mut result = Vec::new();
        for i in 0..count {
            let ptr_off = i * ps;
            if ptr_off + ps > arr.data.len() {
                break;
            }
            let mat_addr = Self::read_ptr(&arr.data, ptr_off, ps, le);
            if mat_addr == 0 {
                continue;
            }
            if let Some(ma) = self.block_at_code(mat_addr, "MA")
            {
                result.push(ma);
            }
        }
        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    /// Follow `<struct>.**mat` → first `Material*` → the `MA` block.
    pub(crate) fn first_material_block(
        &self,
        block: &BlendBlock,
        struct_name: &str,
        dna: &DnaType,
    ) -> Option<&BlendBlock> {
        let ps = self.header.pointer_size as usize;
        let le = self.header.little_endian;
        let off = Self::struct_field_offset_exact(struct_name, "**mat", dna, ps)?;
        let arr_addr = Self::read_ptr(&block.data, off, ps, le);
        if arr_addr == 0 {
            return None;
        }
        let arr = self.block_at(arr_addr)?;
        let mat_addr = Self::read_ptr(&arr.data, 0, ps, le);
        self.block_at_code(mat_addr, "MA")
    }

    /// Byte offset of a field by EXACT name (unlike `struct_field_offset`,
    /// which uses `contains` and would mis-match short names like `r`).
    /// A bare name also matches its fixed-array spelling (`idname` →
    /// `idname[64]`): DNA names carry the dimensions, and callers asking for
    /// `name` mean the char array. Answered from the layout worked out
    /// once per file (`file/sdna.rs`), as Blender's SDNA answers it.
    pub(crate) fn struct_field_offset_exact(
        struct_name: &str,
        field_name: &str,
        dna: &DnaType,
        ptr_size: usize,
    ) -> Option<usize> {
        dna.layout(ptr_size).offset(struct_name, field_name)
    }
}
