//! Per-object material extraction: resolves the object's first `Material`
//! (`MA`) block and reads its diffuse colour + PBR roughness/metallic.
//!
//! Blender's `Object` holds `Material **mat;` (a heap array of `Material*`)
//! plus `totcol`. We follow `**mat` → first `Material*` → the `MA` block, then
//! read `Material.{r,g,b,a,roughness,metallic}` (the viewport/legacy diffuse +
//! Eevee/Principled scalar defaults). This is the colour without parsing the
//! shader node tree — enough for skin vs garment differentiation.

use crate::blend::file::types::*;

/// Default surface when no material is linked.
const DEFAULT: ([f32; 4], f32, f32) = ([0.8, 0.8, 0.8, 1.0], 0.5, 0.0);


/// `DNA_material_defaults.h`: a material's viewport roughness and metallic,
/// and what Solid mode gives a surface with no material.
pub const VIEWPORT_PBR: [f32; 2] = [0.4, 0.0];

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
            let (c, r, m, _) = self.read_material_fields(ma, dna);
            return (c, r, m);
        }
        // 2) Materials linked to the mesh data (`Object.*data` → ME block).
        if let Some(data_off) = Self::struct_field_offset_exact("Object", "*data", dna, ps) {
            let me_addr = Self::read_ptr(&obj_block.data, data_off, ps, le);
            if let Some(me) = self.block_at_code(me_addr, "ME")
            {
                if let Some(ma) = self.first_material_block(me, "Mesh", dna) {
                    let (c, r, m, _) = self.read_material_fields(ma, dna);
            return (c, r, m);
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
    ) -> (Vec<crate::blend::file::types::BlendMaterial>, Vec<String>) {
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
                let me = self.block_at_code(me_addr, "ME")?;
                self.all_material_blocks(me, "Mesh", dna)
            });

        if let Some(blocks) = mat_blocks {
            for ma in blocks {
                self.scoped(ma, || {
                    let (color, roughness, metallic, bsdf) = self.read_material_fields(ma, dna);
                    let viewport_color = self.viewport_color(ma, dna);
                    let name = self
                        .extract_name_from_block(ma, "id.name")
                        .unwrap_or_default();
                    let tex = self.base_color_image(ma, dna).unwrap_or_else(|| self.image_texture_path(ma, dna));
                    if let Some(ref p) = tex {
                        if !all_textures.contains(p) {
                            all_textures.push(p.clone());
                        }
                    }
                    // The Base Color chain as a bake (`shade/bake.rs`):
                    // images that drive mix factors stay per-texel instead
                    // of folding to white.
                    let bake = self.base_bake(ma, dna);
                    for img in bake.iter().flat_map(|b| b.images.iter()) {
                        if !all_textures.contains(&img.key) {
                            all_textures.push(img.key.clone());
                        }
                    }
                    let graph = self.material_graph(ma, dna).unwrap_or_default();
                    materials.push(crate::blend::file::types::BlendMaterial {
                        name,
                        graph,
                        base_color: color,
                        viewport_color,
                        viewport_pbr: self.viewport_pbr(ma, dna),
                        roughness,
                        metallic,
                        base_color_texture: tex,
                        bsdf,
                        mix: self.surface_mix(ma, dna),
                        procedural: self.surface_procedural(ma, dna),
                        normal_map: self.normal_map(ma, dna),
                        maps: self.shade_maps(ma, dna),
                        bake,
                    });
                });
            }
        }

        if materials.is_empty() {
            let (color, roughness, metallic) = self.extract_object_material(obj_block, dna);
            materials.push(crate::blend::file::types::BlendMaterial {
                name: String::new(),
                base_color: color,
                viewport_color: color,
                viewport_pbr: VIEWPORT_PBR,
                roughness,
                metallic,
                base_color_texture: None,
                bsdf: Default::default(),
                mix: None,
                procedural: Default::default(),
                // The fallback material is the object's own colour, not a tree.
                graph: Default::default(),
                normal_map: None,
                maps: Default::default(),
                bake: None,
            });
        }

        (materials, all_textures)
    }

    pub(crate) fn read_material_fields(
        &self,
        ma: &BlendBlock,
        dna: &DnaType,
    ) -> ([f32; 4], f32, f32, crate::blend::scene::types::BlendBsdf) {
        let ps = self.header.pointer_size as usize;
        let f = |name: &str, default: f32| -> f32 {
            match Self::struct_field_offset_exact("Material", name, dna, ps) {
                Some(o) if o + 4 <= ma.data.len() => self.read_f32(&ma.data[o..o + 4]),
                _ => default,
            }
        };
        // The surface the Material Output is linked to (`surface.rs`) —
        // its Principled BSDF, or the other closures folded to one — wins;
        // the legacy `Material.r/g/b/roughness/metallic` fields are the
        // fallback for node-less materials.
        match self.surface_bsdf(ma, dna).or_else(|| self.principled(ma, dna)) {
            Some(b) => (b.base, b.roughness.clamp(0.0, 1.0), b.metallic.clamp(0.0, 1.0), b),
            None => {
                let color = [f("r", 0.8), f("g", 0.8), f("b", 0.8), f("a", 1.0)];
                let (r, m) = (f("roughness", 0.5).clamp(0.0, 1.0), f("metallic", 0.0).clamp(0.0, 1.0));
                let bsdf = crate::blend::scene::types::BlendBsdf { base: color, roughness: r, metallic: m, ..Default::default() };
                (color, r, m, bsdf)
            }
        }
    }

    /// `Material.roughness` / `metallic` — what Solid mode shades with.
    pub(crate) fn viewport_pbr(&self, ma: &BlendBlock, dna: &DnaType) -> [f32; 2] {
        let ps = self.header.pointer_size as usize;
        let f = |name: &str, default: f32| -> f32 {
            match Self::struct_field_offset_exact("Material", name, dna, ps) {
                Some(o) if o + 4 <= ma.data.len() => self.read_f32(&ma.data[o..o + 4]).clamp(0.0, 1.0),
                _ => default,
            }
        };
        [f("roughness", VIEWPORT_PBR[0]), f("metallic", VIEWPORT_PBR[1])]
    }

    /// `Material.r/g/b/a` — Blender's Solid-mode display colour
    /// (`View3DShading.color_type == V3D_SHADING_MATERIAL_COLOR`, the default).
    pub(crate) fn viewport_color(&self, ma: &BlendBlock, dna: &DnaType) -> [f32; 4] {
        let ps = self.header.pointer_size as usize;
        let f = |name: &str, default: f32| -> f32 {
            match Self::struct_field_offset_exact("Material", name, dna, ps) {
                Some(o) if o + 4 <= ma.data.len() => self.read_f32(&ma.data[o..o + 4]),
                _ => default,
            }
        };
        [f("r", 0.8), f("g", 0.8), f("b", 0.8), f("a", 1.0)]
    }
}
