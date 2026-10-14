//! The viewport modifier stack (Blender `mesh_calc_modifiers`), limited to
//! modifiers that change how an existing mesh looks.
//!
//! Declarative: [`KINDS`] maps a DNA `ModifierData.type` to a reader that
//! turns the saved settings into a [`BModifier`]. The stack runs in file
//! order on an [`BlendEvalMesh`]; unknown or disabled modifiers are skipped
//! (and listed in the read report by the caller).

pub mod driver;
pub(crate) mod driver_read;
pub(crate) mod armature;
pub(crate) mod deform;
pub(crate) mod prefix;
#[cfg(test)]
mod prefix_tests;
pub(crate) mod bbone;
mod array;
mod displace;
mod geom;
mod mirror;
mod solidify;
pub mod skin;
mod subsurf;
mod topology;
mod triangulate;
mod weld;
mod lattice;
mod crease;
#[cfg(test)]
mod tests;

use crate::blend::file::types::*;
use glam::Mat4;
use std::collections::HashMap;

/// `eModifierMode_Realtime` — shown in the viewport.
const MODE_REALTIME: u32 = 1;

/// The mesh a modifier stack works on (object space).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BlendEvalMesh {
    pub vertices: Vec<[f32; 3]>,
    pub faces: crate::blend::BFaces,
    /// Per-corner UVs in face-corner order (empty when the mesh has none).
    pub loop_uvs: Vec<[f32; 2]>,
    /// The colour attribute per corner, in the same order (empty when the
    /// mesh has none); every modifier carries it as it carries the UVs.
    pub loop_colors: Vec<[f32; 3]>,
    pub face_materials: Vec<u16>,
    pub face_smooth: Vec<bool>,
    /// Per-vertex `(group, weight)`; carried through topology changes so a
    /// later Armature still deforms (Blender interpolates `MDeformVert`).
    pub dverts: Vec<Vec<(u32, f32)>>,
    /// Creased edges `(a, b, crease)`, `a < b`, crease `0..1`
    /// (`crease_edge`), which the Subdivision Surface sharpens by.
    pub edge_creases: Vec<(u32, u32, f32)>,
    /// Vertex creases (`crease_vert`), one per vertex or empty.
    pub vert_creases: Vec<f32>,
}

/// What a modifier may look up while evaluating.
pub(crate) struct EvalCtx<'a> {
    /// The evaluated object's world matrix.
    pub world: Mat4,
    /// World matrices of all scene objects by saved address.
    pub worlds: &'a HashMap<u64, Mat4>,
}

pub(crate) trait BModifier: Send + Sync {
    fn apply(&self, mesh: &mut BlendEvalMesh, ctx: &EvalCtx);
    /// Whether the stack must load vertex-group weights for this modifier.
    fn wants_weights(&self) -> bool {
        false
    }
    /// As a batched skin job instead of `apply` (only asked of the last modifier).
    /// Test hook: is this the lattice deform?
    #[cfg(test)]
    fn is_lattice(&self) -> bool {
        false
    }
    fn skin_job(&self, _mesh: &BlendEvalMesh, _ctx: &EvalCtx) -> Option<skin::SkinJob> {
        None
    }
    /// An Armature modifier's rig object and vertex-group names.
    fn armature_source(&self) -> Option<(u64, &[String])> {
        None
    }
}

/// A mesh an Armature modifier deforms, kept so its rig can pose it at any
/// frame (`scene/rig.rs`): the mesh the Armature modifier receives (the
/// modifiers before it applied, weights loaded), its vertex groups, the rig
/// object, and the modifiers after it.
pub struct BlendSkinSource {
    /// Index into the scene's objects.
    pub object: usize,
    /// The rig object's block address.
    pub armature: u64,
    pub base: BlendEvalMesh,
    pub groups: Vec<String>,
    pub(crate) tail: Vec<Box<dyn BModifier>>,
}

impl std::fmt::Debug for BlendSkinSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlendSkinSource").field("object", &self.object).field("armature", &self.armature).field("tail", &self.tail.len()).finish()
    }
}

impl BlendSkinSource {
    /// The modifiers after the Armature over `deformed` (the base's
    /// vertices, posed) — the mesh the viewport draws at that pose.
    pub fn evaluate(&self, deformed: Vec<[f32; 3]>, world: Mat4) -> BlendEvalMesh {
        if self.tail.is_empty() {
            return BlendEvalMesh { vertices: deformed, ..Default::default() };
        }
        // Positions only: the UVs, materials and weights the modifiers would
        // carry along are the ones read with the mesh.
        let mut mesh = BlendEvalMesh {
            vertices: deformed,
            faces: self.base.faces.clone(),
            edge_creases: self.base.edge_creases.clone(),
            vert_creases: self.base.vert_creases.clone(),
            ..Default::default()
        };
        let worlds = HashMap::new();
        let ctx = EvalCtx { world, worlds: &worlds };
        for m in &self.tail {
            m.apply(&mut mesh, &ctx);
        }
        mesh
    }
}

type Reader = fn(&BlendFile, &BlendBlock, &BlendBlock, &DnaType) -> Option<Box<dyn BModifier>>;

/// One supported modifier type.
pub(crate) struct BModifierKind {
    pub dna_type: u32,
    pub name: &'static str,
    pub read: Reader,
}

/// Supported modifiers, by `eModifierType`.
pub(crate) const KINDS: &[BModifierKind] = &[
    BModifierKind { dna_type: 8, name: "ARMATURE", read: armature::read },
    BModifierKind { dna_type: 1, name: "SUBSURF", read: subsurf::read },
    BModifierKind { dna_type: 5, name: "MIRROR", read: mirror::read },
    BModifierKind { dna_type: 12, name: "ARRAY", read: array::read },
    BModifierKind { dna_type: 44, name: "TRIANGULATE", read: triangulate::read },
    BModifierKind { dna_type: 33, name: "SOLIDIFY", read: solidify::read },
    BModifierKind { dna_type: 14, name: "DISPLACE", read: displace::read },
    BModifierKind { dna_type: 55, name: "WELD", read: weld::read },
    BModifierKind { dna_type: 2, name: "LATTICE", read: lattice::read },
];

/// Names of the modifiers the viewport evaluates (Blender's RNA type ids).
pub fn supported_modifiers() -> impl Iterator<Item = &'static str> {
    KINDS.iter().map(|k| k.name)
}

impl BlendFile {
    /// Enabled viewport modifiers of `ob`, in stack order (supported ones).
    fn modifier_stack(&self, ob: &BlendBlock, dna: &DnaType) -> Vec<Box<dyn BModifier>> {
        let first = self.ptr(ob, self.off("Object", "modifiers", dna));
        let next = self.off("ModifierData", "*next", dna);
        let mds = self.scoped(ob, || self.list(first, next));
        mds.into_iter()
            .filter(|md| self.int_field(md, "ModifierData", "mode", dna) & MODE_REALTIME != 0)
            .filter_map(|md| {
                let ty = self.int_field(md, "ModifierData", "type", dna);
                let kind = KINDS.iter().find(|k| k.dna_type == ty)?;
                self.scoped(ob, || (kind.read)(self, md, ob, dna))
            })
            .collect()
    }

    /// Linked-list walk from a `ListBase.first` address.
    pub(crate) fn list(&self, first: u64, next: Option<usize>) -> Vec<&BlendBlock> {
        let mut out = Vec::new();
        let mut addr = first;
        while let Some(b) = self.block_at(addr) {
            if out.len() > 100_000 { break; }
            out.push(b);
            addr = self.ptr(b, next);
        }
        out
    }

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
        for (i, o) in objects.iter().enumerate() {
            if o.object_type != "MESH" || o.vertices.is_empty() {
                continue;
            }
            let Some(ob) = self.block_at_code(o.block_address, "OB") else { continue };
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
            };
            jobs.push(BEvalJob { index: i, mesh, stack, world: Mat4::from_cols_array_2d(&o.matrix_world), weights: dverts_loaded });
        }
        DeferredRaw { jobs, worlds, prefix: self.prefix_box.clone() }
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
            jobs.push((i, o, stack, tail, rig, groups.to_vec(), dverts, creases));
        }
        use rayon::prelude::*;
        // The same drop box the stack apply uses (`prefix.rs`): `stack` here
        // is exactly the prefix it splits off, so one of the two applies it
        // and the other takes the result.
        let prefix = self.prefix_box.clone();
        jobs.into_par_iter()
            .map(|(i, o, stack, tail, rig, groups, dverts, creases)| {
                // `skin_sources` always loads the weights, so `weights` is
                // true on this side by construction.
                let key = (prefix::sharing() && !stack.is_empty())
                    .then_some(prefix::BPrefixKey { object: i, applied: stack.len(), weights: true });
                if let Some(base) = key.and_then(|k| prefix.take(k)) {
                    return BlendSkinSource { object: i, armature: rig, base, groups, tail };
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
                };
                let ctx = EvalCtx { world: Mat4::from_cols_array_2d(&o.matrix_world), worlds: &worlds };
                for m in &stack {
                    m.apply(&mut base, &ctx);
                }
                if let Some(k) = key {
                    prefix.put(k, &base);
                }
                BlendSkinSource { object: i, armature: rig, base, groups, tail }
            })
            .collect()
    }

    /// Run all pending skin jobs in one batch and write the results back.
    fn finish_skins(
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

/// One mesh object's modifier work, captured so the apply can run off the read
/// thread (progressive open). Owned and `Send` — the base mesh, the stack and
/// the object's world matrix.
pub(crate) struct BEvalJob {
    /// Index into the read scene's `objects` (the `evaluated` map's key).
    pub index: usize,
    pub mesh: BlendEvalMesh,
    pub stack: Vec<Box<dyn BModifier>>,
    pub world: Mat4,
    /// Whether `mesh` was built with the object's vertex-group weights, which
    /// decides whether its prefix can stand in for `skin_sources`'s `base`
    /// (`prefix.rs`, [`prefix::BPrefixKey`]).
    pub weights: bool,
}

/// Every object's captured modifier jobs plus the shared world matrices — all
/// the deferred apply needs, and nothing that borrows the file.
pub(crate) struct DeferredRaw {
    pub jobs: Vec<BEvalJob>,
    pub worlds: HashMap<u64, Mat4>,
    /// Shared with the file, so the prefix each armature-deformed mesh
    /// reaches is applied once however the two consumers end up ordered
    /// (`prefix.rs`). Held by `Arc` because a progressive open applies these
    /// jobs on a worker thread, long after the read that made them.
    pub prefix: std::sync::Arc<prefix::BPrefixBox>,
}

impl DeferredRaw {
    pub fn is_empty(&self) -> bool {
        self.jobs.is_empty()
    }
}

// The jobs hold modifier trait objects, which are not `Debug`; `BlendScene`
// derives `Debug`, so report the shape rather than the contents.
impl std::fmt::Debug for DeferredRaw {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DeferredRaw {{ jobs: {} }}", self.jobs.len())
    }
}

/// Apply captured modifier jobs, in parallel, to their evaluated meshes —
/// keyed by object index, exactly as [`BlendFile::evaluate_modifiers`] would.
/// Takes no reader, so a progressive open can call it on a worker thread after
/// the base scene is already on screen.
pub(crate) fn apply_eval_jobs(raw: DeferredRaw) -> HashMap<usize, BlendEvalMesh> {
    use rayon::prelude::*;
    let DeferredRaw { jobs, worlds, prefix } = raw;
    let batch = skin::skinner();
    let evaluated: Vec<(usize, BlendEvalMesh, Option<skin::SkinJob>)> = jobs
        .into_par_iter()
        .map(|job| {
            let BEvalJob { index, mut mesh, stack, world, weights } = job;
            let ctx = EvalCtx { world, worlds: &worlds };
            // Everything up to this stack's first Armature is the same mesh
            // `skin_sources` builds (`prefix.rs`). Whichever of the two runs
            // first applies it and leaves a copy; this is both halves of that
            // deal, since which one runs first depends on the open.
            let split = prefix::sharing()
                .then(|| prefix::split_at(&stack).filter(|&k| k > 0))
                .flatten()
                .map(|k| prefix::BPrefixKey { object: index, applied: k, weights });
            let mut from = 0;
            if let Some(key) = split {
                match prefix.take(key) {
                    Some(done) => {
                        mesh = done;
                        from = key.applied;
                    }
                    None => {
                        for m in &stack[..key.applied] {
                            m.apply(&mut mesh, &ctx);
                        }
                        prefix.put(key, &mesh);
                        from = key.applied;
                    }
                }
            }
            let (last, head) = stack.split_last().expect("non-empty stack");
            for m in head.iter().skip(from) {
                m.apply(&mut mesh, &ctx);
            }
            let skinjob = batch.as_ref().and_then(|_| last.skin_job(&mesh, &ctx));
            if skinjob.is_none() {
                last.apply(&mut mesh, &ctx);
            }
            (index, mesh, skinjob)
        })
        .collect();
    BlendFile::finish_skins(evaluated, batch.as_deref())
}
