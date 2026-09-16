//! Object metadata extraction: modifiers and instance info.

use super::types::*;

impl BlendFile {
    /// Extract modifiers from Object struct (if modifiers field exists)
    #[allow(dead_code)]
    pub(crate) fn extract_modifiers(
        &self,
        block: &BlendBlock,
        dna: &DnaType,
        obj_idx: usize,
    ) -> Vec<String> {
        let mut modifiers = Vec::new();

        // Look for modifier-related fields in the Object struct
        // In Blender DNA, the "modifiers" field is a ListBase containing modifier pointers
        let obj_struct = &dna.structs[obj_idx];
        let mut offset = 0usize;

        for field in &obj_struct.fields {
            if (field.type_index as usize) >= dna.types.len()
                || (field.name_index as usize) >= dna.names.len()
            {
                // Calculate proper field size for offset
                let type_size = if (field.type_index as usize) < dna.type_sizes.len() {
                    dna.type_sizes[field.type_index as usize] as usize
                } else {
                    8 // assume pointer
                };
                offset += type_size;
                continue;
            }

            let field_name = &dna.names[field.name_index as usize];
            let type_size = if (field.type_index as usize) < dna.type_sizes.len() {
                dna.type_sizes[field.type_index as usize] as usize
            } else {
                8 // assume pointer
            };

            // Check for the specific "modifiers" field (ListBase)
            if field_name == "modifiers" || field_name == "modifiers.first" {
                // Try to read the pointer at this offset to check if modifiers exist
                if offset + 8 <= block.data.len() {
                    let ptr = self.read_u64(&block.data[offset..offset + 8]);
                    if ptr != 0 {
                        modifiers.push(format!("modifiers (active)"));
                    }
                }
            }

            offset += type_size;
        }

        modifiers
    }

    /// Extract instance information from object block
    #[allow(dead_code)]
    pub(crate) fn extract_instance_info(
        &self,
        block: &BlendBlock,
        dna: &DnaType,
        obj_idx: usize,
    ) -> (bool, Vec<String>) {
        let mut is_instance = false;
        let mut targets = Vec::new();

        // Look for instance-related fields
        let obj_struct = &dna.structs[obj_idx];
        let mut offset = 0usize;

        for field in &obj_struct.fields {
            if (field.type_index as usize) >= dna.types.len()
                || (field.name_index as usize) >= dna.names.len()
            {
                let type_size = if (field.type_index as usize) < dna.type_sizes.len() {
                    dna.type_sizes[field.type_index as usize] as usize
                } else {
                    8
                };
                offset += type_size;
                continue;
            }

            let field_name = &dna.names[field.name_index as usize];
            let type_size = if (field.type_index as usize) < dna.type_sizes.len() {
                dna.type_sizes[field.type_index as usize] as usize
            } else {
                8
            };

            // Detect specific instance fields: dup_group or instance_collection
            if field_name == "dup_group" || field_name == "instance_collection" {
                // Check if pointer is non-null (indicating actual instance)
                if offset + 8 <= block.data.len() {
                    let ptr = self.read_u64(&block.data[offset..offset + 8]);
                    if ptr != 0 {
                        is_instance = true;
                        targets.push(format!("{} (active)", field_name));
                    }
                }
            }

            offset += type_size;
        }

        (is_instance, targets)
    }
}
