//! `Mesh` field offsets (modern 4.x/5.x attribute layout + legacy fields),
//! computed from the file's DNA, plus the count / pointer readers.

use crate::blend::file::types::*;

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
                // DNA keeps the 3.6 spelling `poly_offset_indices` (DNA_rename).
                "*face_offset_indices" | "*poly_offset_indices" => {
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
                // 4.x/5.x name for the face CustomData (holds `material_index`).
                "pdata" | "face_data" => {
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
}
