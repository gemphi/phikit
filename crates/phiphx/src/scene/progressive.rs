//! Progressive open: build the base scene now, finish the modifiers later.
//!
//! [`SceneModel::from_blend_progressive`] returns the base meshes immediately
//! (as if no object had a modifier) plus a [`DeferredModifiers`] carrying the
//! captured modifier work. The host shows the base scene, runs
//! [`DeferredModifiers::finish`] on a worker thread, and swaps the evaluated
//! objects it returns into the scene — each at its `objects` index.
//!
//! `finish()` produces exactly what an eager open would: it runs the same
//! apply (`apply_eval_jobs`) and the same conversion (`evaluated_object`), so
//! `base + finish()` equals the eager result object for object.

use super::object::evaluated_object;
use super::{SceneModel, SceneObject};
use crate::blend::{BlendScene, BlendSceneObject, DeferredRaw, apply_eval_jobs};
use std::sync::Arc;

/// The modifier work of a progressively-opened scene, not yet applied. Owned
/// and `Send`, so it travels to a worker thread.
pub struct DeferredModifiers {
    raw: Arc<DeferredRaw>,
    /// One entry per modified mesh object: its `objects` index, its raw index
    /// (the key in the evaluated map), and the raw object the conversion needs.
    targets: Vec<(usize, usize, BlendSceneObject)>,
}

impl DeferredModifiers {
    /// Nothing to do — no object in the scene carries a modifier.
    pub fn is_empty(&self) -> bool {
        self.targets.is_empty()
    }

    /// Apply the modifiers and build the evaluated objects. Returns
    /// `(objects index, evaluated SceneObject)` for every modified object, for
    /// the host to splice into the base scene's `objects`.
    pub fn finish(self) -> Vec<(usize, SceneObject)> {
        // `raw` is held only here (taken out of the `BlendScene`), so unwrap
        // succeeds; if it were somehow shared, there is simply nothing to swap.
        let Ok(raw) = Arc::try_unwrap(self.raw) else { return Vec::new() };
        let evaluated = apply_eval_jobs(raw);
        self.targets
            .into_iter()
            .filter_map(|(scene_index, raw_index, src)| {
                evaluated
                    .get(&raw_index)
                    .map(|mesh| (scene_index, evaluated_object(&src, Some(mesh))))
            })
            .collect()
    }
}

impl SceneModel {
    /// Splice finished (modifier-evaluated) objects back into the scene — each
    /// at its `objects` index. The evaluated object keeps the base object's
    /// resolved `materials` (a modifier changes geometry, not which textures a
    /// slot resolved to), so the result matches an eager open that ran
    /// `texture::attach`, without re-resolving textures here.
    pub fn apply_deferred(&mut self, swaps: Vec<(usize, SceneObject)>) {
        for (i, mut obj) in swaps {
            let Some(base) = self.objects.get(i) else { continue };
            obj.materials = base.materials.clone();
            self.objects[i] = obj;
        }
        // The report counted the base meshes at read; the evaluated ones are
        // heavier, so refresh it to follow them (matching an eager open).
        self.refresh_report();
    }

    /// Recompute the geometry counts (`verts`, `tris`) of [`Self::report`] from
    /// the current `objects`, so the status bar and the read summary report the
    /// geometry actually shown once the modifiers have evaluated — the same
    /// figures an eager open prints. `faces` is the file's stored polygon count
    /// (as in the eager report) and does not change with evaluation; a rig's
    /// deform moves vertices without adding any, so it leaves these counts
    /// untouched too — only the generative modifiers in the stack change them.
    pub fn refresh_report(&mut self) {
        self.report.verts = self.objects.iter().map(|o| o.mesh.vertices.len()).sum();
        self.report.tris = self.objects.iter().map(|o| o.mesh.indices.len() / 3).sum();
    }

    /// Build the base scene (no modifiers applied) and, when the file was read
    /// progressively (`BlendScene::deferred` set), the deferred modifier work.
    /// An eager scene (no `deferred`) returns `None` — its `props.evaluated` is
    /// already baked into the objects by [`SceneModel::from_blend_owned`].
    pub fn from_blend_progressive(
        mut blend: BlendScene,
        source_path: &str,
    ) -> (SceneModel, Option<DeferredModifiers>) {
        let raw = blend.deferred.take();
        // The `objects` index of each blend object, by the same rule the base
        // build uses (cameras / lights / empties are not objects). A modifier
        // job is always a mesh, so it always maps.
        let mut scene_index = vec![None; blend.objects.len()];
        let mut k = 0usize;
        for (j, o) in blend.objects.iter().enumerate() {
            if matches!(o.object_type.as_str(), "CAMERA" | "LAMP" | "LIGHT" | "EMPTY") {
                continue;
            }
            scene_index[j] = Some(k);
            k += 1;
        }
        let targets: Vec<(usize, usize, BlendSceneObject)> = raw
            .as_ref()
            .map(|raw| {
                raw.jobs
                    .iter()
                    .filter_map(|job| {
                        scene_index
                            .get(job.index)
                            .copied()
                            .flatten()
                            .map(|si| (si, job.index, blend.objects[job.index].clone()))
                    })
                    .collect()
            })
            .unwrap_or_default();

        // The base model: `props.evaluated` is empty on a progressive read, so
        // every object converts from its base mesh.
        let model = SceneModel::from_blend_owned(blend, source_path);
        let deferred = raw
            .filter(|r| !r.is_empty())
            .map(|raw| DeferredModifiers { raw, targets });
        (model, deferred)
    }
}
