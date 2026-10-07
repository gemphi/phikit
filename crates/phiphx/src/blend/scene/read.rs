//! Scene extraction: picks `FileGlobal.curscene`, walks its collection tree
//! (`collection.rs`), resolves world matrices (`world.rs`) and folds in the
//! view layer (`view_layer.rs`) and scene properties (`scene_props.rs`).

use crate::blend::file::types::*;
use crate::Result;

impl BlendFile {
    /// Extract scene information from the parsed file
    pub fn extract_scene(&self) -> Result<BlendScene> {
        self.extract_scene_with_progress(&|_, _| true)
    }

    /// [`Self::extract_scene`] reporting `(fraction 0..=1, stage)` per object
    /// extracted. The callback returns `false` to cancel.
    pub fn extract_scene_with_progress(
        &self,
        progress: &(dyn Fn(f32, &str) -> bool + Sync),
    ) -> Result<BlendScene> {
        // Denominator for per-object progress: every OB block in the file.
        let total_objects = self.blocks.iter().filter(|b| b.code_str == "OB").count().max(1);
        let mut scene = BlendScene {
            name: "Scene".to_string(),
            collections: Vec::new(),
            objects: Vec::new(),
            active_object: None,
            selected: Vec::new(),
            hidden: Vec::new(),
            props: Default::default(),
            deferred: None,
        };

        let dna = match &self.dna {
            Some(d) => d,
            None => return Ok(scene),
        };
        let ptr_size = self.header.pointer_size as usize;
        let little_endian = self.header.little_endian;

        let mut unique_objs = Vec::new();

        // Blender's `read_global` reads `FileGlobal.curscene` from the GLOB
        // block and opens *that* scene — not the first SC block in the file.
        // Fall back to the first SC block only when GLOB is absent/unreadable.
        let cur_scene_block = self.file_global_curscene(dna);
        if let Some(scene_block) = cur_scene_block
            .or_else(|| self.blocks.iter().find(|b| b.code_str == "SC"))
        {
            let sc_name = self
                .extract_name_from_block(scene_block, "id.name")
                .unwrap_or_else(|_| "Scene".to_string());
            if !sc_name.is_empty() {
                scene.name = sc_name;
            }

            if let Some(master_offset) =
                Self::struct_field_offset("Scene", "master_collection", dna, ptr_size)
            {
                let master_addr =
                    Self::read_ptr(&scene_block.data, master_offset, ptr_size, little_endian);
                let report = |objs: &Vec<BlendSceneObject>, name: &str| -> bool {
                    let frac = objs.len() as f32 / total_objects as f32;
                    progress(frac.min(1.0), &format!("Reading objects ({}/{total_objects}) {name}", objs.len()))
                };
                if let Some(master_collection) = self.extract_collection_hierarchy(
                    master_addr,
                    dna,
                    ptr_size,
                    little_endian,
                    &mut unique_objs,
                    0,
                    &report,
                ) {
                    scene.collections.push(master_collection);
                }
            }

            // Fallback for older Blender or if master_collection failed: Extract all OB blocks
            if unique_objs.is_empty() {
                for block in self.blocks.iter().filter(|b| b.code_str == "OB") {
                    if let Ok(Some(obj)) = self.extract_object(block) {
                        let frac = unique_objs.len() as f32 / total_objects as f32;
                        if !progress(frac, &format!("Reading objects {}", obj.name)) {
                            return Err(crate::PointCloudError::Cancelled);
                        }
                        unique_objs.push(obj);
                    }
                }
            }
            if self.cancel_requested.load(std::sync::atomic::Ordering::Relaxed) {
                return Err(crate::PointCloudError::Cancelled);
            }
        }

        // `ID.name` is already unique per ID type in Blender; use it verbatim
        // (re-suffixing here renamed legitimately unique objects).
        scene.objects = unique_objs;
        Self::resolve_world_matrices(&mut scene.objects);

        // View layer (selection / hidden / active / collection flags) and the
        // scene-level properties Blender shows on open.
        if let Some(sc) = cur_scene_block.or_else(|| self.blocks.iter().find(|b| b.code_str == "SC")) {
            let vl = self.extract_view_layer(sc, dna);
            for (i, o) in scene.objects.iter().enumerate() {
                let flag = vl.base_flags.get(&o.block_address).copied().unwrap_or(0);
                if flag & crate::blend::object::layer::BASE_SELECTED != 0 {
                    scene.selected.push(i);
                }
                if flag & crate::blend::object::layer::BASE_HIDDEN != 0 {
                    scene.hidden.push(i);
                }
                if vl.active_object != 0 && o.block_address == vl.active_object {
                    scene.active_object = Some(i);
                }
            }
            for c in &mut scene.collections {
                crate::blend::object::world::apply_layer_flags(c, &vl.collection_flags);
            }
            scene.props = self.extract_scene_props(sc, dna);
            if !vl.name.is_empty() {
                scene.props.view_layer_name = vl.name;
            }
            scene.props.ui = self.extract_ui_state(Some(sc), dna);
            scene.props.packed_images = self.packed_images(dna);
            scene.props.compositor = self.compositor();
            // Everything below used to run after a single `progress(1.0, …)`,
            // with no further report and no cancel check — modifier
            // evaluation (which may skin on the GPU), actions, motion, NLA
            // and drivers. The bar sat at one number and Cancel was dead for
            // the whole phase, which is what a hang looks like from outside.
            // Each step now reports before it runs and honours a decline.
            if !progress(0.90, "Evaluating modifiers") {
                return Err(crate::PointCloudError::Cancelled);
            }
            // Progressive open: capture the modifier work but do not apply it
            // now — the base meshes show first and the apply runs off the read
            // thread (`SceneModel::from_blend_progressive`). Eager open applies
            // it here as before.
            if crate::blend::progressive_open() {
                scene.deferred = Some(std::sync::Arc::new(self.modifier_jobs(&scene.objects, dna)));
            } else {
                scene.props.evaluated = self.evaluate_modifiers(&scene.objects, dna);
            }
            if !progress(0.94, "Reading actions") {
                return Err(crate::PointCloudError::Cancelled);
            }
            let actions = self.actions(dna);
            scene.props.actions = self.action_summaries(&scene.objects, &actions, dna);
            if !progress(0.96, "Reading motion") {
                return Err(crate::PointCloudError::Cancelled);
            }
            scene.props.motion = self.motion_list(&scene.objects, &actions, dna);
            scene.props.material_drivers = self.material_drivers(dna).into_iter().map(|(_, set)| set).collect();
            scene.props.light_drivers = self.light_drivers(dna);
            scene.props.world_drivers = self.world_drivers(dna);
            scene.props.modifier_drivers = self.modifier_drivers(&scene.objects, dna);
            scene.props.rigs = self.rigs(&scene.objects, dna, &actions);
            // Objects that follow others — by a constraint or a bone parent
            // — stand where their targets put them at the saved frame
            // (`BKE_object_where_is_calc`), not where their parents alone
            // would.
            if scene.props.motion.follows() {
                let at = scene.props.motion.solve(scene.props.frame_current as f32, &scene.props.rigs.rigs, false);
                for (object, world) in scene.objects.iter_mut().zip(at.world) {
                    object.matrix_world = world;
                }
            }
            if !progress(0.98, "Reading NLA") {
                return Err(crate::PointCloudError::Cancelled);
            }
            scene.props.nla = self.nla_list(&scene.objects, dna);
            if !progress(0.99, "Reading drivers") {
                return Err(crate::PointCloudError::Cancelled);
            }
            scene.props.drivers = self.drivers_list(&scene.objects, dna);
        }
        Ok(scene)
    }

    /// Resolve the `SC` block that `FileGlobal.curscene` points at (Blender
    /// `read_global`). `None` when there is no GLOB block, the field cannot
    /// be located in this file's DNA, or the pointer does not resolve.
    pub(crate) fn file_global_curscene(&self, dna: &DnaType) -> Option<&BlendBlock> {
        let ps = self.header.pointer_size as usize;
        let le = self.header.little_endian;
        let glob = self.blocks.iter().find(|b| b.code_str == "GLOB")?;
        let off = Self::struct_field_offset("FileGlobal", "*curscene", dna, ps)?;
        let addr = Self::read_ptr(&glob.data, off, ps, le);
        self.block_at_code(addr, "SC")
    }

    /// Byte offset of `ID.name` inside any ID block, from this file's DNA.
    /// `ID` is always the first member of an ID struct, so the offset within
    /// `ID` is the offset within the block.
    pub(crate) fn id_name_offset(&self, dna: &DnaType) -> Option<usize> {
        let ps = self.header.pointer_size as usize;
        Self::struct_field_offset("ID", "name[", dna, ps)
    }
}
