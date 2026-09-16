//! Per-object material extraction: resolves the object's first `Material`
//! (`MA`) block and reads its diffuse colour + PBR roughness/metallic.
//!
//! Blender's `Object` holds `Material **mat;` (a heap array of `Material*`)
//! plus `totcol`. We follow `**mat` → first `Material*` → the `MA` block, then
//! read `Material.{r,g,b,a,roughness,metallic}` (the viewport/legacy diffuse +
//! Eevee/Principled scalar defaults). This is the colour without parsing the
//! shader node tree — enough for skin vs garment differentiation.

use super::types::*;

/// Default surface when no material is linked.
const DEFAULT: ([f32; 4], f32, f32) = ([0.8, 0.8, 0.8, 1.0], 0.5, 0.0);

impl BlendFile {
    /// Resolve the first material's colour + roughness/metallic for an object.
    /// Materials may be linked to the Object (`Object.**mat`) or — more
    /// commonly — to its Mesh data (`Mesh.**mat`); try both.
    pub(crate) fn extract_object_material(
        &self,
        obj_block: &BlendBlock,
        dna: &DnaType,
    ) -> ([f32; 4], f32, f32) {
        let ps = self.header.pointer_size as usize;
        let le = self.header.little_endian;

        // 1) Materials linked directly to the object.
        if let Some(ma) = self.first_material_block(obj_block, "Object", dna) {
            return self.read_material_fields(ma, dna);
        }
        // 2) Materials linked to the mesh data (`Object.*data` → ME block).
        if let Some(data_off) = Self::struct_field_offset_exact("Object", "*data", dna, ps) {
            let me_addr = Self::read_ptr(&obj_block.data, data_off, ps, le);
            if let Some(me) = self
                .blocks
                .iter()
                .find(|b| b.old_address == me_addr && b.code_str == "ME")
            {
                if let Some(ma) = self.first_material_block(me, "Mesh", dna) {
                    return self.read_material_fields(ma, dna);
                }
            }
        }
        DEFAULT
    }

    /// Extract ALL material slots for an object (multi-material support).
    /// Returns the full list of `BlendMaterial` structs plus a flat list of
    /// all referenced texture paths. Falls back to a single-entry list with
    /// the primary material when multi-material resolution fails.
    pub(crate) fn extract_all_materials(
        &self,
        obj_block: &BlendBlock,
        dna: &DnaType,
    ) -> (Vec<super::types::BlendMaterial>, Vec<String>) {
        let ps = self.header.pointer_size as usize;
        let le = self.header.little_endian;
        let mut materials = Vec::new();
        let mut all_textures = Vec::new();

        // Try Object.**mat array first, then Mesh.**mat.
        let mat_blocks = self
            .all_material_blocks(obj_block, "Object", dna)
            .or_else(|| {
                let data_off = Self::struct_field_offset_exact("Object", "*data", dna, ps)?;
                let me_addr = Self::read_ptr(&obj_block.data, data_off, ps, le);
                let me = self
                    .blocks
                    .iter()
                    .find(|b| b.old_address == me_addr && b.code_str == "ME")?;
                self.all_material_blocks(me, "Mesh", dna)
            });

        if let Some(blocks) = mat_blocks {
            for ma in blocks {
                let (color, roughness, metallic) = self.read_material_fields(ma, dna);
                let name = self
                    .extract_name_from_block(ma, "id.name")
                    .unwrap_or_default();
                let tex = self.image_texture_path(ma, dna);
                if let Some(ref p) = tex {
                    if !all_textures.contains(p) {
                        all_textures.push(p.clone());
                    }
                }
                materials.push(super::types::BlendMaterial {
                    name,
                    base_color: color,
                    roughness,
                    metallic,
                    base_color_texture: tex,
                });
            }
        }

        if materials.is_empty() {
            let (color, roughness, metallic) = self.extract_object_material(obj_block, dna);
            materials.push(super::types::BlendMaterial {
                name: String::new(),
                base_color: color,
                roughness,
                metallic,
                base_color_texture: None,
            });
        }

        (materials, all_textures)
    }

    /// Follow `<struct>.**mat` and return ALL material blocks (not just the first).
    fn all_material_blocks<'a>(
        &'a self,
        block: &BlendBlock,
        struct_name: &str,
        dna: &DnaType,
    ) -> Option<Vec<&'a BlendBlock>> {
        let ps = self.header.pointer_size as usize;
        let le = self.header.little_endian;
        let off = Self::struct_field_offset_exact(struct_name, "**mat", dna, ps)?;
        let arr_addr = Self::read_ptr(&block.data, off, ps, le);
        if arr_addr == 0 {
            return None;
        }
        let arr = self.blocks.iter().find(|b| b.old_address == arr_addr)?;
        // totcol tells us how many material pointers are in the array.
        let totcol_off = Self::struct_field_offset_exact(struct_name, "totcol", dna, ps);
        let count = totcol_off
            .and_then(|o| {
                (o + 2 <= block.data.len()).then(|| self.read_u16(&block.data[o..o + 2]) as usize)
            })
            .unwrap_or(1)
            .min(64); // sanity cap

        let mut result = Vec::new();
        for i in 0..count {
            let ptr_off = i * ps;
            if ptr_off + ps > arr.data.len() {
                break;
            }
            let mat_addr = Self::read_ptr(&arr.data, ptr_off, ps, le);
            if mat_addr == 0 {
                continue;
            }
            if let Some(ma) = self
                .blocks
                .iter()
                .find(|b| b.old_address == mat_addr && b.code_str == "MA")
            {
                result.push(ma);
            }
        }
        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    /// Walk the material's node tree looking for an Image Texture node and
    /// extract its filepath string. Returns the raw Blender path (may start
    /// with `//` for blend-relative).
    fn image_texture_path(&self, ma: &BlendBlock, dna: &DnaType) -> Option<String> {
        let ps = self.header.pointer_size as usize;
        let le = self.header.little_endian;
        let nt_off = Self::struct_field_offset_exact("Material", "*nodetree", dna, ps)?;
        let nt_addr = Self::read_ptr(&ma.data, nt_off, ps, le);
        let nt = self.blocks.iter().find(|b| b.old_address == nt_addr)?;

        let nodes_off = Self::struct_field_offset_exact("bNodeTree", "nodes", dna, ps)?;
        let next_off = Self::struct_field_offset_exact("bNode", "*next", dna, ps)?;
        let idname_off = Self::struct_field_offset_exact("bNode", "idname", dna, ps)?;
        let id_off = Self::struct_field_offset_exact("bNode", "*id", dna, ps);

        let mut addr = Self::read_ptr(&nt.data, nodes_off, ps, le);
        let mut guard = 0;
        while addr != 0 && guard < 8192 {
            guard += 1;
            let Some(node) = self.blocks.iter().find(|b| b.old_address == addr) else {
                break;
            };
            let idname = read_cstr(&node.data, idname_off, 64);
            // ShaderNodeTexImage is the Image Texture node.
            if idname.contains("TexImage") || idname.contains("TEX_IMAGE") {
                // The Image datablock is linked via bNode.*id → IM block.
                if let Some(id_field) = id_off {
                    let img_addr = Self::read_ptr(&node.data, id_field, ps, le);
                    if img_addr != 0 {
                        if let Some(im) = self.blocks.iter().find(|b| b.old_address == img_addr) {
                            // Image.filepath is a char[1024] at a known offset.
                            let fp_off =
                                Self::struct_field_offset_exact("Image", "filepath", dna, ps)
                                    .or_else(|| {
                                        Self::struct_field_offset("Image", "name", dna, ps)
                                    });
                            if let Some(fp) = fp_off {
                                let path = read_cstr(&im.data, fp, 1024);
                                if !path.is_empty() {
                                    return Some(path);
                                }
                            }
                        }
                    }
                }
            }
            addr = Self::read_ptr(&node.data, next_off, ps, le);
        }
        None
    }

    /// Follow `<struct>.**mat` → first `Material*` → the `MA` block.
    fn first_material_block(
        &self,
        block: &BlendBlock,
        struct_name: &str,
        dna: &DnaType,
    ) -> Option<&BlendBlock> {
        let ps = self.header.pointer_size as usize;
        let le = self.header.little_endian;
        let off = Self::struct_field_offset_exact(struct_name, "**mat", dna, ps)?;
        let arr_addr = Self::read_ptr(&block.data, off, ps, le);
        if arr_addr == 0 {
            return None;
        }
        let arr = self.blocks.iter().find(|b| b.old_address == arr_addr)?;
        let mat_addr = Self::read_ptr(&arr.data, 0, ps, le);
        self.blocks
            .iter()
            .find(|b| b.old_address == mat_addr && b.code_str == "MA")
    }

    fn read_material_fields(&self, ma: &BlendBlock, dna: &DnaType) -> ([f32; 4], f32, f32) {
        let ps = self.header.pointer_size as usize;
        let f = |name: &str, default: f32| -> f32 {
            match Self::struct_field_offset_exact("Material", name, dna, ps) {
                Some(o) if o + 4 <= ma.data.len() => self.read_f32(&ma.data[o..o + 4]),
                _ => default,
            }
        };
        let roughness = f("roughness", 0.5).clamp(0.0, 1.0);
        let metallic = f("metallic", 0.0).clamp(0.0, 1.0);
        // Prefer the Principled BSDF "Base Color" from the shader node tree
        // (where modern Blender stores the authored colour); fall back to the
        // legacy/viewport diffuse `Material.r/g/b`.
        let color = self.node_tree_base_color(ma, dna).unwrap_or([
            f("r", 0.8),
            f("g", 0.8),
            f("b", 0.8),
            f("a", 1.0),
        ]);
        (color, roughness, metallic)
    }

    /// Walk `Material.*nodetree` → node list → Principled BSDF → `Base Color`
    /// input socket → `bNodeSocketValueRGBA.value`.
    fn node_tree_base_color(&self, ma: &BlendBlock, dna: &DnaType) -> Option<[f32; 4]> {
        let ps = self.header.pointer_size as usize;
        let le = self.header.little_endian;
        let nt_off = Self::struct_field_offset_exact("Material", "*nodetree", dna, ps)?;
        let nt_addr = Self::read_ptr(&ma.data, nt_off, ps, le);
        let nt = self.blocks.iter().find(|b| b.old_address == nt_addr)?;

        let nodes_off = Self::struct_field_offset_exact("bNodeTree", "nodes", dna, ps)?;
        let next_off = Self::struct_field_offset_exact("bNode", "*next", dna, ps)?;
        let inputs_off = Self::struct_field_offset_exact("bNode", "inputs", dna, ps)?;
        let idname_off = Self::struct_field_offset_exact("bNode", "idname", dna, ps);
        let type_off = Self::struct_field_offset_exact("bNode", "type", dna, ps);

        let mut addr = Self::read_ptr(&nt.data, nodes_off, ps, le);
        let mut guard = 0;
        while addr != 0 && guard < 8192 {
            guard += 1;
            let node = self.blocks.iter().find(|b| b.old_address == addr)?;
            let principled = idname_off
                .map(|o| read_cstr(&node.data, o, 64).contains("Principled"))
                .unwrap_or(false)
                || type_off
                    .map(|o| o + 2 <= node.data.len() && self.read_u16(&node.data[o..o + 2]) == 194)
                    .unwrap_or(false);
            if principled {
                if let Some(c) = self.socket_base_color(node, inputs_off, dna) {
                    return Some(c);
                }
            }
            addr = Self::read_ptr(&node.data, next_off, ps, le);
        }
        None
    }

    /// Find a node's "Base Color"/"Color" input socket and read its RGBA value.
    fn socket_base_color(
        &self,
        node: &BlendBlock,
        inputs_off: usize,
        dna: &DnaType,
    ) -> Option<[f32; 4]> {
        let ps = self.header.pointer_size as usize;
        let le = self.header.little_endian;
        let next_off = Self::struct_field_offset_exact("bNodeSocket", "*next", dna, ps)?;
        let name_off = Self::struct_field_offset_exact("bNodeSocket", "name", dna, ps)?;
        let val_off = Self::struct_field_offset_exact("bNodeSocket", "*default_value", dna, ps)?;

        let mut addr = Self::read_ptr(&node.data, inputs_off, ps, le);
        let mut guard = 0;
        while addr != 0 && guard < 512 {
            guard += 1;
            let sock = self.blocks.iter().find(|b| b.old_address == addr)?;
            let name = read_cstr(&sock.data, name_off, 64);
            if name == "Base Color" || name == "Color" {
                let val_addr = Self::read_ptr(&sock.data, val_off, ps, le);
                if let Some(vb) = self.blocks.iter().find(|b| b.old_address == val_addr) {
                    if vb.data.len() >= 16 {
                        return Some([
                            self.read_f32(&vb.data[0..4]),
                            self.read_f32(&vb.data[4..8]),
                            self.read_f32(&vb.data[8..12]),
                            self.read_f32(&vb.data[12..16]),
                        ]);
                    }
                }
            }
            addr = Self::read_ptr(&sock.data, next_off, ps, le);
        }
        None
    }

    /// Byte offset of a field by EXACT name (unlike `struct_field_offset`,
    /// which uses `contains` and would mis-match short names like `r`).
    pub(crate) fn struct_field_offset_exact(
        struct_name: &str,
        field_name: &str,
        dna: &DnaType,
        ptr_size: usize,
    ) -> Option<usize> {
        let struct_idx = dna.types.iter().position(|t| t == struct_name)?;
        let s = dna
            .structs
            .iter()
            .find(|s| s.type_index as usize == struct_idx)?;
        let mut offset = 0;
        for fld in &s.fields {
            let f_name = &dna.names[fld.name_index as usize];
            if f_name == field_name {
                return Some(offset);
            }
            let f_type = &dna.types[fld.type_index as usize];
            offset += Self::field_size(f_type, f_name, dna, ptr_size);
        }
        None
    }
}

/// Read a fixed-width C string field (NUL-terminated) from `data` at `off`.
fn read_cstr(data: &[u8], off: usize, max: usize) -> String {
    if off >= data.len() {
        return String::new();
    }
    let end = (off + max).min(data.len());
    let slice = &data[off..end];
    let n = slice.iter().position(|&b| b == 0).unwrap_or(slice.len());
    String::from_utf8_lossy(&slice[..n]).to_string()
}
