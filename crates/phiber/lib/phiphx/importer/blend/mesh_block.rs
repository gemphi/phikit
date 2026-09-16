//! Vertex/face extraction from a single ME (Mesh) data block.

use super::types::*;

/// Byte offsets (within a Mesh block) of the geometry pointers and counts.
///
/// Two layouts are supported. The **modern (Blender 4.x)** layout — grounded in
/// `REFS/blender/blender/source/blender/makesdna/DNA_mesh_types.h` — stores
/// geometry as four consecutive `int` counts (`verts_num`, `edges_num`,
/// `faces_num`, `corners_num`), an `int *face_offset_indices` array of length
/// `faces_num + 1`, an `AttributeStorage attribute_storage`, and the CustomData
/// members `vert_data` / `edge_data` / `face_data` / `corner_data`. Vertex
/// positions live in the `position` float3 attribute (via `attribute_storage`
/// or `vert_data`), and corner→vertex indices live in the `.corner_vert` int
/// attribute (via `corner_data`). The **legacy (pre-4.x)** `mvert` / `mpoly` /
/// `mloop` pointers and their `tot*` counts are all `DNA_DEPRECATED` and are
/// retained here only for reading old files.
#[derive(Default)]
pub(crate) struct MeshOffsets {
    // --- Modern (Blender 4.x) layout — primary path ---
    /// Offset of the `faces_num` int count.
    pub faces_num: Option<usize>,
    /// Offset of the `corners_num` int count.
    pub corners_num: Option<usize>,
    /// Offset of the `verts_num` int count.
    pub verts_num: Option<usize>,
    /// Offset of the `int *face_offset_indices` pointer (array len = faces_num + 1).
    pub face_offset_indices_ptr: Option<usize>,
    /// Offset of the `AttributeStorage attribute_storage` member.
    pub attribute_storage: Option<usize>,
    /// Offset of the `CustomData vert_data` member (holds the `position` attribute).
    pub vert_data: Option<usize>,
    /// Offset of the `CustomData corner_data` member (holds the `.corner_vert` attribute).
    pub corner_data: Option<usize>,

    // --- Legacy (pre-4.x) layout — retained for reading old files ---
    pub mvert_ptr: Option<usize>,
    pub totvert: Option<usize>,
    pub mpoly_ptr: Option<usize>,
    pub totpoly: Option<usize>,
    pub mloop_ptr: Option<usize>,
    pub totloop: Option<usize>,
    /// Pointer to `MLoopUV *mloopuv` (Blender 2.x–3.4, per-loop UV).
    pub mloopuv_ptr: Option<usize>,
    /// Pointer to per-loop normals (`float (*lnors)[3]` or custom split normals).
    pub lnor_ptr: Option<usize>,
    /// Offset of `CustomData vdata` within the Mesh struct (for Blender 3.5+).
    pub vdata: Option<usize>,
    /// Offset of `CustomData ldata` (loop/corner data) within the Mesh struct.
    pub ldata: Option<usize>,
    /// Offset of `CustomData pdata` (poly/face data) within the Mesh struct.
    pub pdata: Option<usize>,
}

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

            // Fallback: heuristic scan if pointer chase failed.
            if vertices.is_empty() && block.data.len() > 200 {
                vertices = self.scan_for_vertices(&block.data);
            }
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
                        // Read poly offsets (face start indices) from pdata or as
                        // a flat int array pointed to by *poly_offsets.
                        let poly_offsets = if let Some(pdata_off) = off.pdata {
                            self.read_customdata_int(
                                block,
                                pdata_off,
                                poly_count + 1,
                                dna,
                                ".corner_vert",
                            )
                        } else {
                            Vec::new()
                        };
                        // Try reading poly_offsets from the pointer field directly.
                        let offsets = if poly_offsets.is_empty() {
                            self.read_poly_offsets_ptr(block, off.mpoly_ptr, poly_count, ptr_size)
                        } else {
                            poly_offsets.iter().map(|&v| v as usize).collect()
                        };
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

    /// Walk the Mesh struct fields to locate the geometry pointers/counts.
    pub(crate) fn mesh_field_offsets(
        &self,
        mesh_struct: &DnaStruct,
        dna: &DnaType,
        ptr_size: usize,
    ) -> MeshOffsets {
        let mut o = MeshOffsets::default();
        let mut offset = 0usize;
        for field in &mesh_struct.fields {
            if (field.type_index as usize) >= dna.types.len()
                || (field.name_index as usize) >= dna.names.len()
            {
                offset += ptr_size;
                continue;
            }
            let type_name = &dna.types[field.type_index as usize];
            let field_name = &dna.names[field.name_index as usize];
            match field_name.as_str() {
                // --- Modern (Blender 4.x) layout, per DNA_mesh_types.h ---
                // Counts: `int verts_num/edges_num/faces_num/corners_num`.
                "verts_num" => {
                    o.verts_num = Some(offset);
                    if o.totvert.is_none() {
                        o.totvert = Some(offset);
                    }
                }
                "faces_num" => {
                    o.faces_num = Some(offset);
                    if o.totpoly.is_none() {
                        o.totpoly = Some(offset);
                    }
                }
                "corners_num" => {
                    o.corners_num = Some(offset);
                    if o.totloop.is_none() {
                        o.totloop = Some(offset);
                    }
                }
                // `int *face_offset_indices` — array of length faces_num + 1.
                "*face_offset_indices" => {
                    o.face_offset_indices_ptr = Some(offset);
                }
                // `AttributeStorage attribute_storage` — generic attribute store.
                "attribute_storage" => {
                    o.attribute_storage = Some(offset);
                }
                // `CustomData vert_data` — holds the `position` float3 attribute.
                "vert_data" => {
                    o.vert_data = Some(offset);
                }
                // `CustomData corner_data` — holds the `.corner_vert` int attribute.
                "corner_data" => {
                    o.corner_data = Some(offset);
                }
                // Traditional Blender 2.x/3.0–3.4 fields (legacy fallback)
                "*mvert" | "mvert" => o.mvert_ptr = Some(offset),
                "totvert" => o.totvert = Some(offset),
                "*mpoly" | "mpoly" => o.mpoly_ptr = Some(offset),
                "totpoly" => o.totpoly = Some(offset),
                "*mloop" | "mloop" => o.mloop_ptr = Some(offset),
                "totloop" => o.totloop = Some(offset),
                // Blender 3.5+ renamed some fields (compatibility)
                "*vert_positions" => {
                    if o.mvert_ptr.is_none() {
                        o.mvert_ptr = Some(offset);
                    }
                }
                "*poly_offsets" => {
                    if o.mpoly_ptr.is_none() {
                        o.mpoly_ptr = Some(offset);
                    }
                }
                "*corner_verts" => {
                    if o.mloop_ptr.is_none() {
                        o.mloop_ptr = Some(offset);
                    }
                }
                "polys_num" => {
                    if o.totpoly.is_none() {
                        o.totpoly = Some(offset);
                    }
                }
                "loops_num" => {
                    if o.totloop.is_none() {
                        o.totloop = Some(offset);
                    }
                }
                // CustomData structs for Blender 3.5+ attribute-based mesh
                "vdata" => {
                    o.vdata = Some(offset);
                }
                "ldata" => {
                    o.ldata = Some(offset);
                }
                "pdata" => {
                    o.pdata = Some(offset);
                }
                // UV and normal pointers (Blender 2.x–3.4)
                "*mloopuv" => {
                    o.mloopuv_ptr = Some(offset);
                }
                "*lnors" | "*poly_normals" => {
                    o.lnor_ptr = Some(offset);
                }
                _ => {}
            }
            if field_name.starts_with('*') {
                offset += ptr_size;
            } else {
                offset += Self::field_size(type_name, field_name, dna, ptr_size);
            }
        }
        o
    }

    /// Read a 4-byte count at `off` (0 if absent / out of range).
    pub(crate) fn read_count(&self, block: &BlendBlock, off: Option<usize>) -> usize {
        match off {
            Some(o) if o + 4 <= block.data.len() => self.read_i32(&block.data[o..o + 4]) as usize,
            _ => 0,
        }
    }

    /// Read a pointer value of `ptr_size` bytes at `off`. Offsets past the
    /// block end (newer SDNA layouts expose fields this struct lacks, or older
    /// blocks are smaller than the embedded DNA) read as null.
    pub(crate) fn read_block_ptr(&self, block: &BlendBlock, off: usize, ptr_size: usize) -> u64 {
        let width = ptr_size.max(4);
        if off + width > block.data.len() {
            return 0;
        }
        if ptr_size == 8 {
            u64::from_le_bytes(block.data[off..off + 8].try_into().unwrap_or([0; 8]))
        } else {
            u32::from_le_bytes(block.data[off..off + 4].try_into().unwrap_or([0; 4])) as u64
        }
    }

    /// Follow the `*mvert` pointer and decode the vertex positions.
    pub(crate) fn read_mvert(
        &self,
        block: &BlendBlock,
        mvert_ptr: Option<usize>,
        vert_count: usize,
        ptr_size: usize,
    ) -> Vec<[f32; 3]> {
        let mut vertices = Vec::new();
        let Some(ptr_off) = mvert_ptr else {
            return vertices;
        };
        if ptr_off + ptr_size > block.data.len() || vert_count == 0 || vert_count >= 10_000_000 {
            return vertices;
        }
        let addr = self.read_block_ptr(block, ptr_off, ptr_size);
        if addr == 0 {
            return vertices;
        }
        let Some(vb) = self.blocks.iter().find(|b| b.old_address == addr) else {
            return vertices;
        };
        // MVert is 20 bytes in Blender 2.x; newer files use other strides.
        let mut best = 0usize;
        for &s in &[20usize, 16, 12, 24, 28, 32] {
            if s * vert_count <= vb.data.len() + s {
                best = s;
                break;
            }
        }
        if best == 0 {
            best = vb.data.len() / vert_count;
        }
        if best < 12 {
            return vertices;
        }
        for i in 0..vert_count {
            let base = i * best;
            if base + 12 > vb.data.len() {
                break;
            }
            let x = self.read_f32(&vb.data[base..base + 4]);
            let y = self.read_f32(&vb.data[base + 4..base + 8]);
            let z = self.read_f32(&vb.data[base + 8..base + 12]);
            if x.is_finite()
                && y.is_finite()
                && z.is_finite()
                && x.abs() < 100_000.0
                && y.abs() < 100_000.0
                && z.abs() < 100_000.0
            {
                vertices.push([x, y, z]);
            }
        }
        if vertices.len() < 3 {
            vertices.clear();
        }
        vertices
    }

    /// Follow `*mpoly`/`*mloop` pointers and decode polygon vertex indices.
    pub(crate) fn read_faces(
        &self,
        block: &BlendBlock,
        mpoly_ptr: Option<usize>,
        mloop_ptr: Option<usize>,
        poly_count: usize,
        loop_count: usize,
    ) -> Vec<Vec<u32>> {
        let mut faces = Vec::new();
        let (Some(poly_off), Some(loop_off)) = (mpoly_ptr, mloop_ptr) else {
            return faces;
        };
        if poly_count == 0 || poly_count >= 10_000_000 {
            return faces;
        }
        let ps = self.header.pointer_size as usize;
        let poly_addr = self.read_block_ptr(block, poly_off, ps);
        let loop_addr = self.read_block_ptr(block, loop_off, ps);
        let (Some(pb), Some(lb)) = (
            self.blocks.iter().find(|b| b.old_address == poly_addr),
            self.blocks.iter().find(|b| b.old_address == loop_addr),
        ) else {
            return faces;
        };
        let plen = pb.data.len();
        let llen = lb.data.len();
        let mpoly_size = if poly_count > 0 {
            plen / poly_count
        } else {
            12
        };
        let mloop_size = if loop_count > 0 { llen / loop_count } else { 8 };
        // A zero stride would make every `pbase`/`lbase` collapse to 0 and spin
        // without progress; bail rather than emit garbage.
        if mpoly_size == 0 || mloop_size == 0 {
            return faces;
        }
        for p in 0..poly_count {
            let pbase = p * mpoly_size;
            // Overflow-safe end check: compute the exclusive end with
            // `checked_add` so a corrupt/huge stride can never wrap past the
            // guard and slice out of range (observed panic at pbase..pbase+4).
            match pbase.checked_add(8) {
                Some(end) if end <= plen => {}
                _ => break,
            }
            let loop_start = self.read_i32(&pb.data[pbase..pbase + 4]) as usize;
            let loop_tot = self.read_i32(&pb.data[pbase + 4..pbase + 8]) as usize;
            if !(3..100).contains(&loop_tot) {
                continue;
            }
            let mut face = Vec::with_capacity(loop_tot);
            for l in 0..loop_tot {
                // Overflow-safe: `loop_start + l` and the `* mloop_size` product
                // both come from file data and can be enormous.
                let Some(base) = loop_start
                    .checked_add(l)
                    .and_then(|i| i.checked_mul(mloop_size))
                else {
                    break;
                };
                match base.checked_add(4) {
                    Some(end) if end <= llen => {}
                    _ => break,
                }
                face.push(self.read_i32(&lb.data[base..base + 4]) as u32);
            }
            if face.len() >= 3 {
                faces.push(face);
            }
        }
        faces
    }

    /// Read float3 data from a CustomData struct's layers (Blender 3.5+).
    /// Walks the `CustomData.layers` array looking for a layer whose name
    /// matches `attr_name` (e.g. "position") and type is CD_PROP_FLOAT3 (18).
    /// Returns the decoded vec3 positions.
    pub(crate) fn read_customdata_float3(
        &self,
        block: &BlendBlock,
        cd_offset: usize,
        count: usize,
        dna: &DnaType,
        attr_name: &str,
    ) -> Vec<[f32; 3]> {
        let ps = self.header.pointer_size as usize;
        // CustomData has `*layers` pointer and `totlayer` int.
        let layers_ptr_off = cd_offset; // First field of CustomData is typically *layers
        let totlayer_off = cd_offset + ps; // Second field is totlayer (int)
        if totlayer_off + 4 > block.data.len() {
            return Vec::new();
        }
        let layers_addr = self.read_block_ptr(block, layers_ptr_off, ps);
        let totlayer = self.read_i32(&block.data[totlayer_off..totlayer_off + 4]) as usize;
        if layers_addr == 0 || totlayer == 0 || totlayer > 1000 {
            return Vec::new();
        }

        let Some(layers_block) = self.blocks.iter().find(|b| b.old_address == layers_addr) else {
            return Vec::new();
        };
        // Each CustomDataLayer is a struct. We need its size to iterate.
        // Find CustomDataLayer struct size from DNA.
        let layer_size = Self::struct_size("CustomDataLayer", dna, ps).unwrap_or(0);
        if layer_size < 12 {
            return Vec::new();
        }
        // Within CustomDataLayer: type (int, offset 0), name (char[68], varies),
        // *data pointer. Find them from DNA.
        let type_off_in_layer = 0usize; // `type` is always the first int field
        let name_off_in_layer =
            Self::struct_field_offset("CustomDataLayer", "name", dna, ps).unwrap_or(8);
        let data_off_in_layer = Self::struct_field_offset("CustomDataLayer", "*data", dna, ps)
            .or_else(|| Self::struct_field_offset("CustomDataLayer", "data", dna, ps))
            .unwrap_or(ps + 4); // fallback guess

        for i in 0..totlayer.min(200) {
            let base = i * layer_size;
            if base + layer_size > layers_block.data.len() {
                break;
            }
            let layer_type = self.read_i32(
                &layers_block.data[base + type_off_in_layer..base + type_off_in_layer + 4],
            );
            // CD_PROP_FLOAT3 = 18 (positions), also check by name
            let name_start = base + name_off_in_layer;
            let name_end = (name_start + 64).min(layers_block.data.len());
            let name_slice = &layers_block.data[name_start..name_end];
            let nul = name_slice
                .iter()
                .position(|&b| b == 0)
                .unwrap_or(name_slice.len());
            let layer_name = String::from_utf8_lossy(&name_slice[..nul]);

            let is_match = layer_name == attr_name || (layer_type == 18 && attr_name == "position");
            if !is_match {
                continue;
            }

            let data_ptr_pos = base + data_off_in_layer;
            if data_ptr_pos + ps > layers_block.data.len() {
                continue;
            }
            let data_addr = Self::read_ptr(
                &layers_block.data,
                data_ptr_pos,
                ps,
                self.header.little_endian,
            );
            if data_addr == 0 {
                continue;
            }
            let Some(data_block) = self.blocks.iter().find(|b| b.old_address == data_addr) else {
                continue;
            };

            let mut verts = Vec::with_capacity(count);
            for v in 0..count {
                let off = v * 12;
                if off + 12 > data_block.data.len() {
                    break;
                }
                let x = self.read_f32(&data_block.data[off..off + 4]);
                let y = self.read_f32(&data_block.data[off + 4..off + 8]);
                let z = self.read_f32(&data_block.data[off + 8..off + 12]);
                if x.is_finite()
                    && y.is_finite()
                    && z.is_finite()
                    && x.abs() < 100_000.0
                    && y.abs() < 100_000.0
                    && z.abs() < 100_000.0
                {
                    verts.push([x, y, z]);
                }
            }
            if verts.len() >= 3 {
                return verts;
            }
        }
        Vec::new()
    }

    /// Read int data from a CustomData struct's layers (Blender 3.5+).
    /// Used for corner_verts (CD_PROP_INT32 = 7).
    pub(crate) fn read_customdata_int(
        &self,
        block: &BlendBlock,
        cd_offset: usize,
        count: usize,
        dna: &DnaType,
        attr_name: &str,
    ) -> Vec<i32> {
        let ps = self.header.pointer_size as usize;
        let layers_addr = self.read_block_ptr(block, cd_offset, ps);
        let totlayer_off = cd_offset + ps;
        if totlayer_off + 4 > block.data.len() {
            return Vec::new();
        }
        let totlayer = self.read_i32(&block.data[totlayer_off..totlayer_off + 4]) as usize;
        if layers_addr == 0 || totlayer == 0 || totlayer > 1000 {
            return Vec::new();
        }

        let Some(layers_block) = self.blocks.iter().find(|b| b.old_address == layers_addr) else {
            return Vec::new();
        };
        let layer_size = Self::struct_size("CustomDataLayer", dna, ps).unwrap_or(0);
        if layer_size < 12 {
            return Vec::new();
        }
        let name_off = Self::struct_field_offset("CustomDataLayer", "name", dna, ps).unwrap_or(8);
        let data_off = Self::struct_field_offset("CustomDataLayer", "*data", dna, ps)
            .or_else(|| Self::struct_field_offset("CustomDataLayer", "data", dna, ps))
            .unwrap_or(ps + 4);

        for i in 0..totlayer.min(200) {
            let base = i * layer_size;
            if base + layer_size > layers_block.data.len() {
                break;
            }
            let layer_type = self.read_i32(&layers_block.data[base..base + 4]);
            let name_start = base + name_off;
            let name_end = (name_start + 64).min(layers_block.data.len());
            let name_slice = &layers_block.data[name_start..name_end];
            let nul = name_slice
                .iter()
                .position(|&b| b == 0)
                .unwrap_or(name_slice.len());
            let layer_name = String::from_utf8_lossy(&name_slice[..nul]);

            // CD_PROP_INT32 = 7; match by name or type
            let is_match =
                layer_name == attr_name || layer_name.contains(attr_name) || layer_type == 7;
            if !is_match {
                continue;
            }

            let data_ptr_pos = base + data_off;
            if data_ptr_pos + ps > layers_block.data.len() {
                continue;
            }
            let data_addr = Self::read_ptr(
                &layers_block.data,
                data_ptr_pos,
                ps,
                self.header.little_endian,
            );
            if data_addr == 0 {
                continue;
            }
            let Some(data_block) = self.blocks.iter().find(|b| b.old_address == data_addr) else {
                continue;
            };

            let mut values = Vec::with_capacity(count);
            for v in 0..count {
                let off = v * 4;
                if off + 4 > data_block.data.len() {
                    break;
                }
                values.push(self.read_i32(&data_block.data[off..off + 4]));
            }
            if !values.is_empty() {
                return values;
            }
        }
        Vec::new()
    }

    /// Read face offsets from the `*poly_offsets` / `*face_offset_indices` pointer.
    /// In Blender 3.5+, this is an array of (poly_count + 1) ints giving the
    /// start index into corner_verts for each face.
    pub(crate) fn read_poly_offsets_ptr(
        &self,
        block: &BlendBlock,
        ptr_field: Option<usize>,
        poly_count: usize,
        ptr_size: usize,
    ) -> Vec<usize> {
        let Some(off) = ptr_field else {
            return Vec::new();
        };
        if off + ptr_size > block.data.len() {
            return Vec::new();
        }
        let addr = self.read_block_ptr(block, off, ptr_size);
        if addr == 0 {
            return Vec::new();
        }
        let Some(data_block) = self.blocks.iter().find(|b| b.old_address == addr) else {
            return Vec::new();
        };
        let expected = (poly_count + 1) * 4;
        if data_block.data.len() < expected {
            return Vec::new();
        }
        let mut offsets = Vec::with_capacity(poly_count + 1);
        for i in 0..=poly_count {
            let o = i * 4;
            offsets.push(self.read_i32(&data_block.data[o..o + 4]) as usize);
        }
        offsets
    }

    /// Read the modern `int *face_offset_indices` array (Blender 4.x).
    ///
    /// The array has `faces_num + 1` entries; face `f` spans corners
    /// `[offsets[f] .. offsets[f + 1])` and the final entry equals the total
    /// corner count. This reads *up to* `faces_num + 1` ints from the pointed
    /// block, clamping every `[start, end)` range to the block's actual byte
    /// length and stopping (rather than slicing past the end) once a read would
    /// overrun. The returned vector may therefore be shorter than
    /// `faces_num + 1` when the pointed block is short — never longer, never a
    /// panic (Requirements 2.1, 2.2, 2.3).
    pub(crate) fn read_face_offsets(
        &self,
        block: &BlendBlock,
        off: Option<usize>,
        faces_num: usize,
        ptr_size: usize,
    ) -> Vec<usize> {
        let Some(ptr_off) = off else {
            return Vec::new();
        };
        // The pointer itself must fit inside the mesh block.
        if ptr_off + ptr_size > block.data.len() || faces_num == 0 || faces_num >= 10_000_000 {
            return Vec::new();
        }
        let addr = self.read_block_ptr(block, ptr_off, ptr_size);
        if addr == 0 {
            return Vec::new();
        }
        let Some(data_block) = self.blocks.iter().find(|b| b.old_address == addr) else {
            return Vec::new();
        };
        let len = data_block.data.len();
        let want = faces_num + 1;
        let mut offsets = Vec::with_capacity(want);
        for i in 0..want {
            let start = i * 4;
            let end = start + 4;
            // Clamp to actual block length: skip (stop) rather than slice past.
            if start > len || end > len {
                break;
            }
            offsets.push(self.read_i32(&data_block.data[start..end]) as usize);
        }
        offsets
    }

    /// Read the `position` float3 attribute (Blender 4.x), matched by EXACT
    /// attribute name (`"position"`).
    ///
    /// Walks the CustomData `layers` array in `vert_data` looking for a layer
    /// whose name is exactly `"position"`, then decodes up to `verts_num`
    /// float3 positions from the layer's `*data` block. Every 12-byte vertex
    /// read is clamped to the data block's byte length; a read that would
    /// overrun stops the loop rather than slicing past the end (Requirements
    /// 2.1, 2.2, 2.3).
    pub(crate) fn read_positions(
        &self,
        block: &BlendBlock,
        off: Option<usize>,
        verts_num: usize,
        dna: &DnaType,
    ) -> Vec<[f32; 3]> {
        let Some(cd_offset) = off else {
            return Vec::new();
        };
        let ps = self.header.pointer_size as usize;
        let layers_ptr_off = cd_offset;
        let totlayer_off = cd_offset + ps;
        if totlayer_off + 4 > block.data.len() || verts_num == 0 || verts_num >= 10_000_000 {
            return Vec::new();
        }
        let layers_addr = self.read_block_ptr(block, layers_ptr_off, ps);
        let totlayer = self.read_i32(&block.data[totlayer_off..totlayer_off + 4]) as usize;
        if layers_addr == 0 || totlayer == 0 || totlayer > 1000 {
            return Vec::new();
        }
        let Some(layers_block) = self.blocks.iter().find(|b| b.old_address == layers_addr) else {
            return Vec::new();
        };
        let layer_size = Self::struct_size("CustomDataLayer", dna, ps).unwrap_or(0);
        if layer_size < 12 {
            return Vec::new();
        }
        let name_off = Self::struct_field_offset("CustomDataLayer", "name", dna, ps).unwrap_or(8);
        let data_off = Self::struct_field_offset("CustomDataLayer", "*data", dna, ps)
            .or_else(|| Self::struct_field_offset("CustomDataLayer", "data", dna, ps))
            .unwrap_or(ps + 4);

        for i in 0..totlayer.min(1000) {
            let base = i * layer_size;
            if base + layer_size > layers_block.data.len() {
                break;
            }
            // EXACT name match — no "any float3 layer" / "contains" fallback.
            if self.layer_name_at(&layers_block.data, base + name_off) != "position" {
                continue;
            }
            let data_ptr_pos = base + data_off;
            if data_ptr_pos + ps > layers_block.data.len() {
                continue;
            }
            let data_addr = Self::read_ptr(
                &layers_block.data,
                data_ptr_pos,
                ps,
                self.header.little_endian,
            );
            if data_addr == 0 {
                continue;
            }
            let Some(data_block) = self.blocks.iter().find(|b| b.old_address == data_addr) else {
                continue;
            };
            let len = data_block.data.len();
            let mut verts = Vec::with_capacity(verts_num);
            for v in 0..verts_num {
                let start = v * 12;
                let end = start + 12;
                if start > len || end > len {
                    break;
                }
                let x = self.read_f32(&data_block.data[start..start + 4]);
                let y = self.read_f32(&data_block.data[start + 4..start + 8]);
                let z = self.read_f32(&data_block.data[start + 8..end]);
                verts.push([x, y, z]);
            }
            if !verts.is_empty() {
                return verts;
            }
        }
        Vec::new()
    }

    /// Read the `.corner_vert` int attribute (Blender 4.x), matched by EXACT
    /// attribute name (`".corner_vert"`).
    ///
    /// This replaces the old `layer_type == 7 || name.contains(..)` heuristic
    /// (root cause #2), which could pick the wrong INT32 layer (e.g.
    /// `.corner_edge` or `material_index`). Walks the CustomData `layers` array
    /// in `corner_data` for a layer whose name is exactly `".corner_vert"`,
    /// then decodes up to `corners_num` ints from the layer's `*data` block.
    /// Every 4-byte read is clamped to the data block's byte length; a read
    /// that would overrun stops the loop rather than slicing past the end
    /// (Requirements 2.1, 2.2, 2.3).
    pub(crate) fn read_corner_verts(
        &self,
        block: &BlendBlock,
        off: Option<usize>,
        corners_num: usize,
        dna: &DnaType,
    ) -> Vec<i32> {
        let Some(cd_offset) = off else {
            return Vec::new();
        };
        let ps = self.header.pointer_size as usize;
        let layers_ptr_off = cd_offset;
        let totlayer_off = cd_offset + ps;
        if totlayer_off + 4 > block.data.len() || corners_num == 0 || corners_num >= 10_000_000 {
            return Vec::new();
        }
        let layers_addr = self.read_block_ptr(block, layers_ptr_off, ps);
        let totlayer = self.read_i32(&block.data[totlayer_off..totlayer_off + 4]) as usize;
        if layers_addr == 0 || totlayer == 0 || totlayer > 1000 {
            return Vec::new();
        }
        let Some(layers_block) = self.blocks.iter().find(|b| b.old_address == layers_addr) else {
            return Vec::new();
        };
        let layer_size = Self::struct_size("CustomDataLayer", dna, ps).unwrap_or(0);
        if layer_size < 12 {
            return Vec::new();
        }
        let name_off = Self::struct_field_offset("CustomDataLayer", "name", dna, ps).unwrap_or(8);
        let data_off = Self::struct_field_offset("CustomDataLayer", "*data", dna, ps)
            .or_else(|| Self::struct_field_offset("CustomDataLayer", "data", dna, ps))
            .unwrap_or(ps + 4);

        for i in 0..totlayer.min(1000) {
            let base = i * layer_size;
            if base + layer_size > layers_block.data.len() {
                break;
            }
            // EXACT name match — replaces the loose `layer_type == 7` heuristic.
            if self.layer_name_at(&layers_block.data, base + name_off) != ".corner_vert" {
                continue;
            }
            let data_ptr_pos = base + data_off;
            if data_ptr_pos + ps > layers_block.data.len() {
                continue;
            }
            let data_addr = Self::read_ptr(
                &layers_block.data,
                data_ptr_pos,
                ps,
                self.header.little_endian,
            );
            if data_addr == 0 {
                continue;
            }
            let Some(data_block) = self.blocks.iter().find(|b| b.old_address == data_addr) else {
                continue;
            };
            let len = data_block.data.len();
            let mut values = Vec::with_capacity(corners_num);
            for v in 0..corners_num {
                let start = v * 4;
                let end = start + 4;
                if start > len || end > len {
                    break;
                }
                values.push(self.read_i32(&data_block.data[start..end]));
            }
            if !values.is_empty() {
                return values;
            }
        }
        Vec::new()
    }

    /// Decode a NUL-terminated `CustomDataLayer.name` at `start` within a
    /// layers block, clamped to the block length. Returns an empty string when
    /// `start` is out of range so callers can compare by exact name safely.
    fn layer_name_at(&self, data: &[u8], start: usize) -> String {
        if start >= data.len() {
            return String::new();
        }
        let name_end = (start + 64).min(data.len());
        let name_slice = &data[start..name_end];
        let nul = name_slice
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(name_slice.len());
        String::from_utf8_lossy(&name_slice[..nul]).into_owned()
    }

    /// Get the total size of a DNA struct by name.
    pub(crate) fn struct_size(name: &str, dna: &DnaType, ptr_size: usize) -> Option<usize> {
        let type_idx = dna.types.iter().position(|t| t == name)?;
        let s = dna
            .structs
            .iter()
            .find(|s| s.type_index as usize == type_idx)?;
        let mut size = 0usize;
        for f in &s.fields {
            let ft = &dna.types[f.type_index as usize];
            let fn_ = &dna.names[f.name_index as usize];
            size += Self::field_size(ft, fn_, dna, ptr_size);
        }
        Some(size)
    }

    /// Read per-loop UV coordinates from the Mesh's `*mloopuv` pointer.
    ///
    /// `MLoopUV` is a struct of `{ float uv[2]; int flag; }` = 12 bytes in
    /// Blender 2.x–3.4. There is one entry per loop (same count as `totloop`).
    /// Returns an empty vec if the pointer is absent or the data doesn't parse.
    pub(crate) fn read_loop_uvs(
        &self,
        block: &BlendBlock,
        mloopuv_ptr: Option<usize>,
        loop_count: usize,
    ) -> Vec<[f32; 2]> {
        let Some(ptr_off) = mloopuv_ptr else {
            return Vec::new();
        };
        let ps = self.header.pointer_size as usize;
        if ptr_off + ps > block.data.len() || loop_count == 0 {
            return Vec::new();
        }
        let addr = self.read_block_ptr(block, ptr_off, ps);
        if addr == 0 {
            return Vec::new();
        }
        let Some(uv_block) = self.blocks.iter().find(|b| b.old_address == addr) else {
            return Vec::new();
        };
        // MLoopUV stride: 12 bytes (float[2] + int flag) in most versions.
        let stride = if loop_count > 0 && uv_block.data.len() >= loop_count * 8 {
            uv_block.data.len() / loop_count
        } else {
            12
        };
        if stride < 8 {
            return Vec::new();
        }
        let mut uvs = Vec::with_capacity(loop_count);
        for i in 0..loop_count {
            let base = i * stride;
            if base + 8 > uv_block.data.len() {
                break;
            }
            let u = self.read_f32(&uv_block.data[base..base + 4]);
            let v = self.read_f32(&uv_block.data[base + 4..base + 8]);
            if u.is_finite() && v.is_finite() {
                uvs.push([u, v]);
            } else {
                uvs.push([0.0, 0.0]);
            }
        }
        uvs
    }

    /// Read per-loop normals from the Mesh's custom split normal pointer or
    /// from a CustomData layer named ".corner_normal" (Blender 3.5+).
    ///
    /// Blender stores custom split normals as `short[4][3]` (quantized, in the
    /// `lnors_spacearr`) or as `float[3]` per loop. We try the float variant
    /// first (common in 3.5+ via CustomData), and fall back to decoding shorts.
    pub(crate) fn read_loop_normals(
        &self,
        block: &BlendBlock,
        off: &MeshOffsets,
        loop_count: usize,
        dna: &DnaType,
    ) -> Vec<[f32; 3]> {
        // Try CustomData ldata layer ".corner_normal" (Blender 4.x)
        if let Some(ldata_off) = off.ldata {
            let normals =
                self.read_customdata_float3(block, ldata_off, loop_count, dna, ".corner_normal");
            if normals.len() == loop_count {
                return normals;
            }
        }
        // Try the explicit *lnors pointer (rare, older files)
        let Some(ptr_off) = off.lnor_ptr else {
            return Vec::new();
        };
        let ps = self.header.pointer_size as usize;
        if ptr_off + ps > block.data.len() || loop_count == 0 {
            return Vec::new();
        }
        let addr = self.read_block_ptr(block, ptr_off, ps);
        if addr == 0 {
            return Vec::new();
        }
        let Some(nr_block) = self.blocks.iter().find(|b| b.old_address == addr) else {
            return Vec::new();
        };
        // float[3] per loop
        if nr_block.data.len() >= loop_count * 12 {
            let mut normals = Vec::with_capacity(loop_count);
            for i in 0..loop_count {
                let base = i * 12;
                let nx = self.read_f32(&nr_block.data[base..base + 4]);
                let ny = self.read_f32(&nr_block.data[base + 4..base + 8]);
                let nz = self.read_f32(&nr_block.data[base + 8..base + 12]);
                if nx.is_finite() && ny.is_finite() && nz.is_finite() {
                    normals.push([nx, ny, nz]);
                } else {
                    normals.push([0.0, 1.0, 0.0]);
                }
            }
            return normals;
        }
        Vec::new()
    }

    /// Read per-loop UVs from CustomData ldata (Blender 3.5+ attribute system).
    /// Looks for a layer named "UVMap" or ".uv_map" with type CD_PROP_FLOAT2 (16).
    pub(crate) fn read_customdata_uvs(
        &self,
        block: &BlendBlock,
        cd_offset: usize,
        count: usize,
        dna: &DnaType,
    ) -> Vec<[f32; 2]> {
        let ps = self.header.pointer_size as usize;
        let layers_ptr_off = cd_offset;
        let totlayer_off = cd_offset + ps;
        if totlayer_off + 4 > block.data.len() {
            return Vec::new();
        }
        let layers_addr = self.read_block_ptr(block, layers_ptr_off, ps);
        let totlayer = self.read_i32(&block.data[totlayer_off..totlayer_off + 4]) as usize;
        if layers_addr == 0 || totlayer == 0 || totlayer > 1000 {
            return Vec::new();
        }

        let Some(layers_block) = self.blocks.iter().find(|b| b.old_address == layers_addr) else {
            return Vec::new();
        };
        let layer_size = Self::struct_size("CustomDataLayer", dna, ps).unwrap_or(0);
        if layer_size < 12 {
            return Vec::new();
        }
        let name_off_in_layer =
            Self::struct_field_offset("CustomDataLayer", "name", dna, ps).unwrap_or(8);
        let type_off_in_layer = 0usize;
        let data_off_in_layer = Self::struct_field_offset("CustomDataLayer", "*data", dna, ps)
            .or_else(|| Self::struct_field_offset("CustomDataLayer", "data", dna, ps))
            .unwrap_or(ps + 4);

        for i in 0..totlayer.min(200) {
            let base = i * layer_size;
            if base + layer_size > layers_block.data.len() {
                break;
            }
            let layer_type = self.read_i32(
                &layers_block.data[base + type_off_in_layer..base + type_off_in_layer + 4],
            );
            let name_start = base + name_off_in_layer;
            let name_end = (name_start + 64).min(layers_block.data.len());
            let name_slice = &layers_block.data[name_start..name_end];
            let nul = name_slice
                .iter()
                .position(|&b| b == 0)
                .unwrap_or(name_slice.len());
            let layer_name = String::from_utf8_lossy(&name_slice[..nul]);

            // CD_PROP_FLOAT2 = 16, also accept CD_MTFACE = 5 (legacy UV)
            let is_uv = layer_type == 16
                || layer_type == 5
                || layer_name.contains("UV")
                || layer_name.contains("uv");
            if !is_uv {
                continue;
            }

            let data_ptr_pos = base + data_off_in_layer;
            if data_ptr_pos + ps > layers_block.data.len() {
                continue;
            }
            let data_addr = Self::read_ptr(
                &layers_block.data,
                data_ptr_pos,
                ps,
                self.header.little_endian,
            );
            if data_addr == 0 {
                continue;
            }
            let Some(data_block) = self.blocks.iter().find(|b| b.old_address == data_addr) else {
                continue;
            };

            let mut uvs = Vec::with_capacity(count);
            for v in 0..count {
                let off = v * 8;
                if off + 8 > data_block.data.len() {
                    break;
                }
                let u = self.read_f32(&data_block.data[off..off + 4]);
                let vv = self.read_f32(&data_block.data[off + 4..off + 8]);
                uvs.push(if u.is_finite() && vv.is_finite() {
                    [u, vv]
                } else {
                    [0.0, 0.0]
                });
            }
            if uvs.len() == count {
                return uvs;
            }
        }
        Vec::new()
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
