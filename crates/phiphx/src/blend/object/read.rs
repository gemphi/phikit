//! Object block extraction (assembling `BlendSceneObject`s).

use crate::blend::file::types::*;
use crate::Result;

impl BlendFile {
    /// Simple object extraction - includes metadata (modifiers, instances) and mesh data
    pub(crate) fn extract_object_simple(
        &self,
        block: &BlendBlock,
    ) -> Result<Option<BlendSceneObject>> {
        let name = self.extract_name_from_block(block, "id.name")?;
        let raw_xf = self
            .dna
            .as_ref()
            .map(|dna| self.read_object_transform_raw(block, dna))
            .unwrap_or_default();

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
            loop_colors,
            loop_normals,
            face_materials,
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
                // Only mesh objects own mesh data; a Camera / Light / Empty
                // `*data` block must not be scanned as geometry.
                // A text object's mesh is made from its string, a curve
                // object's from its splines (`blend/text`, `blend/curve`).
                let (verts, fac, uvs, norms, face_mats) = match t.as_str() {
                    "MESH" => self.extract_mesh_data_full(block, dna),
                    "FONT" => self.object_text_mesh(block, dna).map_or_else(Default::default, |m| (m.vertices, m.faces, Vec::new(), Vec::new(), m.face_materials)),
                    "CURVE" => self.object_curve_mesh(block, dna).map_or_else(Default::default, |m| (m.vertices, m.faces, Vec::new(), Vec::new(), m.face_materials)),
                    _ => (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()),
                };
                // The active colour attribute, a colour per corner (Solid's
                // Color ▸ Attribute).
                let loop_colors = if t == "MESH" {
                    self.mesh_block_of(block, dna).map_or_else(Vec::new, |me| {
                        let active = self.cstring_at(me, "Mesh", "*active_color_attribute", dna);
                        self.attr_corner_colors(me, dna, active.as_deref(), &fac, verts.len())
                    })
                } else {
                    Vec::new()
                };

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
                    loop_colors,
                    norms,
                    face_mats,
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
                Vec::new(),
                Vec::new(),
            )
        };

        let data = match &self.dna {
            Some(dna) => self.extract_object_data(block, &obj_type, dna),
            None => Default::default(),
        };
        let face_smooth = match (&self.dna, obj_type.as_str()) {
            (Some(dna), "MESH") => self.extract_face_smooth(block, dna, faces.len()),
            _ => Vec::new(),
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
            matrix_local: mat,
            block_address: block.old_address,
            parent_address: raw_xf.parent_address,
            parentinv: raw_xf.parentinv,
            partype: raw_xf.partype,
            parent_verts: raw_xf.par,
            visibility: self.dna.as_ref().map(|dna| self.object_visibility(block, dna)).unwrap_or_default(),
            color: self.dna.as_ref().map(|dna| self.object_color(block, dna)).unwrap_or([1.0; 4]),
            shadow: self.dna.as_ref().map(|dna| self.object_shadow(block, dna)).unwrap_or_default(),
            vertices,
            faces: faces.into(),
            loop_normals,
            loop_uvs,
            loop_colors,
            face_materials,
            data,
            face_smooth,
            modifiers,
            skipped_modifiers: Vec::new(),
            instance_targets,
            is_instance,
            instance_source: None,
            base_color,
            roughness,
            metallic,
            materials,
            texture_paths,
            loose_edges: Vec::new(),
        }))
    }
}
