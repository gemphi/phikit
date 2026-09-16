//! Mesh data resolution, heuristic vertex scanning, and triangulation.

use super::types::*;

impl BlendFile {
    /// Extract mesh data from linked ME block or embedded data
    #[allow(dead_code)]
    pub(crate) fn extract_mesh_data(
        &self,
        obj_block: &BlendBlock,
        dna: &DnaType,
    ) -> (Vec<[f32; 3]>, Vec<Vec<u32>>) {
        let (verts, faces, _, _) = self.extract_mesh_data_full(obj_block, dna);
        (verts, faces)
    }

    /// Extract mesh data including per-loop UVs and normals.
    pub(crate) fn extract_mesh_data_full(
        &self,
        obj_block: &BlendBlock,
        dna: &DnaType,
    ) -> (Vec<[f32; 3]>, Vec<Vec<u32>>, Vec<[f32; 2]>, Vec<[f32; 3]>) {
        let ptr_size = self.header.pointer_size as usize;
        let little_endian = self.header.little_endian;

        // Helper: given a found ME block, extract verts+faces+UVs+normals.
        let extract_full = |me_block: &BlendBlock| -> Option<(
            Vec<[f32; 3]>,
            Vec<Vec<u32>>,
            Vec<[f32; 2]>,
            Vec<[f32; 3]>,
        )> {
            let (verts, faces) = self.extract_mesh_from_block(me_block, dna);
            if verts.is_empty() {
                return None;
            }
            // Read UVs and normals from the same ME block.
            let mesh_idx = dna.structs.iter().position(|s| {
                (s.type_index as usize) < dna.types.len()
                    && dna.types[s.type_index as usize] == "Mesh"
            });
            let (uvs, normals) = if let Some(mi) = mesh_idx {
                let off = self.mesh_field_offsets(&dna.structs[mi], dna, ptr_size);
                let loop_count = self.read_count(me_block, off.totloop);
                // UVs: try mloopuv pointer first (Blender 2.x–3.4), then CustomData ldata (3.5+)
                let uvs = {
                    let legacy = self.read_loop_uvs(me_block, off.mloopuv_ptr, loop_count);
                    if !legacy.is_empty() {
                        legacy
                    } else if let Some(ldata_off) = off.ldata {
                        self.read_customdata_uvs(me_block, ldata_off, loop_count, dna)
                    } else {
                        Vec::new()
                    }
                };
                // Normals: per-loop custom/smooth normals
                let normals = self.read_loop_normals(me_block, &off, loop_count, dna);
                (uvs, normals)
            } else {
                (Vec::new(), Vec::new())
            };
            Some((verts, faces, uvs, normals))
        };

        // === PREFERRED PATH ===
        // Each Object owns a `void *data;` pointer to its own ObData (Mesh)
        // block. Following this pointer per-object is the ONLY correct way to
        // resolve distinct geometry.
        if let Some(data_off) = Self::struct_field_offset_exact("Object", "*data", dna, ptr_size) {
            let data_addr = Self::read_ptr(&obj_block.data, data_off, ptr_size, little_endian);
            if data_addr != 0 {
                if let Some(me_block) = self
                    .blocks
                    .iter()
                    .find(|b| b.old_address == data_addr && b.code_str == "ME")
                {
                    if let Some(result) = extract_full(me_block) {
                        return result;
                    }
                }
                // The *data pointer might resolve to a non-ME block (e.g. an
                // Armature or Curve). Try without the code filter.
                if let Some(me_block) = self.blocks.iter().find(|b| b.old_address == data_addr) {
                    if me_block.data.len() > 200 {
                        if let Some(result) = extract_full(me_block) {
                            return result;
                        }
                    }
                }
            }
        }
        // Also try the contains-based offset as fallback for older DNA layouts.
        if let Some(data_off) = Self::struct_field_offset("Object", "*data", dna, ptr_size) {
            let data_addr = Self::read_ptr(&obj_block.data, data_off, ptr_size, little_endian);
            if data_addr != 0 {
                if let Some(me_block) = self
                    .blocks
                    .iter()
                    .find(|b| b.old_address == data_addr && b.code_str == "ME")
                {
                    if let Some(result) = extract_full(me_block) {
                        return result;
                    }
                }
            }
        }

        // === FALLBACK (legacy / safety net) ===
        let obj_name = if obj_block.data.len() >= 64 {
            let name_bytes = &obj_block.data[0..64];
            let end = name_bytes.iter().position(|&b| b == 0).unwrap_or(64);
            String::from_utf8_lossy(&name_bytes[0..end]).to_string()
        } else {
            String::new()
        };
        if obj_name.is_empty() {
            return (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        }
        for block in &self.blocks {
            if block.code_str != "ME" {
                continue;
            }
            let mesh_name = if block.data.len() >= 64 {
                let name_bytes = &block.data[0..64];
                let end = name_bytes.iter().position(|&b| b == 0).unwrap_or(64);
                String::from_utf8_lossy(&name_bytes[0..end]).to_string()
            } else {
                String::new()
            };
            if mesh_name == obj_name {
                if let Some(result) = extract_full(block) {
                    return result;
                }
            }
        }

        (Vec::new(), Vec::new(), Vec::new(), Vec::new())
    }

    /// Scan for vertex data by looking for float patterns
    #[allow(dead_code)]
    pub(crate) fn scan_for_vertices(&self, data: &[u8]) -> Vec<[f32; 3]> {
        let mut vertices = Vec::new();

        // Look for sequences of 3 floats that could be vertex positions
        // Skip the first 152 bytes (ID structure)
        let start = data.len().min(152);
        let end = data.len().saturating_sub(12);

        for i in (start..end).step_by(4) {
            // Read 3 floats
            let x = self.read_f32(&data[i..i + 4]);
            let y = self.read_f32(&data[i + 4..i + 8]);
            let z = self.read_f32(&data[i + 8..i + 12]);

            // Check if they look like valid positions (not NaN, not infinity, reasonable range)
            if x.is_finite()
                && y.is_finite()
                && z.is_finite()
                && x.abs() < 10000.0
                && y.abs() < 10000.0
                && z.abs() < 10000.0
            {
                vertices.push([x, y, z]);

                // Limit to avoid reading garbage
                if vertices.len() >= 100000 {
                    break;
                }
            }
        }

        vertices
    }

    /// Generate simple triangulation for faces
    #[allow(dead_code)]
    pub(crate) fn generate_tris(&self, vert_count: usize, face_count: usize) -> Vec<Vec<u32>> {
        let mut faces = Vec::new();

        // Simple fan triangulation for placeholder
        // Real implementation would read actual MFace data
        for f in 0..face_count.min(vert_count / 3) {
            let base = (f * 3) % vert_count;
            faces.push(vec![
                base as u32,
                ((base + 1) % vert_count) as u32,
                ((base + 2) % vert_count) as u32,
            ]);
        }

        faces
    }
}
