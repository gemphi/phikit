//! Object block extraction (assembling `BlendSceneObject`s).

use super::types::*;
use crate::phx::Result;

impl BlendFile {
    /// Simple object extraction - includes metadata (modifiers, instances) and mesh data
    pub(crate) fn extract_object_simple(
        &self,
        block: &BlendBlock,
    ) -> Result<Option<BlendSceneObject>> {
        let name = self.extract_name_from_block(block, "id.name")?;

        // Try to get object type, metadata, and mesh data from DNA if available
        let (
            loc,
            rot,
            quat,
            sca,
            mat,
            obj_type,
            modifiers,
            instance_targets,
            is_instance,
            vertices,
            faces,
            loop_uvs,
            loop_normals,
        ) = if let Some(ref dna) = self.dna {
            if let Some(obj_idx) = dna.structs.iter().position(|s| {
                (s.type_index as usize) < dna.types.len()
                    && dna.types[s.type_index as usize] == "Object"
            }) {
                // Extract transform fields from DNA structure
                let (l, r, q, s, m, t) = self.extract_object_transforms(block, dna, obj_idx);

                // Extract modifiers
                let mods = self.extract_modifiers(block, dna, obj_idx);

                // Extract instance info
                let (is_inst, inst_targets) = self.extract_instance_info(block, dna, obj_idx);

                // Extract mesh data including UVs and normals
                let (verts, fac, uvs, norms) = self.extract_mesh_data_full(block, dna);

                (
                    l,
                    r,
                    q,
                    s,
                    m,
                    t,
                    mods,
                    inst_targets,
                    is_inst,
                    verts,
                    fac,
                    uvs,
                    norms,
                )
            } else {
                (
                    [0.0, 0.0, 0.0],
                    [0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0, 0.0],
                    [1.0, 1.0, 1.0],
                    [
                        [1.0, 0.0, 0.0, 0.0],
                        [0.0, 1.0, 0.0, 0.0],
                        [0.0, 0.0, 1.0, 0.0],
                        [0.0, 0.0, 0.0, 1.0],
                    ],
                    "MESH".to_string(),
                    Vec::new(),
                    Vec::new(),
                    false,
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                )
            }
        } else {
            (
                [0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0, 0.0],
                [1.0, 1.0, 1.0],
                [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0],
                ],
                "MESH".to_string(),
                Vec::new(),
                Vec::new(),
                false,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            )
        };

        // Resolve the object's first material (diffuse colour + PBR scalars).
        let (base_color, roughness, metallic) = match &self.dna {
            Some(dna) => self.extract_object_material(block, dna),
            None => ([0.8, 0.8, 0.8, 1.0], 0.5, 0.0),
        };

        // Multi-material + texture path extraction.
        let (materials, texture_paths) = match &self.dna {
            Some(dna) => self.extract_all_materials(block, dna),
            None => (Vec::new(), Vec::new()),
        };

        Ok(Some(BlendSceneObject {
            name: if name.is_empty() {
                "Object".to_string()
            } else {
                name
            },
            object_type: obj_type,
            location: loc,
            rotation: rot,
            quaternion: quat,
            scale: sca,
            matrix_world: mat,
            vertices,
            faces,
            loop_normals,
            loop_uvs,
            modifiers,
            instance_targets,
            is_instance,
            base_color,
            roughness,
            metallic,
            materials,
            texture_paths,
        }))
    }

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
            vertices,
            faces,
            loop_normals: Vec::new(),
            loop_uvs: Vec::new(),
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
