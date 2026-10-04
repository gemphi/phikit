//! Scoped Scene_Source context — threads the cached loaded [`SceneModel`]
//! through a single render pass to BOTH the Viewport and the Outliner without
//! changing the ~40 `BViewItem::build` / `BViewList::items` signatures.
//!
//! Why a scoped thread-local rather than a parameter: the two Scene_Source
//! consumers reach the scene at different points in one render pass. The
//! Outliner rows are built into the `BView` tree during `BSpaceList::items(&state)`
//! (a trait walk over many `BViewItem::build(&self, &BState)` implementors),
//! while the Viewport is produced later by the gpui renderer via `BRenderCtx`
//! (which only carries `&BState`). Neither seam carries a `SceneModel`, and
//! widening those ~40 signatures to add one is far more invasive than a
//! render-pass-scoped context that both seams can read.
//!
//! `BPhiberApp::render` sets the scope from `scene_cache` at the top of the
//! pass and clears it at the bottom, so the Outliner build and the Viewport
//! walk (both run synchronously between) observe the same model. When the
//! scope is unset (the default, and after `clear_loaded_scene`) the source
//! yields the Default_Scene, so nothing leaks into the next pass.
//!
//! The thread-local stores an **owned** [`SceneModel`] clone (one shallow
//! `Vec` clone per render when a scene is loaded) rather than a borrow or raw
//! pointer, which sidesteps all lifetime/unsafe concerns for the cost of a
//! single clone per loaded-scene render.

use std::cell::RefCell;
use std::rc::Rc;

use crate::phiphx::scene::SceneModel;
use crate::v2::scene::BObject3D;
use crate::v2::state::BState;

thread_local! {
    /// The loaded scene model in scope for the current render pass, or `None`
    /// when no blend is loaded / the scope has been cleared.
    ///
    /// Shared, not cloned: this is set on every render pass, and a `.blend`
    /// with thousands of objects made that one deep copy of the whole model
    /// per frame — paid again on every resize tick and every space change.
    /// An `Arc` makes it a refcount bump.
    static LOADED: RefCell<Option<std::sync::Arc<SceneModel>>> = const { RefCell::new(None) };

    /// The object list built from whichever model was last in scope, kept so
    /// the several readers in one render pass share one build.
    ///
    /// The entry holds a strong reference to the model it was built from, so
    /// the pointer it is keyed on cannot be reused by a different scene while
    /// the entry is alive — an address comparison is sound here only because
    /// of that.
    static BUILT: RefCell<Option<(Option<std::sync::Arc<SceneModel>>, (Option<f32>, Option<f32>, u64), Rc<Vec<BObject3D>>)>> =
        const { RefCell::new(None) };
}

thread_local! {
    /// How many times this thread has built the object list. The readers in
    /// one render pass must share one build; only a new model in scope may
    /// move it. Per-thread, like the memo it counts.
    static OBJECT_BUILDS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// The build count — what a frame really paid to read the scene.
pub fn object_builds() -> usize {
    OBJECT_BUILDS.with(std::cell::Cell::get)
}

/// Is `built` the list for `model`?
fn same_model(
    built: &Option<std::sync::Arc<SceneModel>>,
    model: &Option<std::sync::Arc<SceneModel>>,
) -> bool {
    match (built, model) {
        (None, None) => true,
        (Some(a), Some(b)) => std::sync::Arc::ptr_eq(a, b),
        _ => false,
    }
}

/// Set the loaded scene model in scope for the current render pass. Pass
/// `Some(model)` — an owned `SceneModel` or a shared `Arc` from
/// `scene_cache` — to render the loaded objects, or `None` to render the
/// Default_Scene. Called at the top of `BPhiberApp::render`, which passes the
/// `Arc` so the frame costs a refcount bump rather than a copy of the scene.
pub fn set_loaded_scene(model: Option<SceneModel>) {
    set_loaded_shared(model.map(std::sync::Arc::new));
}

/// Put an already-shared model in scope. This is what the render pass uses:
/// the app holds the scene behind an `Arc`, so a frame costs a refcount bump
/// instead of a copy of every object, collection and action in the file.
pub fn set_loaded_shared(model: Option<std::sync::Arc<SceneModel>>) {
    LOADED.with(|cell| *cell.borrow_mut() = model);
}

/// Set the loaded model and the frame it is shown at for this render pass.
pub fn set_loaded_at(model: Option<std::sync::Arc<SceneModel>>, state: &BState) {
    // The file's model with the session's actions in (`scene::action`).
    let model = model.map(|m| super::action::set::BActionScene::over(m, state));
    let frame = super::motion::shown_frame(state);
    let moved = model.as_deref().and_then(|m| super::transform::BTransforms::now(state).extra_worlds(state, m, frame));
    set_loaded_shared(model);
    super::motion::set_shown_frame(state);
    super::motion::set_moved(moved);
    super::render_set::BResolution::scope(state);
}

/// Clear the loaded scene scope so it does not leak into the next render pass.
/// Called at the bottom of `BPhiberApp::render`, after the tree build and the
/// renderer walk have both read it.
pub fn clear_loaded_scene() {
    LOADED.with(|cell| *cell.borrow_mut() = None);
}

/// Drop the memoised object list. The next reader rebuilds it.
pub fn clear_scene_objects() {
    BUILT.with(|cell| *cell.borrow_mut() = None);
}

/// The scoped loaded model, shared.
pub fn loaded_shared() -> Option<std::sync::Arc<SceneModel>> {
    LOADED.with(|cell| cell.borrow().clone())
}

/// Borrow the scoped loaded model and call `f` with `Option<&SceneModel>`.
/// The borrow is held only for the duration of `f`.
pub fn with_loaded<R>(f: impl FnOnce(Option<&SceneModel>) -> R) -> R {
    LOADED.with(|cell| {
        let borrow = cell.borrow();
        f(borrow.as_deref())
    })
}

/// The single Scene_Source both the Viewport and the Outliner call instead of
/// `current_scene(s, None)`. Reads the render-pass-scoped loaded model (set by
/// `BPhiberApp::render`): the Default_Scene when no model is in scope,
/// otherwise the loaded objects, each rigged mesh as the poser has it
/// (`scene::pose`). `current_scene` stays pure and exact.
pub fn scene_source(s: &BState) -> Vec<BObject3D> {
    (*scene_objects(s)).clone()
}

/// The same list, shared rather than copied — what the readers inside a render
/// pass should call.
///
/// The Viewport, the overlay, the Outliner and the frame trace each asked for
/// this list independently, and every ask rebuilt it: a `String` and a
/// materials `Vec` per object, several times a frame, for a list that cannot
/// change within one pass. It depends on nothing but the model in scope
/// ([`current_scene`] takes state only for its signature), so it is built once
/// per model and handed out by reference after that.
///
/// A rebuild is the cost of a new model coming into scope — loading a file —
/// which is where it belongs.
pub fn scene_objects(s: &BState) -> Rc<Vec<BObject3D>> {
    let model = LOADED.with(|cell| cell.borrow().clone());
    // A moving scene is rebuilt per frame, a still one only per model —
    // and either when something is added, deleted or duplicated.
    // A rigged mesh is drawn as the poser's thread has it (`scene::pose`).
    let frame = model.as_deref().and_then(|m| super::motion::moving_frame(m, super::motion::shown_frame(s)));
    let posed = model.as_ref().map(|m| super::pose::BPoser::shown(m, s)).unwrap_or_default();
    let at = (frame, posed.frame, super::edit::BSceneEdit::now(s).fingerprint() ^ super::transform::BTransforms::fingerprint(s).rotate_left(17) ^ super::shadow::BObjectShadowList::fingerprint(s).rotate_left(41));
    if let Some(hit) = BUILT.with(|cell| {
        cell.borrow()
            .as_ref()
            .filter(|(built, built_at, _)| same_model(built, &model) && *built_at == at)
            .map(|(_, _, objects)| Rc::clone(objects))
    }) {
        return hit;
    }
    OBJECT_BUILDS.with(|n| n.set(n.get() + 1));
    let built = Rc::new(posed.scene(s, model.as_deref()));
    BUILT.with(|cell| *cell.borrow_mut() = Some((model, at, Rc::clone(&built))));
    built
}

// ── Scene-level surfaces read by the non-mesh spaces ──────────────────────
//
// Each returns the loaded scene's value when a `.blend` is in scope, or the
// Blender factory default otherwise. Hosts (Time, Animation, Rendering,
// Shading, Compositing, Info, Statusbar) call these instead of reaching into
// the model directly — same pattern as `scene_source` for objects.

use crate::phiphx::scene::{
    SceneAction, SceneCamera, SceneLight, SceneRender, SceneTimeline, SceneWorld,
};

/// Timeline of the currently loaded scene, or Blender's default range.
pub fn scene_timeline() -> SceneTimeline {
    with_loaded(|loaded| loaded.map(|m| m.timeline).unwrap_or_default())
}

/// World / environment horizon colour.
pub fn scene_world() -> SceneWorld {
    with_loaded(|loaded| loaded.map(|m| m.world.clone()).unwrap_or_default())
}

/// Render settings — resolution + engine.
pub fn scene_render() -> SceneRender {
    with_loaded(|loaded| loaded.map(|m| m.render.clone()).unwrap_or_default())
}

/// Every camera object in the loaded scene where it stands at the frame in
/// scope, or an empty list.
pub fn scene_cameras() -> Vec<SceneCamera> {
    with_loaded(|loaded| loaded.map(|m| super::motion::cameras_at(m, super::motion::scoped_frame())).unwrap_or_default())
}

/// Every light object in the loaded scene where it stands at the frame in
/// scope, or an empty list.
pub fn scene_lights() -> Vec<SceneLight> {
    with_loaded(|loaded| loaded.map(|m| super::motion::lights_at(m, super::motion::scoped_frame())).unwrap_or_default())
}

/// Every animation Action in the loaded scene, or an empty list.
pub fn scene_actions() -> Vec<SceneAction> {
    with_loaded(|loaded| loaded.map(|m| m.actions.clone()).unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phiphx::scene::{SceneMesh, SceneModel, SceneNormals, SceneObject};
    use crate::v2::scene::OBJECTS;

    fn loaded_object(name: &str) -> SceneObject {
        SceneObject {
            name: name.to_string(),
            mesh: SceneMesh {
                vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
                indices: vec![0, 1, 2],
                normals: SceneNormals::PerFace(vec![[0.0, 0.0, 1.0]]),
                corners: Default::default(),
            }.into(),
            location: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            matrix_world: crate::phiphx::scene::IDENTITY_M4,
            materials: Vec::new(),
            material_indices: Default::default(),
            uvs: Default::default(),
            color: [1.0; 4],
            shadow: Default::default(),
        }
    }

    /// With no scope set, `scene_source` yields the Default_Scene; after
    /// `set_loaded_scene(Some(model))` it yields the loaded objects; after
    /// `clear_loaded_scene()` it is back to the Default_Scene. The three
    /// phases run in one test so they share (and reset) the same thread-local.
    #[test]
    fn scene_source_scopes_default_loaded_and_clear() {
        let s = BState::new();
        // Ensure a clean starting scope regardless of test ordering.
        clear_loaded_scene();

        // Phase 1: no scope -> Default_Scene primitives.
        let default = scene_source(&s);
        assert_eq!(default.len(), OBJECTS.len());
        assert!(
            default
                .iter()
                .all(|o| matches!(o, BObject3D::Primitive { .. }))
        );

        // Phase 2: scope set -> loaded objects in order.
        let model = SceneModel {
            objects: vec![loaded_object("Alpha"), loaded_object("Beta")],
            source_path: Some("/dir/model.blend".to_string()),
        ..Default::default()
        };
        set_loaded_scene(Some(model));
        let loaded = scene_source(&s);
        assert_eq!(loaded.len(), 2);
        match (&loaded[0], &loaded[1]) {
            (BObject3D::Loaded { name: a, .. }, BObject3D::Loaded { name: b, .. }) => {
                assert_eq!(a, "Alpha");
                assert_eq!(b, "Beta");
            }
            _ => panic!("expected two Loaded objects in scope"),
        }

        // Phase 3: cleared -> Default_Scene again.
        clear_loaded_scene();
        let after = scene_source(&s);
        assert_eq!(after.len(), OBJECTS.len());
        assert!(
            after
                .iter()
                .all(|o| matches!(o, BObject3D::Primitive { .. }))
        );
    }

    /// `set_loaded_scene(None)` behaves like a clear.
    #[test]
    fn set_loaded_scene_none_yields_default() {
        let s = BState::new();
        set_loaded_scene(None);
        let scene = scene_source(&s);
        assert_eq!(scene.len(), OBJECTS.len());
        assert!(
            scene
                .iter()
                .all(|o| matches!(o, BObject3D::Primitive { .. }))
        );
        clear_loaded_scene();
    }
}

#[cfg(test)]
mod memo_tests {
    use super::*;
    use crate::phiphx::scene::{SceneMesh, SceneNormals, SceneObject};
    use crate::v2::scene::OBJECTS;

    fn model(objects: usize) -> SceneModel {
        let mesh = SceneMesh {
            vertices: vec![[0.0, 0.0, 0.0]; 3],
            indices: vec![0, 1, 2],
            normals: SceneNormals::PerVertex(vec![[0.0, 0.0, 1.0]; 3]),
            corners: Default::default(),
        };
        SceneModel {
            objects: (0..objects)
                .map(|i| SceneObject::identity(format!("Object.{i:03}"), mesh.clone()))
                .collect(),
            ..SceneModel::default()
        }
    }

    /// The Viewport, the overlay, the Outliner and the trace all read the
    /// scene in one pass. They must share one build, not take one each.
    #[test]
    fn the_readers_in_one_pass_share_one_build() {
        let s = BState::new();
        clear_scene_objects();
        set_loaded_shared(Some(std::sync::Arc::new(model(4))));
        let before = object_builds();
        let first = scene_objects(&s);
        for _ in 0..3 {
            let again = scene_objects(&s);
            assert!(Rc::ptr_eq(&first, &again), "each reader rebuilt the list");
        }
        assert_eq!(
            object_builds() - before,
            1,
            "four readers in one pass must cost one build"
        );
        clear_loaded_scene();
        clear_scene_objects();
    }

    /// A new file is exactly what a rebuild is for.
    #[test]
    fn a_new_model_rebuilds_and_its_contents_follow() {
        let s = BState::new();
        clear_scene_objects();
        set_loaded_shared(Some(std::sync::Arc::new(model(2))));
        assert_eq!(scene_objects(&s).len(), 2);
        let before = object_builds();
        set_loaded_shared(Some(std::sync::Arc::new(model(5))));
        let after = scene_objects(&s);
        assert_eq!(after.len(), 5, "the memo served the previous file's objects");
        assert_eq!(object_builds() - before, 1, "a new model must build once");

        // And clearing the scope returns the Default_Scene, not a stale list.
        clear_loaded_scene();
        assert_eq!(scene_objects(&s).len(), OBJECTS.len());
        clear_scene_objects();
    }
}

/// What the view layer shows, counted: objects, vertices, faces.
///
/// Blender's `info_stats` counts what is VISIBLE, not what the file holds —
/// a character with three hidden hair variants reads 1.4M faces there and
/// read 9M here until the hidden ones were filtered out. The viewport's
/// read-out and the status bar both ask this, so the two can never disagree
/// about how big the scene is.
pub fn visible_metrics(s: &BState) -> (usize, usize, usize) {
    let hidden = crate::v2::part::outliner::key::BHidden::now(s);
    let visible: Vec<BObject3D> = scene_objects(s)
        .iter()
        .filter(|o| !hidden.hides(s, object_name(o)))
        .cloned()
        .collect();
    super::scene_metrics(&visible)
}

/// That count as one line, in Blender's status-bar wording.
pub fn stats_line(s: &BState) -> String {
    let (objects, verts, faces) = visible_metrics(s);
    format!("Objects: {objects} | Verts: {verts} | Faces: {faces}")
}

/// What an object is called. One match, in the layer that owns the enum: the
/// painter, the pick and the stats all name an object the same way.
pub fn object_name(object: &BObject3D) -> &str {
    match object {
        BObject3D::Primitive { name, .. } | BObject3D::Loaded { name, .. } => name,
    }
}
