//! Evaluating a scene's modifier stacks: the read, the captured jobs and the skin batches.
use crate::device::blend::modifier::*;

impl BlendFile {
    /// Evaluated meshes for every mesh object whose stack changes it, keyed
    /// by index into `objects`. Objects without modifiers are left out.
    ///
    /// The read (sequential, pointer-scoped) and the apply (parallel) are
    /// split so a progressive open can run the apply off the read thread:
    /// [`Self::modifier_jobs`] captures the work here, [`apply_eval_jobs`]
    /// performs it anywhere.
    pub(crate) fn evaluate_modifiers(&self, objects: &[BlendSceneObject], dna: &DnaType) -> HashMap<usize, BlendEvalMesh> {
        apply_eval_jobs(self.modifier_jobs(objects, dna))
    }

    /// Capture every mesh object's modifier work as owned, `Send` jobs — the
    /// base mesh, the stack and the world matrices — without applying it. This
    /// is the part that must read the file (pointer scoping), so it stays on
    /// the read thread; the heavy apply ([`apply_eval_jobs`]) can wait.
    pub(crate) fn modifier_jobs(&self, objects: &[BlendSceneObject], dna: &DnaType) -> DeferredRaw {
        let worlds: HashMap<u64, Mat4> = objects
            .iter()
            .map(|o| (o.block_address, Mat4::from_cols_array_2d(&o.matrix_world)))
            .collect();
        let mut jobs = Vec::new();
        // A collection instance draws its member's evaluated geometry, and
        // Blender evaluates the member once whatever number of instances
        // show it: each later instance of an evaluated member shares that
        // result (`aliases`, `scene/share.rs`) rather than a stack of its own.
        let mut by_block: HashMap<u64, usize> = HashMap::new();
        let mut aliases = Vec::new();
        for (i, o) in objects.iter().enumerate() {
            if !matches!(o.object_type.as_str(), "MESH" | "FONT" | "CURVE") || o.vertices.is_empty() {
                continue;
            }
            let (address, world) = match o.instance_source {
                Some((member, world)) => (member, world),
                None => (o.block_address, o.matrix_world),
            };
            if let Some(&first) = by_block.get(&address) {
                aliases.push((i, first));
                continue;
            }
            let Some(ob) = self.block_at_code(address, "OB") else { continue };
            let mut stack = self.modifier_stack(ob, dna);
            // `PHIPHX_MODIFIERS=<n>` keeps only the first `n` of every stack.
            // Parity work compares one stage at a time against Blender with
            // the same modifiers muted; nothing in the app sets it.
            if let Some(n) = std::env::var("PHIPHX_MODIFIERS").ok().and_then(|v| v.parse::<usize>().ok()) {
                stack.truncate(n);
            }
            if stack.is_empty() {
                continue;
            }
            let me = self.mesh_block_of(ob, dna);
            let dverts_loaded = stack.iter().any(|m| m.wants_weights()) && me.is_some();
            let dverts = match (dverts_loaded, me) {
                (true, Some(me)) => self.deform_verts(me, dna, o.vertices.len()),
                _ => Vec::new(),
            };
            let creases = me.map(|me| self.mesh_creases(me, dna, o.vertices.len())).unwrap_or_default();
            let loose = me.map(|me| self.mesh_loose_edges(me, dna, &o.faces, o.vertices.len())).unwrap_or_default();
            let sharp_edges = me.map(|me| self.mesh_sharp_edges(me, dna, o.vertices.len())).unwrap_or_default();
            let mesh = BlendEvalMesh {
                vertices: o.vertices.clone(),
                faces: o.faces.clone(),
                loop_uvs: o.loop_uvs.clone(),
                loop_colors: o.loop_colors.clone(),
                face_materials: o.face_materials.clone(),
                face_smooth: o.face_smooth.clone(),
                dverts,
                edge_creases: creases.edge,
                vert_creases: creases.vert,
                loose_edges: loose,
                sharp_edges,
            };
            by_block.insert(address, i);
            jobs.push(BEvalJob { index: i, mesh, stack, world: Mat4::from_cols_array_2d(&world), weights: dverts_loaded });
        }
        DeferredRaw { jobs, worlds, prefix: self.prefix_box.clone(), aliases }
    }

    /// Every mesh object an Armature modifier deforms (the first one in its
    /// stack), split at that modifier.
    pub(crate) fn skin_sources(&self, objects: &[BlendSceneObject], dna: &DnaType) -> Vec<BlendSkinSource> {
        let worlds: HashMap<u64, Mat4> = objects.iter().map(|o| (o.block_address, Mat4::from_cols_array_2d(&o.matrix_world))).collect();
        // Reading stays sequential (pointer scoping); the stacks under the
        // armatures — a subdivision surface per clothed mesh, for a dressed
        // figure — are applied in parallel, as `evaluate_modifiers` applies
        // its (Blender evaluates its depsgraph's objects on all threads).
        let mut jobs = Vec::new();
        for (i, o) in objects.iter().enumerate() {
            if o.object_type != "MESH" || o.vertices.is_empty() {
                continue;
            }
            let Some(ob) = self.block_at_code(o.block_address, "OB") else { continue };
            let mut stack = self.modifier_stack(ob, dna);
            let Some(k) = stack.iter().position(|m| m.armature_source().is_some()) else { continue };
            let Some(me) = self.mesh_block_of(ob, dna) else { continue };
            let tail = stack.split_off(k + 1);
            let armature = stack.pop().expect("the armature modifier");
            let Some((rig, groups)) = armature.armature_source() else { continue };
            let dverts = self.deform_verts(me, dna, o.vertices.len());
            let creases = self.mesh_creases(me, dna, o.vertices.len());
            let loose = self.mesh_loose_edges(me, dna, &o.faces, o.vertices.len());
            let sharp_edges = self.mesh_sharp_edges(me, dna, o.vertices.len());
            jobs.push((i, o, stack, tail, rig, groups.to_vec(), dverts, creases, loose, sharp_edges));
        }
        use rayon::prelude::*;
        // The same drop box the stack apply uses (`prefix.rs`): `stack` here
        // is exactly the prefix it splits off, so one of the two applies it
        // and the other takes the result.
        let prefix = self.prefix_box.clone();
        jobs.into_par_iter()
            .map(|(i, o, stack, tail, rig, groups, dverts, creases, loose, sharp_edges)| {
                // `skin_sources` always loads the weights, so `weights` is
                // true on this side by construction.
                let key = (prefix::sharing() && !stack.is_empty())
                    .then_some(prefix::BPrefixKey { object: i, applied: stack.len(), weights: true });
                if let Some(base) = key.and_then(|k| prefix.take(k)) {
                    return BlendSkinSource { object: i, object_address: o.block_address, armature: rig, base, groups, tail };
                }
                let mut base = BlendEvalMesh {
                    vertices: o.vertices.clone(),
                    faces: o.faces.clone(),
                    loop_uvs: o.loop_uvs.clone(),
                    loop_colors: o.loop_colors.clone(),
                    face_materials: o.face_materials.clone(),
                    face_smooth: o.face_smooth.clone(),
                    dverts,
                    edge_creases: creases.edge,
                    vert_creases: creases.vert,
                    loose_edges: loose,
                    sharp_edges,
                };
                let ctx = EvalCtx { world: Mat4::from_cols_array_2d(&o.matrix_world), worlds: &worlds };
                for m in &stack {
                    m.apply(&mut base, &ctx);
                }
                if let Some(k) = key {
                    prefix.put(k, &base);
                }
                BlendSkinSource { object: i, object_address: o.block_address, armature: rig, base, groups, tail }
            })
            .collect()
    }

    /// Run all pending skin jobs in one batch and write the results back.
    pub(super) fn finish_skins(
        evaluated: Vec<(usize, BlendEvalMesh, Option<skin::SkinJob>)>,
        batch: Option<&dyn skin::Skinner>,
    ) -> HashMap<usize, BlendEvalMesh> {
        let (mut meshes, mut jobs): (Vec<_>, Vec<_>) = (Vec::new(), Vec::new());
        for (i, mesh, job) in evaluated {
            if let Some(j) = job {
                jobs.push(j);
            }
            meshes.push((i, mesh, jobs.len()));
        }
        if !jobs.is_empty() && !batch.is_some_and(|s| s.run(&mut jobs)) {
            skin::Skinner::run(&skin::CpuSkin, &mut jobs);
        }
        let mut prev = 0;
        meshes
            .into_iter()
            .map(|(i, mut mesh, upto)| {
                if upto > prev {
                    mesh.vertices = std::mem::take(&mut jobs[upto - 1].out);
                    prev = upto;
                }
                (i, mesh)
            })
            .collect()
    }
}
