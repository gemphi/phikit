//! Legacy full-DNA object extraction (kept for the asset pipeline).

use crate::blend::file::types::*;
use crate::Result;

impl BlendFile {
    /// Extract object from an OB block using DNA structure (with vertices)
    #[allow(dead_code)]
    pub(crate) fn extract_object(&self, block: &BlendBlock) -> Result<Option<BlendSceneObject>> {
        let dna = match &self.dna {
            Some(d) => d,
            None => return Ok(None),
        };

        // Find Object structure in DNA
        let obj_struct_idx = dna.structs.iter().position(|s| {
            (s.type_index as usize) < dna.types.len()
                && dna.types[s.type_index as usize] == "Object"
        });

        if obj_struct_idx.is_none() {
            // Debug: print available types
            eprintln!(
                "[Blend Parser] Object struct not found in DNA. Available types: {:?}",
                dna.types.iter().take(20).collect::<Vec<_>>()
            );
        }

        // Extract name from ID - try multiple strategies
        let name = if block.data.len() >= 66 {
            // Strategy 1: First 66 bytes (name[66] in ID struct)
            let name_bytes = &block.data[0..66];
            let end = name_bytes.iter().position(|&b| b == 0).unwrap_or(66);
            let extracted = String::from_utf8_lossy(&name_bytes[0..end]).to_string();

            if !extracted.is_empty() {
                extracted
            } else {
                // Strategy 2: Look for non-null bytes anywhere in first 200 bytes
                let search_bytes = &block.data[0..block.data.len().min(200)];
                let mut found_name = "Unnamed".to_string();
                for i in 0..search_bytes.len().saturating_sub(3) {
                    // Look for pattern: non-null, non-null, null (short name)
                    if search_bytes[i] != 0 && search_bytes[i + 1] != 0 && search_bytes[i + 2] == 0
                    {
                        let name_end = i + 2;
                        let name_start = i.saturating_sub(62); // ID names are 66 bytes max
                        let candidate = &block.data[name_start..name_end];
                        let candidate_str = String::from_utf8_lossy(candidate).to_string();
                        if candidate_str.len() > 1
                            && candidate_str
                                .chars()
                                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
                        {
                            found_name = candidate_str;
                            break;
                        }
                    }
                }
                found_name
            }
        } else {
            "Unknown".to_string()
        };

        // Try to extract location/rotation/scale from Object struct
        // These are at known offsets in modern Blender versions
        let raw_xf = obj_struct_idx.map(|_| self.read_object_transform_raw(block, dna));
        let (location, rotation, quaternion, scale, matrix_world, obj_type) =
            if let Some(obj_idx) = obj_struct_idx {
                self.extract_object_transforms(block, dna, obj_idx)
            } else {
                // Fallback: try hardcoded offsets for common Blender versions
                let (loc, rot, sz, t) = self.extract_transforms_fallback(block);
                (
                    loc,
                    rot,
                    [1.0, 0.0, 0.0, 0.0],
                    sz,
                    [
                        [1.0, 0.0, 0.0, loc[0]],
                        [0.0, 1.0, 0.0, loc[1]],
                        [0.0, 0.0, 1.0, loc[2]],
                        [0.0, 0.0, 0.0, 1.0],
                    ],
                    t,
                )
            };

        // Try to find linked mesh data
        let (vertices, faces) = self.extract_mesh_data(block, dna);

        // Extract modifiers and instance info
        let modifiers = if let Some(obj_idx) = obj_struct_idx {
            self.extract_modifiers(block, dna, obj_idx)
        } else {
            Vec::new()
        };
        let (is_instance, instance_targets) = if let Some(obj_idx) = obj_struct_idx {
            self.extract_instance_info(block, dna, obj_idx)
        } else {
            (false, Vec::new())
        };

        Ok(Some(BlendSceneObject {
            name,
            object_type: obj_type,
            location,
            rotation,
            quaternion,
            scale,
            matrix_world,
            matrix_local: matrix_world,
            block_address: block.old_address,
            parent_address: raw_xf.map(|r| r.parent_address).unwrap_or(0),
            parentinv: raw_xf
                .map(|r| r.parentinv)
                .unwrap_or(crate::blend::object::transform::IDENTITY4),
            partype: raw_xf.map(|r| r.partype).unwrap_or(0),
            visibility: self.object_visibility(block, dna),
            color: self.object_color(block, dna),
            shadow: self.object_shadow(block, dna),
            vertices,
            faces: faces.into(),
            loop_normals: Vec::new(),
            loop_uvs: Vec::new(),
            loop_colors: Vec::new(),
            face_materials: Vec::new(),
            face_smooth: Vec::new(),
            data: Default::default(),
            modifiers,
            instance_targets,
            is_instance,
            base_color: [0.8, 0.8, 0.8, 1.0],
            roughness: 0.5,
            metallic: 0.0,
            materials: Vec::new(),
            texture_paths: Vec::new(),
        }))
    }
}
