//! Outliner state keys and the small predicates that read them
//! (expansion, search, display mode, sort, per-object restrictions).

use super::set::BOutlineMode;
use super::tree::intern_name;
use crate::v2::state::{
    BKey as _, BPath, BState, OutlinerKey,
    val::{V_FALSE, V_TRUE},
};


/// Per-object restriction flag, e.g. `ctx.outliner.Cube.hide_render` —
/// the object's name interned so the key is `'static`.
pub fn k_restrict(object: &str, field: &'static str) -> BPath {
    OutlinerKey::Restrict(intern_name(object), field).path()
}

/// Is `object` hidden in the viewport? Read by the 3D viewport, which is what
/// turns the eye icon from decoration into a working control.
pub fn is_hidden(s: &BState, object: &str) -> bool {
    s.get(&k_restrict(object, "hide_get")) == Some(V_TRUE)
        || s.get(&k_restrict(object, "hide_viewport")) == Some(V_TRUE)
}

/// Is the collection row `id` unchecked (`LAYER_COLLECTION_EXCLUDE`)? An
/// excluded collection is out of the view layer altogether: not drawn,
/// not rendered, not selectable.
pub fn is_excluded(s: &BState, id: &str) -> bool {
    s.get(&k_restrict(id, "exclude")) == Some(V_TRUE)
}

/// Is `object` left out of renders (the camera toggle, `OB_HIDE_RENDER`)?
/// The eye does not decide this: Blender renders what the viewport hides.
pub fn is_render_hidden(s: &BState, object: &str) -> bool {
    s.get(&k_restrict(object, "hide_render")) == Some(V_TRUE)
}

/// Can `object` not be picked (the arrow toggle, `OB_HIDE_SELECT`)?
pub fn is_unselectable(s: &BState, object: &str) -> bool {
    s.get(&k_restrict(object, "hide_select")) == Some(V_TRUE)
}

/// What one pass leaves out: objects with their own toggle set, plus
/// everything under a collection whose toggle is set. Built once per frame
/// and asked per object, so the walk over the collection tree is not repeated
/// for each of a few hundred objects.
pub struct BHidden {
    under: std::collections::HashSet<String>,
    own: fn(&BState, &str) -> bool,
}

impl BHidden {
    /// The viewport's: the eye, and the monitor.
    pub fn now(s: &BState) -> Self {
        Self::by(s, is_hidden)
    }

    /// A render's: the camera.
    pub fn for_render(s: &BState) -> Self {
        Self::by(s, is_render_hidden)
    }

    /// A pick's: the arrow.
    pub fn for_select(s: &BState) -> Self {
        Self::by(s, is_unselectable)
    }

    fn by(s: &BState, own: fn(&BState, &str) -> bool) -> Self {
        let under = crate::v2::scene::source::with_loaded(|m| {
            m.map(|m| super::layer::hidden_by_collections(s, m, own)).unwrap_or_default()
        });
        Self { under, own }
    }

    pub fn hides(&self, s: &BState, object: &str) -> bool {
        (self.own)(s, object) || self.under.contains(object)
    }
}

/// Expansion state with a per-row default.
///
/// Blender opens the Scene Collection and its collections but leaves objects
/// closed, so their data-blocks (mesh, materials, modifiers) stay out of the
/// way until asked for. A single "default open" rule would show a Mesh row
/// under every object on first paint.
pub fn is_expanded_or(s: &BState, id: &'static str, default_open: bool) -> bool {
    match s.get(OutlinerKey::Expanded(id)) {
        Some(V_FALSE) => false,
        Some(V_TRUE) => true,
        _ => default_open,
    }
}

/// Rows default to expanded; only an explicit `"false"` collapses one.
pub fn is_expanded(s: &BState, id: &'static str) -> bool {
    is_expanded_or(s, id, true)
}

pub fn search_text(s: &BState) -> String {
    s.get(OutlinerKey::Search).unwrap_or("").trim().to_lowercase()
}

pub fn display_mode(s: &BState) -> BOutlineMode {
    s.get(OutlinerKey::DisplayMode)
        .and_then(BOutlineMode::from_slug)
        .unwrap_or(BOutlineMode::ViewLayer)
}

/// Blender sorts alphabetically by default; the UI flag is the inverted
/// `SO_SKIP_SORT_ALPHA`, so only an explicit `"false"` turns sorting off.
pub fn sort_alpha(s: &BState) -> bool {
    s.get(OutlinerKey::SortAlpha) != Some(V_FALSE)
}

