//! Modifier work captured as owned jobs, so the apply can run off the read thread.
use super::*;

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
    /// `(instance, member)` object indices: an instance that draws the
    /// member's evaluated mesh — by sharing the member's converted
    /// geometry (`scene/share.rs`), so the evaluated map holds it once.
    pub aliases: Vec<(usize, usize)>,
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
    let DeferredRaw { jobs, worlds, prefix, aliases: _ } = raw;
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
