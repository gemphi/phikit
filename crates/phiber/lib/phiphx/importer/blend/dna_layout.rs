//! DNA field/struct layout helpers: field sizes, offsets, pointer reads.

use super::types::*;

impl BlendFile {
    /// Get the size of a DNA field based on its type and array dimensions
    pub(crate) fn field_size(type_name: &str, name: &str, dna: &DnaType, ptr_size: usize) -> usize {
        if name.starts_with('*') || name.starts_with("**") || name.contains("(*") {
            return ptr_size;
        }

        let base_size = if let Some(idx) = dna.types.iter().position(|t| t == type_name) {
            dna.type_sizes[idx] as usize
        } else {
            0
        };

        let mut multiplier = 1;
        let mut in_bracket = false;
        let mut num_str = String::new();
        for c in name.chars() {
            if c == '[' {
                in_bracket = true;
                num_str.clear();
            } else if c == ']' {
                in_bracket = false;
                if let Ok(n) = num_str.parse::<usize>() {
                    multiplier *= n;
                }
            } else if in_bracket {
                num_str.push(c);
            }
        }

        base_size * multiplier
    }

    /// Calculate the byte offset of a field within a DNA structure
    pub(crate) fn struct_field_offset(
        struct_name: &str,
        field_name: &str,
        dna: &DnaType,
        ptr_size: usize,
    ) -> Option<usize> {
        let struct_idx = dna.types.iter().position(|t| t == struct_name)?;
        let s = dna
            .structs
            .iter()
            .find(|s| s.type_index as usize == struct_idx)?;

        let mut offset = 0;
        for f in &s.fields {
            let f_name = &dna.names[f.name_index as usize];
            if f_name.contains(field_name) {
                return Some(offset);
            }
            let f_type = &dna.types[f.type_index as usize];
            offset += Self::field_size(f_type, f_name, dna, ptr_size);
        }
        None
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
}
