//! Material shader graph walk: `Material.*nodetree` → `bNode` list →
//! the first Image Texture's
//! `Image.filepath` (`DNA_node_types.h`, `DNA_material_types.h`).

use crate::blend::file::types::*;

impl BlendFile {
    /// Walk the material's node tree looking for an Image Texture node and
    /// extract its image key (`image.rs`): the raw Blender path (may start
    /// with `//` for blend-relative), or the image name when it has no path.
    pub(crate) fn image_texture_path(&self, ma: &BlendBlock, dna: &DnaType) -> Option<String> {
        let ps = self.header.pointer_size as usize;
        let le = self.header.little_endian;
        let nt_off = Self::struct_field_offset_exact("Material", "*nodetree", dna, ps)?;
        let nt_addr = Self::read_ptr(&ma.data, nt_off, ps, le);
        let nt = self.block_at(nt_addr)?;

        let nodes_off = Self::struct_field_offset_exact("bNodeTree", "nodes", dna, ps)?;
        let next_off = Self::struct_field_offset_exact("bNode", "*next", dna, ps)?;
        let idname_off = Self::struct_field_offset_exact("bNode", "idname", dna, ps)?;
        let id_off = Self::struct_field_offset_exact("bNode", "*id", dna, ps);

        let mut addr = Self::read_ptr(&nt.data, nodes_off, ps, le);
        let mut guard = 0;
        while addr != 0 && guard < 8192 {
            guard += 1;
            let Some(node) = self.block_at(addr) else {
                break;
            };
            let idname = read_cstr(&node.data, idname_off, 64);
            // ShaderNodeTexImage is the Image Texture node.
            if idname.contains("TexImage") || idname.contains("TEX_IMAGE") {
                // The Image datablock is linked via bNode.*id → IM block.
                if let Some(id_field) = id_off {
                    let img_addr = Self::read_ptr(&node.data, id_field, ps, le);
                    if img_addr != 0 {
                        if let Some(im) = self.block_at(img_addr) {
                            let key = self.image_key(im, dna);
                            if !key.is_empty() {
                                return Some(key);
                            }
                        }
                    }
                }
            }
            addr = Self::read_ptr(&node.data, next_off, ps, le);
        }
        None
    }
}

/// Read a fixed-width C string field (NUL-terminated) from `data` at `off`.
pub(crate) fn read_cstr(data: &[u8], off: usize, max: usize) -> String {
    if off >= data.len() {
        return String::new();
    }
    let end = (off + max).min(data.len());
    let slice = &data[off..end];
    let n = slice.iter().position(|&b| b == 0).unwrap_or(slice.len());
    String::from_utf8_lossy(&slice[..n]).to_string()
}
