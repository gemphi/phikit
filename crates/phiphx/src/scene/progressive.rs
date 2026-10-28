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
//! `base + finish()` equals the eager result object for object — and pose for
//! pose: a mesh its rig deforms is drawn at every frame with the triangles it
//! was evaluated into (`rig.rs`), and those are only known here, so the swap
//! hands them to the rig as well.

use super::object::{STriangles, evaluated_object_and_tris};
use super::share::shared_object;
use super::progressive_links::{parent_links, vertex_parent_of};
use super::{SceneModel, SceneObject};
use crate::blend::{BlendScene, BlendSceneObject, DeferredRaw, apply_eval_jobs, vertex_children_worlds};
use std::sync::Arc;

/// The modifier work of a progressively-opened scene, not yet applied. Owned
/// and `Send`, so it travels to a worker thread.
pub struct DeferredModifiers {
    raw: Arc<DeferredRaw>,
    /// One entry per modified mesh object: its `objects` index, its raw index
    /// (the key in the evaluated map), the raw object the conversion needs,
    /// and whether a rig deforms it (its triangles then go to the rig too).
    targets: Vec<(usize, usize, BlendSceneObject, bool)>,
    /// One entry per collection instance of a modified member: its
    /// `objects` index, the member's raw index, and the raw instance — it
    /// takes the member's evaluated geometry by `Arc` (`share.rs`), not a
    /// conversion of its own.
    aliases: Vec<(usize, usize, BlendSceneObject)>,
    /// Every blend object's parent links (transforms only, no geometry;
    /// the saved vertices of a vertex parent with no modifier job), so the
    /// vertex-parented children can be re-solved against the evaluated
    /// meshes when they land (`BlendFile::reparent_to_vertices` does the
    /// same on an eager open). Empty when no object is vertex-parented.
    links: Vec<BlendSceneObject>,
}

/// One object the deferred modifiers finished, for
/// [`SceneModel::apply_deferred`] to put in place.
#[derive(Debug)]
pub struct SceneSwap {
    /// Its index in `objects`.
    pub index: usize,
    /// The object with its modifier stack applied.
    pub object: SceneObject,
    /// For a mesh a rig deforms, the triangles its evaluated mesh was cut
    /// into — what every pose of it is drawn with. `None` otherwise.
    tris: Option<STriangles>,
    /// `(blend object index, world)` of the vertex-parented children (and
    /// their descendants) that hang from this object's evaluated vertices
    /// — placed by `give_parvert` only once the mesh is evaluated.
    moved: Vec<(usize, [[f32; 4]; 4])>,
}

impl DeferredModifiers {
    /// Nothing to do — no object in the scene carries a modifier.
    pub fn is_empty(&self) -> bool {
        self.targets.is_empty()
    }

    /// Apply the modifiers and build the evaluated objects: one
    /// [`SceneSwap`] per modified object, for the host to splice into the
    /// base scene with [`SceneModel::apply_deferred`].
    pub fn finish(self) -> Vec<SceneSwap> {
        // `raw` is held only here (taken out of the `BlendScene`), so unwrap
        // succeeds; if it were somehow shared, there is simply nothing to swap.
        let Ok(raw) = Arc::try_unwrap(self.raw) else { return Vec::new() };
        let evaluated = apply_eval_jobs(raw);
        let links = &self.links;
        let mut moved = vertex_children_worlds(links, &|p| evaluated.get(&p).map_or(links[p].vertices.as_slice(), |m| m.vertices.as_slice()));
        let mut swaps: Vec<(usize, SceneSwap)> = self
            .targets
            .into_iter()
            .filter_map(|(index, raw_index, src, skinned)| {
                let (object, tris) = evaluated_object_and_tris(&src, Some(evaluated.get(&raw_index)?), skinned);
                // The children re-solved against THIS evaluated mesh.
                let (mine, rest): (Vec<_>, Vec<_>) = std::mem::take(&mut moved).into_iter().partition(|&(j, _)| vertex_parent_of(links, j) == Some(raw_index));
                moved = rest;
                Some((raw_index, SceneSwap { index, object, tris, moved: mine }))
            })
            .collect();
        // Each instance of an evaluated member shares the member's swap: the
        // same `Arc<SceneMesh>`, converted once (`share.rs`).
        let shared: Vec<SceneSwap> = self
            .aliases
            .iter()
            .filter_map(|(index, member, src)| {
                let from = &swaps.iter().find(|(raw_index, _)| raw_index == member)?.1.object;
                Some(SceneSwap { index: *index, object: shared_object(src, from), tris: None, moved: Vec::new() })
            })
            .collect();
        swaps.drain(..).map(|(_, swap)| swap).chain(shared).collect()
    }
}

impl SceneModel {
    /// Splice finished (modifier-evaluated) objects back into the scene — each
    /// at its `objects` index. The evaluated object keeps the base object's
    /// resolved `materials` (a modifier changes geometry, not which textures a
    /// slot resolved to), so the result matches an eager open that ran
    /// `texture::attach`, without re-resolving textures here.
    ///
    /// A mesh a rig deforms takes its evaluated triangles into its skin as
    /// well. Its skin was made with the base read's — the mesh as saved, no
    /// Mirror, no Subdivision — and every pose is drawn over those, so
    /// without this a rigged figure lost its mirrored half (and its
    /// subdivision) the moment the viewport posed it.
    pub fn apply_deferred(&mut self, swaps: Vec<SceneSwap>) {
        for SceneSwap { index, mut object, tris, moved } in swaps {
            let Some(base) = self.objects.get(index) else { continue };
            object.materials = base.materials.clone();
            self.objects[index] = object;
            if let Some(tris) = tris {
                self.motion.rigs.retriangulate(index, tris);
            }
            for (j, world) in moved {
                self.place(j, world);
            }
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
        // The blend objects a rig deforms, which the base build gives skins.
        let skinned: std::collections::HashSet<usize> = blend.props.rigs.skins.iter().map(|s| s.object).collect();
        let targets: Vec<(usize, usize, BlendSceneObject, bool)> = raw
            .as_ref()
            .map(|raw| {
                raw.jobs
                    .iter()
                    .map(|job| job.index)
                    .filter_map(|index| {
                        scene_index
                            .get(index)
                            .copied()
                            .flatten()
                            .map(|si| (si, index, blend.objects[index].clone(), skinned.contains(&index)))
                    })
                    .collect()
            })
            .unwrap_or_default();
        // A collection instance sharing its member's evaluated mesh
        // (`DeferredRaw::aliases`) is swapped in with the member's `Arc`s.
        let aliases: Vec<(usize, usize, BlendSceneObject)> = raw
            .as_ref()
            .map(|raw| {
                raw.aliases
                    .iter()
                    .filter_map(|&(instance, member)| {
                        let si = scene_index.get(instance).copied().flatten()?;
                        Some((si, member, blend.objects[instance].clone()))
                    })
                    .collect()
            })
            .unwrap_or_default();

        let links = parent_links(&blend.objects, raw.as_deref());
        // The base model: `props.evaluated` is empty on a progressive read, so
        // every object converts from its base mesh.
        let model = SceneModel::from_blend_owned(blend, source_path);
        let deferred = raw
            .filter(|r| !r.is_empty())
            .map(|raw| DeferredModifiers { raw, targets, aliases, links });
        (model, deferred)
    }
}
