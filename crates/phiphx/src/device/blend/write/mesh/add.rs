//! Adding a mesh block to the file, from the file's own template mesh.
use crate::device::blend::write::mesh::*;

impl BlendFile {
    /// Whether this file's meshes are ones the writer can add to.
    pub fn writes_meshes(&self) -> bool {
        self.header.version_number >= OLDEST_MESH_LAYOUT && self.dna.as_ref().is_some_and(|d| d.types.iter().any(|t| t == "Mesh"))
    }

    pub(super) fn push_raw(&mut self, after: usize, data: Vec<u8>) -> (usize, u64) {
        let address = self.fresh_address();
        let block = BlendBlock { code: *b"DATA", code_str: "DATA".into(), size: data.len() as u64, old_address: address, sdna_index: 0, count: 1, data };
        let at = (after + 1).min(self.blocks.len());
        self.blocks.insert(at, block);
        self.reindex();
        (at, address)
    }

    /// A `DATA` block of `count` zeroed `struct_name`s.
    pub(super) fn push_structs(&mut self, struct_name: &str, count: usize, after: usize) -> Option<(usize, u64)> {
        let sdna_index = self.sdna_index_of(struct_name)?;
        let size = self.sizeof(struct_name)? * count.max(1);
        let address = self.fresh_address();
        let block = BlendBlock { code: *b"DATA", code_str: "DATA".into(), size: size as u64, old_address: address, sdna_index, count: count.max(1) as u64, data: vec![0; size] };
        let at = (after + 1).min(self.blocks.len());
        self.blocks.insert(at, block);
        self.reindex();
        Some((at, address))
    }

    /// The first mesh block, to clone a new mesh from.
    pub fn mesh_template(&self) -> Option<usize> {
        self.blocks.iter().position(|b| b.code_str == "ME")
    }

    /// A new mesh `mesh`, cloned from `template` and laid out as this file
    /// lays meshes out, inserted before `DNA1`. Its index and address.
    pub fn add_mesh(&mut self, template: usize, mesh: &BlendNewMesh) -> Option<(usize, u64)> {
        if !self.writes_meshes() {
            return None;
        }
        let dna1 = self.blocks.iter().position(|b| &b.code == DNA1).unwrap_or(self.blocks.len());
        let (me, address) = self.clone_block_cleared(template, dna1.saturating_sub(1))?;
        self.set_id_name(me, &mesh.name);
        self.set_int(me, "Mesh", "id.us", 1);
        self.set_int(me, "Mesh", "id.flag", 0);
        self.set_int(me, "Mesh", "id.tag", 0);
        self.set_int(me, "Mesh", "totcol", 0);
        let (edges, corner_edges) = edges_of(&mesh.faces);
        let corner_verts: Vec<u32> = mesh.faces.iter().flatten().copied().collect();
        let mut offsets: Vec<u32> = Vec::with_capacity(mesh.faces.len() + 1);
        let mut acc = 0u32;
        for face in &mesh.faces {
            offsets.push(acc);
            acc += face.len() as u32;
        }
        offsets.push(acc);
        self.set_int(me, "Mesh", "totvert", mesh.vertices.len() as i64);
        self.set_int(me, "Mesh", "totedge", edges.len() as i64);
        self.set_int(me, "Mesh", "totpoly", mesh.faces.len() as i64);
        self.set_int(me, "Mesh", "totloop", corner_verts.len() as i64);
        let le = self.header.little_endian;
        let ints = |v: &[u32]| -> Vec<u8> { v.iter().flat_map(|i| le_bytes(le, i.to_le_bytes())).collect() };
        let positions: Vec<u8> = mesh.vertices.iter().flatten().flat_map(|f| le_bytes(le, f.to_le_bytes())).collect();
        let pairs: Vec<u8> = edges.iter().flatten().copied().collect::<Vec<u32>>().iter().flat_map(|i| le_bytes(le, i.to_le_bytes())).collect();
        let sharp: Vec<u8> = vec![1; mesh.faces.len()];
        // Face offsets, right after the mesh.
        let (_, offsets_address) = self.push_raw(me, ints(&offsets));
        self.set_ptr(me, "Mesh", "*poly_offset_indices", offsets_address);
        if self.header.version_number >= 500 {
            self.write_attribute_storage(me, &positions, &pairs, &ints(&corner_verts), &ints(&corner_edges), &sharp, (mesh.vertices.len(), edges.len(), mesh.faces.len(), corner_verts.len()));
        } else {
            self.write_custom_data(me, &positions, &edges, &ints(&corner_verts), &ints(&corner_edges), &sharp);
        }
        Some((me, address))
    }

    /// Write into one struct of an array block: field `field` of the
    /// `struct_name` at byte `base`.
    pub(super) fn set_struct_field(&mut self, block: usize, base: usize, struct_name: &str, field: &str, value: i64) -> bool {
        let Some(dna) = self.dna.as_ref() else { return false };
        let Some((off, size)) = self.field_span(struct_name, field, dna) else { return false };
        let le = self.header.little_endian;
        let Some(data) = self.blocks.get_mut(block).map(|b| &mut b.data) else { return false };
        let off = base + off;
        if off + size > data.len() {
            return false;
        }
        let bytes = if le { value.to_le_bytes() } else { value.to_be_bytes() };
        match size {
            1 => data[off] = value as u8,
            2 | 4 | 8 => data[off..off + size].copy_from_slice(if le { &bytes[..size] } else { &bytes[8 - size..] }),
            _ => return false,
        }
        true
    }

    pub(super) fn set_struct_text(&mut self, block: usize, base: usize, struct_name: &str, field: &str, text: &str) -> bool {
        let Some(dna) = self.dna.as_ref() else { return false };
        let Some((off, size)) = self.field_span(struct_name, field, dna) else { return false };
        let Some(data) = self.blocks.get_mut(block).map(|b| &mut b.data) else { return false };
        let off = base + off;
        if size == 0 || off + size > data.len() {
            return false;
        }
        let bytes: Vec<u8> = text.bytes().take(size - 1).collect();
        data[off..off + size].fill(0);
        data[off..off + bytes.len()].copy_from_slice(&bytes);
        true
    }
}
