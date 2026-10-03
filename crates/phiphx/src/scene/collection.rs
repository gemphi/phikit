//! Collection tree + view-layer state of a loaded scene — what Blender's
//! Outliner "View Layer" mode draws (`tree_display_view_layer.cc`).
//!
//! Mirrors `Scene.master_collection` → `Collection.children` with the
//! `LayerCollection` exclude / hide flags folded in, and the `Base` flags
//! (selected / hidden / active) per object. Pure data, kit-agnostic.

use crate::blend::{BlendCollection, BlendScene};

/// One collection node. `object_indices` index into `SceneModel::objects`
/// **after** cameras and lights were partitioned out, so they are remapped
/// from the blend indices by [`collections_from_blend`].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SceneCollection {
    pub name: String,
    pub children: Vec<SceneCollection>,
    /// Mesh-type objects directly in this collection (indices into `objects`).
    pub object_indices: Vec<usize>,
    /// Names of non-mesh members (cameras, lights) directly in this collection.
    pub extra_names: Vec<String>,
    /// `LAYER_COLLECTION_EXCLUDE` — checkbox off in the outliner.
    pub exclude: bool,
    /// `LAYER_COLLECTION_HIDE` — eye closed in the outliner.
    pub hide_viewport: bool,
    /// `Collection.flag`: the monitor, the arrow and the camera toggles.
    pub visibility: crate::blend::BlendVisibility,
}

/// Selection / visibility state of `SceneModel::objects`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SceneSelection {
    /// Name of the view layer Blender opened (`ViewLayer.name`).
    pub view_layer: String,
    pub active: Option<String>,
    pub selected: Vec<String>,
    pub hidden: Vec<String>,
    /// `Object.visibility_flag` per object: disabled in viewports, not
    /// selectable, disabled in renders (`OB_HIDE_VIEWPORT / _SELECT / _RENDER`).
    pub viewport_disabled: Vec<String>,
    pub unselectable: Vec<String>,
    pub render_disabled: Vec<String>,
}

/// `remap[blend_index] = Some(objects_index)` for mesh objects, `None` for
/// cameras / lights (they live in `cameras` / `lights`).
pub(super) fn collections_from_blend(
    blend: &BlendScene,
    objects: &[crate::blend::BlendSceneObject],
    remap: &[Option<usize>],
) -> Vec<SceneCollection> {
    blend
        .collections
        .iter()
        .map(|c| collection_node(c, objects, remap))
        .collect()
}

fn collection_node(c: &BlendCollection, objects: &[crate::blend::BlendSceneObject], remap: &[Option<usize>]) -> SceneCollection {
    let mut object_indices = Vec::new();
    let mut extra_names = Vec::new();
    for &bi in &c.object_indices {
        match remap.get(bi).copied().flatten() {
            Some(i) => object_indices.push(i),
            None => {
                if let Some(o) = objects.get(bi) {
                    extra_names.push(o.name.clone());
                }
            }
        }
    }
    SceneCollection {
        name: c.name.clone(),
        children: c.children.iter().map(|k| collection_node(k, objects, remap)).collect(),
        object_indices,
        extra_names,
        exclude: c.exclude,
        hide_viewport: c.hide_viewport,
        visibility: c.visibility,
    }
}

pub(super) fn selection_from_blend(blend: &BlendScene, objects: &[crate::blend::BlendSceneObject]) -> SceneSelection {
    let name = |i: &usize| objects.get(*i).map(|o| o.name.clone());
    SceneSelection {
        view_layer: blend.props.view_layer_name.clone(),
        active: blend.active_object.as_ref().and_then(name),
        selected: blend.selected.iter().filter_map(name).collect(),
        hidden: blend.hidden.iter().filter_map(name).collect(),
        viewport_disabled: objects.iter().filter(|o| o.visibility.hide_viewport).map(|o| o.name.clone()).collect(),
        unselectable: objects.iter().filter(|o| o.visibility.hide_select).map(|o| o.name.clone()).collect(),
        render_disabled: objects.iter().filter(|o| o.visibility.hide_render).map(|o| o.name.clone()).collect(),
    }
}
