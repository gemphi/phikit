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

use crate::phiphx::services::scene::SceneModel;
use crate::v2::scene::{BObject3D, current_scene};
use crate::v2::state::BState;

thread_local! {
    /// The loaded scene model in scope for the current render pass, or `None`
    /// when no blend is loaded / the scope has been cleared. Owned clone.
    static LOADED: RefCell<Option<SceneModel>> = const { RefCell::new(None) };
}

/// Set the loaded scene model in scope for the current render pass. Pass
/// `Some(model)` (an owned clone from `scene_cache`) to render the loaded
/// objects, or `None` to render the Default_Scene. Called at the top of
/// `BPhiberApp::render`.
pub fn set_loaded_scene(model: Option<SceneModel>) {
    LOADED.with(|cell| *cell.borrow_mut() = model);
}

/// Clear the loaded scene scope so it does not leak into the next render pass.
/// Called at the bottom of `BPhiberApp::render`, after the tree build and the
/// renderer walk have both read it.
pub fn clear_loaded_scene() {
    LOADED.with(|cell| *cell.borrow_mut() = None);
}

/// Borrow the scoped loaded model and call `f` with `Option<&SceneModel>`.
/// The borrow is held only for the duration of `f`.
pub fn with_loaded<R>(f: impl FnOnce(Option<&SceneModel>) -> R) -> R {
    LOADED.with(|cell| {
        let borrow = cell.borrow();
        f(borrow.as_ref())
    })
}

/// The single Scene_Source both the Viewport and the Outliner call instead of
/// `current_scene(s, None)`. Reads the render-pass-scoped loaded model (set by
/// `BPhiberApp::render`) and defers to the pure [`current_scene`]: yields the
/// Default_Scene when no model is in scope, otherwise the loaded objects.
///
/// [`current_scene`] stays pure and unchanged — this is the only new surface;
/// the scope read happens here, so unit tests calling `current_scene(s, None)`
/// / `current_scene(s, Some(&m))` are unaffected.
pub fn scene_source(s: &BState) -> Vec<BObject3D> {
    with_loaded(|loaded| current_scene(s, loaded))
}

// ── Scene-level surfaces read by the non-mesh spaces ──────────────────────
//
// Each returns the loaded scene's value when a `.blend` is in scope, or the
// Blender factory default otherwise. Hosts (Time, Animation, Rendering,
// Shading, Compositing, Info, Statusbar) call these instead of reaching into
// the model directly — same pattern as `scene_source` for objects.

use crate::phiphx::services::scene::{
    SceneAction, SceneCamera, SceneLight, SceneRender, SceneTimeline, SceneWorld,
};

/// Timeline of the currently loaded scene, or Blender's default range.
pub fn scene_timeline() -> SceneTimeline {
    with_loaded(|loaded| loaded.map(|m| m.timeline).unwrap_or_default())
}

/// World / environment horizon colour.
pub fn scene_world() -> SceneWorld {
    with_loaded(|loaded| loaded.map(|m| m.world).unwrap_or_default())
}

/// Render settings — resolution + engine.
pub fn scene_render() -> SceneRender {
    with_loaded(|loaded| loaded.map(|m| m.render.clone()).unwrap_or_default())
}

/// Every camera object in the loaded scene, or an empty list.
pub fn scene_cameras() -> Vec<SceneCamera> {
    with_loaded(|loaded| loaded.map(|m| m.cameras.clone()).unwrap_or_default())
}

/// Every light object in the loaded scene, or an empty list.
pub fn scene_lights() -> Vec<SceneLight> {
    with_loaded(|loaded| loaded.map(|m| m.lights.clone()).unwrap_or_default())
}

/// Every animation Action in the loaded scene, or an empty list.
pub fn scene_actions() -> Vec<SceneAction> {
    with_loaded(|loaded| loaded.map(|m| m.actions.clone()).unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phiphx::services::scene::{SceneMesh, SceneModel, SceneNormals, SceneObject};
    use crate::v2::scene::OBJECTS;

    fn loaded_object(name: &str) -> SceneObject {
        SceneObject {
            name: name.to_string(),
            mesh: SceneMesh {
                vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
                indices: vec![0, 1, 2],
                normals: SceneNormals::PerFace(vec![[0.0, 0.0, 1.0]]),
            },
            location: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            materials: Vec::new(),
            material_indices: Vec::new(),
            uvs: Vec::new(),
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
