//! Object transform extraction (DNA-driven and fallback offsets).

use super::types::*;

impl BlendFile {
    /// Extract object transforms using DNA structure info
    pub(crate) fn extract_object_transforms(
        &self,
        block: &BlendBlock,
        dna: &DnaType,
        obj_idx: usize,
    ) -> (
        [f32; 3],
        [f32; 3],
        [f32; 4],
        [f32; 3],
        [[f32; 4]; 4],
        String,
    ) {
        let mut location = [0.0f32, 0.0, 0.0];
        let mut rotation = [0.0f32, 0.0, 0.0];
        let mut quaternion = [1.0f32, 0.0, 0.0, 0.0]; // w,x,y,z
        let mut scale = [1.0f32, 1.0, 1.0];
        let mut matrix_world = [
            [1.0f32, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let mut obj_type = "MESH".to_string();

        // Calculate field offsets by summing sizes of preceding fields
        let ptr_size = self.header.pointer_size as usize;
        let mut offset = 0usize;

        for field in &dna.structs[obj_idx].fields {
            if (field.type_index as usize) >= dna.types.len()
                || (field.name_index as usize) >= dna.names.len()
            {
                continue;
            }

            let _type_name = &dna.types[field.type_index as usize];
            let field_name = &dna.names[field.name_index as usize];

            // Pointer fields (names starting with * or ** or containing (*)
            // are always ptr_size bytes regardless of the pointed-to type's
            // declared size. Without this check, a Material* (1680 bytes)
            // would advance the offset by 1680 instead of 8, making every
            // subsequent field (loc, rot, obmat) read from a wrong position.
            let is_pointer = field_name.starts_with('*')
                || field_name.starts_with("**")
                || field_name.contains("(*");

            let base_size = if is_pointer {
                ptr_size
            } else if (field.type_index as usize) < dna.type_sizes.len() {
                dna.type_sizes[field.type_index as usize] as usize
            } else {
                4
            };

            // Check for array notation like "loc[3]"
            let array_count = if !is_pointer && field_name.contains('[') {
                field_name
                    .split('[')
                    .nth(1)
                    .and_then(|s| s.split(']').next())
                    .and_then(|n| n.parse::<usize>().ok())
                    .unwrap_or(1)
            } else {
                1
            };

            let field_size = base_size * array_count;

            // Extract transform fields
            match field_name.as_str() {
                "loc" | "loc[3]" => {
                    if offset + 12 <= block.data.len() {
                        location[0] = self.read_f32(&block.data[offset..offset + 4]);
                        location[1] = self.read_f32(&block.data[offset + 4..offset + 8]);
                        location[2] = self.read_f32(&block.data[offset + 8..offset + 12]);
                    }
                }
                "rot" | "rot[3]" => {
                    if offset + 12 <= block.data.len() {
                        rotation[0] = self.read_f32(&block.data[offset..offset + 4]);
                        rotation[1] = self.read_f32(&block.data[offset + 4..offset + 8]);
                        rotation[2] = self.read_f32(&block.data[offset + 8..offset + 12]);
                    }
                }
                "quat" | "quat[4]" => {
                    if offset + 16 <= block.data.len() {
                        quaternion[0] = self.read_f32(&block.data[offset..offset + 4]);
                        quaternion[1] = self.read_f32(&block.data[offset + 4..offset + 8]);
                        quaternion[2] = self.read_f32(&block.data[offset + 8..offset + 12]);
                        quaternion[3] = self.read_f32(&block.data[offset + 12..offset + 16]);
                    }
                }
                "size" | "size[3]" => {
                    if offset + 12 <= block.data.len() {
                        scale[0] = self.read_f32(&block.data[offset..offset + 4]);
                        scale[1] = self.read_f32(&block.data[offset + 4..offset + 8]);
                        scale[2] = self.read_f32(&block.data[offset + 8..offset + 12]);
                    }
                }
                "obmat" | "obmat[4][4]" => {
                    if offset + 64 <= block.data.len() {
                        for row in 0..4 {
                            for col in 0..4 {
                                matrix_world[row][col] = self.read_f32(
                                    &block.data[offset + (row * 4 + col) * 4
                                        ..offset + (row * 4 + col) * 4 + 4],
                                );
                            }
                        }
                    }
                }
                "type" => {
                    if offset + 2 <= block.data.len() {
                        let type_val = self.read_u16(&block.data[offset..offset + 2]);
                        // Blender object type values (OB_TYPE_* in DNA)
                        obj_type = match type_val {
                            0 => "EMPTY".to_string(),
                            1 => "MESH".to_string(),
                            2 => "CURVE".to_string(),
                            3 => "SURFACE".to_string(),
                            4 => "FONT".to_string(),
                            5 => "META".to_string(),
                            8 => "GPENCIL".to_string(),
                            9 => "LATTICE".to_string(),
                            10 => "ARMATURE".to_string(),
                            11 => "EMPTY".to_string(), // OB_EMPTY
                            12 => "CAMERA".to_string(),
                            13 => "LIGHT".to_string(), // OB_LAMP
                            14 => "SPEAKER".to_string(),
                            15 => "LIGHTPROBE".to_string(),
                            _ => format!("TYPE_{}", type_val),
                        };
                    }
                }
                _ => {}
            }

            offset += field_size;
        }

        (
            location,
            rotation,
            quaternion,
            scale,
            matrix_world,
            obj_type,
        )
    }

    /// Fallback transform extraction for common Blender versions
    #[allow(dead_code)]
    pub(crate) fn extract_transforms_fallback(
        &self,
        block: &BlendBlock,
    ) -> ([f32; 3], [f32; 3], [f32; 3], String) {
        let mut location = [0.0f32, 0.0, 0.0];
        let mut rotation = [0.0f32, 0.0, 0.0];
        let mut scale = [1.0f32, 1.0, 1.0];

        // Common offset patterns in Blender 2.8-4.x
        // ID header is typically 152 bytes, then Object fields start
        let id_size = if block.data.len() > 200 { 152 } else { 64 };

        // Try to read loc at offset 152 (after ID)
        if block.data.len() >= id_size + 12 {
            location[0] = self.read_f32(&block.data[id_size..id_size + 4]);
            location[1] = self.read_f32(&block.data[id_size + 4..id_size + 8]);
            location[2] = self.read_f32(&block.data[id_size + 8..id_size + 12]);
        }

        // Rotation typically follows location
        if block.data.len() >= id_size + 24 {
            rotation[0] = self.read_f32(&block.data[id_size + 12..id_size + 16]);
            rotation[1] = self.read_f32(&block.data[id_size + 16..id_size + 20]);
            rotation[2] = self.read_f32(&block.data[id_size + 20..id_size + 24]);
        }

        // Scale follows rotation
        if block.data.len() >= id_size + 36 {
            scale[0] = self.read_f32(&block.data[id_size + 24..id_size + 28]);
            scale[1] = self.read_f32(&block.data[id_size + 28..id_size + 32]);
            scale[2] = self.read_f32(&block.data[id_size + 32..id_size + 36]);
        }

        (location, rotation, scale, "MESH".to_string())
    }
}
