//! DNA field/struct layout helpers: field sizes, offsets, pointer reads.

use crate::blend::file::types::*;

impl BlendFile {
    /// Get the size of a DNA field based on its type and array dimensions
    /// (the layout's type table, not a walk of the type list).
    pub(crate) fn field_size(type_name: &str, name: &str, dna: &DnaType, ptr_size: usize) -> usize {
        dna.layout(ptr_size).field_size(type_name, name, dna)
    }

    /// Calculate the byte offset of a field within a DNA structure, by the
    /// first member whose name *contains* `field_name` (the exact lookup is
    /// `struct_field_offset_exact`).
    pub(crate) fn struct_field_offset(
        struct_name: &str,
        field_name: &str,
        dna: &DnaType,
        ptr_size: usize,
    ) -> Option<usize> {
        let layout = dna.layout(ptr_size);
        layout.structure(struct_name)?.order.iter().find(|(name, _)| name.contains(field_name)).map(|(_, off)| *off)
    }

    /// Read a pointer from block data
    pub(crate) fn read_ptr(
        data: &[u8],
        offset: usize,
        ptr_size: usize,
        little_endian: bool,
    ) -> u64 {
        if offset + ptr_size > data.len() {
            return 0;
        }
        if ptr_size == 8 {
            if little_endian {
                u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap())
            } else {
                u64::from_be_bytes(data[offset..offset + 8].try_into().unwrap())
            }
        } else {
            if little_endian {
                u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as u64
            } else {
                u32::from_be_bytes(data[offset..offset + 4].try_into().unwrap()) as u64
            }
        }
    }

    /// Byte offset of `CustomData.totlayer` inside a struct embedding a
    /// `CustomData` at `cd_offset`. `typemap[]` sits between `layers` and
    /// `totlayer`, so this must come from the file's DNA.
    pub(crate) fn customdata_totlayer_off(&self, cd_offset: usize, dna: &DnaType) -> usize {
        let ps = self.header.pointer_size as usize;
        cd_offset
            + Self::struct_field_offset_exact("CustomData", "totlayer", dna, ps).unwrap_or(ps)
    }
}
