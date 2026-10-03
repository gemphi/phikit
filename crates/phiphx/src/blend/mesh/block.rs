//! Vertex/face extraction from a single ME (Mesh) data block.

use crate::blend::file::types::*;

impl BlendFile {
    /// Extract vertices and faces from a ME (Mesh) block.
    ///
    /// The **primary path** follows the modern (Blender 4.x) layout: it reads
    /// the `verts_num` / `faces_num` / `corners_num` counts, the `position`
    /// float3 attribute (`read_positions`), the `.corner_vert` int attribute
    /// (`read_corner_verts`), and the `int *face_offset_indices` array
    /// (`read_face_offsets`), then assembles faces with `build_faces`. The
    /// **legacy path** (`read_mvert` / `read_faces` over `mvert` / `mpoly` /
    /// `mloop`) is only used as a pre-4.x fallback when the modern path yields
    /// nothing. Every reader is bounds-safe, so an unreadable block yields
    /// empty/partial results rather than a panic (Requirements 2.1, 2.3, 2.5).
    #[allow(dead_code)]
    pub(crate) fn extract_mesh_from_block(
        &self,
        block: &BlendBlock,
        dna: &DnaType,
    ) -> (Vec<[f32; 3]>, Vec<Vec<u32>>) {
        let mut vertices = Vec::new();
        let mut faces = Vec::new();

        let Some(mesh_idx) = dna.structs.iter().position(|s| {
            (s.type_index as usize) < dna.types.len() && dna.types[s.type_index as usize] == "Mesh"
        }) else {
            return (vertices, faces);
        };
        let ptr_size = self.header.pointer_size as usize;
        let off = self.mesh_field_offsets(&dna.structs[mesh_idx], dna, ptr_size);

        // --- Primary path: modern (Blender 4.x) attribute layout. ---
        // Counts prefer the explicit modern offsets, falling back to the
        // legacy aliases (`mesh_field_offsets` maps `verts_num`/`faces_num`/
        // `corners_num` onto `totvert`/`totpoly`/`totloop` when the legacy
        // fields are absent).
        let verts_num = self.read_count(block, off.verts_num.or(off.totvert));
        let faces_num = self.read_count(block, off.faces_num.or(off.totpoly));
        let corners_num = self.read_count(block, off.corners_num.or(off.totloop));

        vertices = self.read_positions(block, off.vert_data, verts_num, dna);
        let corner_verts = self.read_corner_verts(block, off.corner_data, corners_num, dna);
        let offsets =
            self.read_face_offsets(block, off.face_offset_indices_ptr, faces_num, ptr_size);
        if !vertices.is_empty() && !corner_verts.is_empty() && offsets.len() >= 2 {
            faces = build_faces(&offsets, &corner_verts, vertices.len());
        }
        // Blender 5.x: geometry lives in `attribute_storage` (`attr_storage.rs`).
        if vertices.is_empty() {
            vertices = self.attr_float3(block, dna, "position", verts_num);
            let cv = self.attr_int32(block, dna, ".corner_vert", crate::blend::mesh::attrs::DOMAIN_CORNER, corners_num);
            if let (false, Some(cv), true) = (vertices.is_empty(), cv, offsets.len() >= 2) {
                faces = build_faces(&offsets, &cv, vertices.len());
            }
        }

        // --- Legacy path: pre-4.x `mvert` / `mpoly` / `mloop` fallback. ---
        // Only engage when the modern path produced no geometry.
        if vertices.is_empty() {
            let vert_count = self.read_count(block, off.totvert);
            vertices = self.read_mvert(block, off.mvert_ptr, vert_count, ptr_size);

            // Blender 3.5+ fallback: read positions from CustomData vdata layers.
            if vertices.is_empty() && vert_count > 0 {
                if let Some(vdata_off) = off.vdata {
                    vertices =
                        self.read_customdata_float3(block, vdata_off, vert_count, dna, "position");
                }
            }

            // No byte-pattern guessing: Blender shows an empty mesh as empty
            // (the old heuristic scan invented hundreds of vertices).
        }

        if faces.is_empty() {
            let poly_count = self.read_count(block, off.totpoly);
            let loop_count = self.read_count(block, off.totloop);
            faces = self.read_faces(block, off.mpoly_ptr, off.mloop_ptr, poly_count, loop_count);

            // Blender 3.5+ fallback: read corner_verts from CustomData ldata.
            if faces.is_empty() && poly_count > 0 && loop_count > 0 {
                if let Some(ldata_off) = off.ldata {
                    let corner_verts =
                        self.read_customdata_int(block, ldata_off, loop_count, dna, ".corner_vert");
                    if !corner_verts.is_empty() {
                        // The face starts: `Mesh.poly_offset_indices` (3.6's own
                        // format, `faces_num + 1` ints), else the legacy `mpoly`
                        // pointer read as offsets. A 3.6-native mesh used to fall
                        // through both and be chunked into triangles.
                        let mut offsets = self.read_face_offsets(block, off.face_offset_indices_ptr, poly_count, ptr_size);
                        if offsets.len() < 2 {
                            offsets = self.read_poly_offsets_ptr(block, off.mpoly_ptr, poly_count, ptr_size);
                        }
                        if !offsets.is_empty() {
                            for p in 0..poly_count {
                                let start = offsets.get(p).copied().unwrap_or(0);
                                let end = offsets.get(p + 1).copied().unwrap_or(loop_count);
                                let count = end.saturating_sub(start);
                                if !(3..100).contains(&count) {
                                    continue;
                                }
                                let face: Vec<u32> = (start..end)
                                    .filter_map(|i| corner_verts.get(i).map(|&v| v as u32))
                                    .collect();
                                if face.len() >= 3 {
                                    faces.push(face);
                                }
                            }
                        } else {
                            // No poly offsets — assume all tris/quads by chunking
                            // the corner_verts array in groups of 3.
                            for chunk in corner_verts.chunks(3) {
                                if chunk.len() == 3 {
                                    faces.push(vec![
                                        chunk[0] as u32,
                                        chunk[1] as u32,
                                        chunk[2] as u32,
                                    ]);
                                }
                            }
                        }
                    }
                }
            }

            // Fallback: generate placeholder faces.
            if faces.is_empty() && !vertices.is_empty() && poly_count > 0 {
                faces = self.generate_tris(vertices.len(), poly_count.min(vertices.len() / 3));
            }
        }

        (vertices, faces)
    }
}

/// Build face corner lists from a modern (Blender 4.x) `face_offset_indices`
/// array plus a `.corner_vert` index array.
///
/// Face `f` spans corners `[offsets[f] .. offsets[f + 1])`. This function is
/// fully bounds-safe and never panics regardless of the input shapes
/// (Requirements 2.3, 2.6, 2.7):
///
/// - The face `end` is clamped to `corner_verts.len()`, so a truncated or
///   inconsistent offset array can never index past the corner array.
/// - Spans with fewer than 3 corners (after clamping) are skipped, so no
///   degenerate face is emitted.
/// - Any corner whose vertex index is negative or `>= verts_num` is dropped,
///   so no out-of-range index is ever emitted. If dropping such corners leaves
///   fewer than 3 valid corners, the whole face is skipped.
///
/// The offset array is expected to hold `faces_num + 1` entries, but this
/// function only relies on adjacent pairs `offsets[f]` / `offsets[f + 1]`, so a
/// shorter array simply yields fewer faces.
pub(crate) fn build_faces(
    offsets: &[usize],
    corner_verts: &[i32],
    verts_num: usize,
) -> Vec<Vec<u32>> {
    let mut faces = Vec::new();
    if offsets.len() < 2 {
        return faces;
    }
    let corner_len = corner_verts.len();
    // Iterate adjacent offset pairs: face f spans [offsets[f]..offsets[f+1]).
    for f in 0..offsets.len() - 1 {
        let start = offsets[f];
        // Clamp end to the corner array length so a bad/short offset never
        // slices past the corner_verts array.
        let end = offsets[f + 1].min(corner_len);
        // Skip spans that are inverted or too short before dropping corners.
        if start >= end || end - start < 3 {
            continue;
        }
        let mut face = Vec::with_capacity(end - start);
        for &idx in &corner_verts[start..end] {
            // Drop negative and out-of-range vertex indices (Req 2.7) so no
            // out-of-range index is ever emitted (Req 2.6).
            if idx < 0 {
                continue;
            }
            let v = idx as usize;
            if v >= verts_num {
                continue;
            }
            face.push(v as u32);
        }
        // Only keep faces that still have at least 3 valid corners.
        if face.len() >= 3 {
            faces.push(face);
        }
    }
    faces
}
